use crate::AppState;
use anyhow::Result;
use parking_lot::Mutex;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod frame_processor;
mod protocol;

pub use frame_processor::UdpFrameProcessor;
pub use protocol::*;

pub struct UdpServer {
    state: Arc<AppState>,
    socket: UdpSocket,
    clients: Arc<Mutex<Vec<ClientInfo>>>,
}

#[derive(Clone)]
struct ClientInfo {
    addr: SocketAddr,
    last_seen: Instant,
    packet_counter: u32,
    compression_enabled: bool,
}

impl UdpServer {
    pub fn new(state: Arc<AppState>) -> Result<Self> {
        let socket = match UdpSocket::bind("0.0.0.0:8081") {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        };

        match socket.set_nonblocking(true) {
            Ok(_) => println!(""),
            Err(e) => {
                return Err(e.into());
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::io::AsRawFd;
            let fd = socket.as_raw_fd();
            unsafe {
                let size: libc::c_int = 2 * 1024 * 1024;
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_RCVBUF,
                    &size as *const _ as *const libc::c_void,
                    std::mem::size_of::<libc::c_int>() as libc::socklen_t,
                );
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_SNDBUF,
                    &size as *const _ as *const libc::c_void,
                    std::mem::size_of::<libc::c_int>() as libc::socklen_t,
                );
            }
        }

        Ok(Self {
            state,
            socket,
            clients: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn run(self) -> Result<()> {
        let state = self.state.clone();
        let clients = self.clients.clone();
        let socket = self.socket.try_clone()?;

        thread::spawn(
            move || {
                if let Err(e) = Self::sender_loop(socket, state, clients) {}
            },
        );

        self.receiver_loop()
    }

    fn sender_loop(
        socket: UdpSocket,
        state: Arc<AppState>,
        clients: Arc<Mutex<Vec<ClientInfo>>>,
    ) -> Result<()> {
        let mut processor = UdpFrameProcessor::new();
        let mut last_cleanup = Instant::now();
        let mut stats = TransmissionStats::new();

        loop {
            if last_cleanup.elapsed() > Duration::from_secs(30) {
                let mut clients_list = clients.lock();
                clients_list.retain(|c| c.last_seen.elapsed() < Duration::from_secs(60));
                last_cleanup = Instant::now();
            }

            let frame = state.led_frame.lock().clone();
            let spectrum = state.spectrum.lock().clone();

            let clients_snapshot = clients.lock().clone();

            for mut client in clients_snapshot {
                let packets = processor.prepare_packets(
                    &frame,
                    &spectrum,
                    client.packet_counter,
                    client.compression_enabled,
                );

                for packet in packets {
                    if let Ok(packet_data) = packet.to_bytes() {
                        match socket.send_to(&packet_data, client.addr) {
                            Ok(bytes_sent) => {
                                stats.add_packet(bytes_sent);
                                client.packet_counter = client.packet_counter.wrapping_add(1);
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }

            if stats.should_print() {
                stats.print_and_reset();
            }

            thread::sleep(Duration::from_micros(16_666));
        }
    }

    fn receiver_loop(&self) -> Result<()> {
        let mut buf = [0u8; 1024];
        let mut packets_received = 0u64;
        let mut last_log = Instant::now();

        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    packets_received += 1;

                    if let Ok(packet) = UdpPacket::from_bytes(&buf[..len]) {
                        self.handle_packet(packet, addr);
                    } else {
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if last_log.elapsed() > Duration::from_secs(10) {
                        last_log = Instant::now();
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {}
            }
        }
    }

    fn handle_packet(&self, packet: UdpPacket, addr: SocketAddr) {
        match packet.packet_type {
            PacketType::Connect => {
                let mut clients = self.clients.lock();
                if let Some(client) = clients.iter_mut().find(|c| c.addr == addr) {
                    client.last_seen = Instant::now();
                } else {
                    clients.push(ClientInfo {
                        addr,
                        last_seen: Instant::now(),
                        packet_counter: 0,
                        compression_enabled: packet.flags.contains(PacketFlags::COMPRESSED),
                    });
                }

                let ack = UdpPacket::new_ack(packet.sequence);
                if let Ok(data) = ack.to_bytes() {
                    let _ = self.socket.send_to(&data, addr);
                }
            }

            PacketType::Command => {
                {
                    let mut clients = self.clients.lock();
                    if let Some(client) = clients.iter_mut().find(|c| c.addr == addr) {
                        client.last_seen = Instant::now();
                    }
                }

                if let Some(command) = UdpCommand::from_payload(&packet.payload) {
                    self.process_command(command);
                }
            }

            PacketType::Ping => {
                let pong = UdpPacket::new_pong(packet.sequence);
                if let Ok(data) = pong.to_bytes() {
                    let _ = self.socket.send_to(&data, addr);
                }
            }

            PacketType::Disconnect => {
                let mut clients = self.clients.lock();
                clients.retain(|c| c.addr != addr);
            }

            _ => {}
        }
    }

    fn process_command(&self, command: UdpCommand) {
        match command {
            UdpCommand::SetEffect(effect_id) => {
                self.state.effect_engine.lock().set_effect(effect_id);
            }

            UdpCommand::SetColorMode(mode) => {
                self.state.effect_engine.lock().set_color_mode(&mode);
            }

            UdpCommand::SetCustomColor(r, g, b) => {
                self.state.effect_engine.lock().set_custom_color(r, g, b);
            }

            UdpCommand::SetParameter(name, value) => {}
        }
    }
}

struct TransmissionStats {
    packets_sent: u64,
    bytes_sent: u64,
    start_time: Instant,
    last_print: Instant,
}

impl TransmissionStats {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            packets_sent: 0,
            bytes_sent: 0,
            start_time: now,
            last_print: now,
        }
    }

    fn add_packet(&mut self, bytes: usize) {
        self.packets_sent += 1;
        self.bytes_sent += bytes as u64;
    }

    fn should_print(&self) -> bool {
        self.last_print.elapsed() > Duration::from_secs(10)
    }

    fn print_and_reset(&mut self) {
        let elapsed = self.last_print.elapsed().as_secs_f64();
        let pps = self.packets_sent as f64 / elapsed;
        let mbps = (self.bytes_sent as f64 * 8.0) / (elapsed * 1_000_000.0);

        self.packets_sent = 0;
        self.bytes_sent = 0;
        self.last_print = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_info() {
        let client = ClientInfo {
            addr: "127.0.0.1:1234".parse().unwrap(),
            last_seen: Instant::now(),
            packet_counter: 0,
            compression_enabled: false,
        };

        assert_eq!(client.packet_counter, 0);
        assert!(!client.compression_enabled);
    }
}
