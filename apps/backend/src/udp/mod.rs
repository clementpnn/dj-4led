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
        let socket = UdpSocket::bind("0.0.0.0:8080")?;
        socket.set_nonblocking(true)?;

        // Optimisations UDP
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::io::AsRawFd;
            let fd = socket.as_raw_fd();
            unsafe {
                // Augmenter les buffers de réception/envoi
                let size: libc::c_int = 2 * 1024 * 1024; // 2MB
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_RCVBUF,
                    &size as *const _ as *const libc::c_void,
                    std::mem::size_of::<libc::c_int>() as u32,
                );
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_SNDBUF,
                    &size as *const _ as *const libc::c_void,
                    std::mem::size_of::<libc::c_int>() as u32,
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
        println!("🚀 UDP server listening on udp://0.0.0.0:8080");

        let state = self.state.clone();
        let clients = self.clients.clone();
        let socket = self.socket.try_clone()?;

        // Thread pour envoyer les données aux clients
        thread::spawn(move || {
            if let Err(e) = Self::sender_loop(socket, state, clients) {
                eprintln!("UDP sender error: {}", e);
            }
        });

        // Thread principal pour recevoir les commandes
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
            // Nettoyage périodique des clients inactifs
            if last_cleanup.elapsed() > Duration::from_secs(30) {
                let mut clients_list = clients.lock();
                clients_list.retain(|c| c.last_seen.elapsed() < Duration::from_secs(60));
                last_cleanup = Instant::now();
            }

            // Obtenir les données actuelles
            let frame = state.led_frame.lock().clone();
            let spectrum = state.spectrum.lock().clone();

            // Traiter et envoyer aux clients actifs
            let clients_snapshot = clients.lock().clone();

            for mut client in clients_snapshot {
                // Préparer les paquets selon le type de client
                let packets = processor.prepare_packets(
                    &frame,
                    &spectrum,
                    client.packet_counter,
                    client.compression_enabled,
                );

                // Envoyer les paquets
                for packet in packets {
                    if let Ok(packet_data) = packet.to_bytes() {
                        match socket.send_to(&packet_data, client.addr) {
                            Ok(bytes_sent) => {
                                stats.add_packet(bytes_sent);
                                client.packet_counter = client.packet_counter.wrapping_add(1);
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // Buffer plein, on skip ce client pour cette frame
                                break;
                            }
                            Err(_) => {
                                // Erreur réseau, on continue avec les autres clients
                                break;
                            }
                        }
                    }
                }
            }

            // Afficher les stats toutes les 10 secondes
            if stats.should_print() {
                stats.print_and_reset();
            }

            // Limiter à ~60 FPS
            thread::sleep(Duration::from_micros(16_666));
        }
    }

    fn receiver_loop(&self) -> Result<()> {
        let mut buf = [0u8; 1024];

        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    if let Ok(packet) = UdpPacket::from_bytes(&buf[..len]) {
                        self.handle_packet(packet, addr);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("UDP receive error: {}", e);
                }
            }
        }
    }

    fn handle_packet(&self, packet: UdpPacket, addr: SocketAddr) {
        match packet.packet_type {
            PacketType::Connect => {
                println!("🔌 New UDP client connected from {}", addr);

                // Ajouter ou mettre à jour le client
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

                // Envoyer un ACK
                let ack = UdpPacket::new_ack(packet.sequence);
                if let Ok(data) = ack.to_bytes() {
                    let _ = self.socket.send_to(&data, addr);
                }
            }

            PacketType::Command => {
                // Mettre à jour le timestamp du client
                {
                    let mut clients = self.clients.lock();
                    if let Some(client) = clients.iter_mut().find(|c| c.addr == addr) {
                        client.last_seen = Instant::now();
                    }
                }

                // Traiter la commande
                if let Some(command) = UdpCommand::from_payload(&packet.payload) {
                    self.process_command(command);
                }
            }

            PacketType::Ping => {
                // Répondre avec un pong
                let pong = UdpPacket::new_pong(packet.sequence);
                if let Ok(data) = pong.to_bytes() {
                    let _ = self.socket.send_to(&data, addr);
                }
            }

            PacketType::Disconnect => {
                println!("🔌 UDP client disconnected from {}", addr);
                let mut clients = self.clients.lock();
                clients.retain(|c| c.addr != addr);
            }

            _ => {}
        }
    }

    fn process_command(&self, command: UdpCommand) {
        match command {
            UdpCommand::SetEffect(effect_id) => {
                println!("🎨 Changing effect to: {}", effect_id);
                self.state.effect_engine.lock().set_effect(effect_id);
            }

            UdpCommand::SetColorMode(mode) => {
                println!("🎨 Setting color mode: {}", mode);
                self.state.effect_engine.lock().set_color_mode(&mode);
            }

            UdpCommand::SetCustomColor(r, g, b) => {
                println!("🎨 Setting custom color: RGB({}, {}, {})", r, g, b);
                self.state.effect_engine.lock().set_custom_color(r, g, b);
            }

            UdpCommand::SetParameter(name, value) => {
                println!("🎛️  Parameter change: {} = {}", name, value);
                // Traiter d'autres paramètres si nécessaire
            }
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

        println!(
            "📊 UDP Stats: {:.0} packets/s, {:.2} Mbps, {} total packets",
            pps, mbps, self.packets_sent
        );

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
