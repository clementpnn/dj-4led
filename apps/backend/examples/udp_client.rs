use flate2::read::GzDecoder;
use std::io::{Cursor, Read, Write};
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

// Protocol constants
const MAX_PACKET_SIZE: usize = 1472;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum PacketType {
    Connect = 0x01,
    Disconnect = 0x02,
    Ping = 0x03,
    Pong = 0x04,
    Ack = 0x05,
    Command = 0x10,
    FrameData = 0x20,
    FrameDataCompressed = 0x21,
    SpectrumData = 0x30,
}

impl PacketType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Connect),
            0x02 => Some(Self::Disconnect),
            0x03 => Some(Self::Ping),
            0x04 => Some(Self::Pong),
            0x05 => Some(Self::Ack),
            0x10 => Some(Self::Command),
            0x20 => Some(Self::FrameData),
            0x21 => Some(Self::FrameDataCompressed),
            0x30 => Some(Self::SpectrumData),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct UdpPacket {
    packet_type: PacketType,
    flags: u8,
    sequence: u32,
    fragment_id: u16,
    fragment_count: u16,
    payload: Vec<u8>,
}

impl UdpPacket {
    fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if data.len() < 12 {
            return Err("Packet too small".into());
        }

        let mut cursor = Cursor::new(data);

        let mut type_byte = [0u8; 1];
        cursor.read_exact(&mut type_byte)?;
        let packet_type = PacketType::from_u8(type_byte[0]).ok_or("Invalid packet type")?;

        let mut flags_byte = [0u8; 1];
        cursor.read_exact(&mut flags_byte)?;
        let flags = flags_byte[0];

        let mut sequence_bytes = [0u8; 4];
        cursor.read_exact(&mut sequence_bytes)?;
        let sequence = u32::from_le_bytes(sequence_bytes);

        let mut fragment_id_bytes = [0u8; 2];
        cursor.read_exact(&mut fragment_id_bytes)?;
        let fragment_id = u16::from_le_bytes(fragment_id_bytes);

        let mut fragment_count_bytes = [0u8; 2];
        cursor.read_exact(&mut fragment_count_bytes)?;
        let fragment_count = u16::from_le_bytes(fragment_count_bytes);

        let mut payload_len_bytes = [0u8; 2];
        cursor.read_exact(&mut payload_len_bytes)?;
        let payload_len = u16::from_le_bytes(payload_len_bytes) as usize;

        let mut payload = vec![0u8; payload_len];
        cursor.read_exact(&mut payload)?;

        Ok(Self {
            packet_type,
            flags,
            sequence,
            fragment_id,
            fragment_count,
            payload,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(12 + self.payload.len());

        buffer.push(self.packet_type as u8);
        buffer.push(self.flags);
        buffer.extend_from_slice(&self.sequence.to_le_bytes());
        buffer.extend_from_slice(&self.fragment_id.to_le_bytes());
        buffer.extend_from_slice(&self.fragment_count.to_le_bytes());
        buffer.extend_from_slice(&(self.payload.len() as u16).to_le_bytes());
        buffer.extend_from_slice(&self.payload);

        buffer
    }
}

fn create_connect_packet(enable_compression: bool) -> Vec<u8> {
    let packet = UdpPacket {
        packet_type: PacketType::Connect,
        flags: if enable_compression { 0x01 } else { 0x00 },
        sequence: 0,
        fragment_id: 0,
        fragment_count: 1,
        payload: vec![],
    };
    packet.to_bytes()
}

fn create_ping_packet(sequence: u32) -> Vec<u8> {
    let packet = UdpPacket {
        packet_type: PacketType::Ping,
        flags: 0,
        sequence,
        fragment_id: 0,
        fragment_count: 1,
        payload: vec![],
    };
    packet.to_bytes()
}

fn create_command_set_effect(sequence: u32, effect_id: u32) -> Vec<u8> {
    let mut payload = vec![0x01]; // Command ID for SetEffect
    payload.extend_from_slice(&effect_id.to_le_bytes());

    let packet = UdpPacket {
        packet_type: PacketType::Command,
        flags: 0,
        sequence,
        fragment_id: 0,
        fragment_count: 1,
        payload,
    };
    packet.to_bytes()
}

fn create_command_set_color(sequence: u32, r: f32, g: f32, b: f32) -> Vec<u8> {
    let mut payload = vec![0x03]; // Command ID for SetCustomColor
    payload.extend_from_slice(&r.to_le_bytes());
    payload.extend_from_slice(&g.to_le_bytes());
    payload.extend_from_slice(&b.to_le_bytes());

    let packet = UdpPacket {
        packet_type: PacketType::Command,
        flags: 0,
        sequence,
        fragment_id: 0,
        fragment_count: 1,
        payload,
    };
    packet.to_bytes()
}

fn process_frame_data(payload: &[u8], compressed: bool) {
    let data = if compressed {
        // Decompress the payload
        let mut decoder = GzDecoder::new(payload);
        let mut decompressed = Vec::new();
        if decoder.read_to_end(&mut decompressed).is_err() {
            eprintln!("Failed to decompress frame data");
            return;
        }
        decompressed
    } else {
        payload.to_vec()
    };

    if data.len() < 5 {
        eprintln!("Frame data too small");
        return;
    }

    let width = u16::from_le_bytes([data[0], data[1]]);
    let height = u16::from_le_bytes([data[2], data[3]]);
    let format = data[4];
    let pixel_data = &data[5..];

    println!(
        "Frame: {}x{}, format: {}, {} bytes",
        width,
        height,
        format,
        pixel_data.len()
    );
}

fn process_spectrum_data(payload: &[u8]) {
    if payload.len() < 2 {
        eprintln!("Spectrum data too small");
        return;
    }

    let band_count = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    let expected_size = 2 + band_count * 4;

    if payload.len() != expected_size {
        eprintln!("Invalid spectrum data size");
        return;
    }

    let mut bands = Vec::with_capacity(band_count);
    for i in 0..band_count {
        let offset = 2 + i * 4;
        let value = f32::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
        ]);
        bands.push(value);
    }

    // Calculate average for display
    let avg: f32 = bands.iter().sum::<f32>() / bands.len() as f32;
    println!("Spectrum: {} bands, avg: {:.2}", band_count, avg);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("UDP Client Example for DJ-4LED");
    println!("==============================");

    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("localhost:8080")?;
    socket.set_nonblocking(true)?;

    println!("Connected to localhost:8080");

    // Send connect packet
    let connect_packet = create_connect_packet(true);
    socket.send(&connect_packet)?;
    println!("Sent connect packet (compression enabled)");

    let mut sequence = 1u32;
    let mut last_ping = Instant::now();
    let mut last_command = Instant::now();
    let mut effect_id = 0u32;
    let mut receive_buffer = [0u8; 2048];
    let mut frame_count = 0u64;
    let mut spectrum_count = 0u64;
    let start_time = Instant::now();

    // Main loop
    loop {
        // Receive packets
        match socket.recv(&mut receive_buffer) {
            Ok(len) => match UdpPacket::from_bytes(&receive_buffer[..len]) {
                Ok(packet) => match packet.packet_type {
                    PacketType::Ack => {
                        println!("Received ACK for sequence {}", packet.sequence);
                    }
                    PacketType::Pong => {
                        println!("Received PONG for sequence {}", packet.sequence);
                    }
                    PacketType::FrameData => {
                        process_frame_data(&packet.payload, false);
                        frame_count += 1;
                    }
                    PacketType::FrameDataCompressed => {
                        process_frame_data(&packet.payload, true);
                        frame_count += 1;
                    }
                    PacketType::SpectrumData => {
                        process_spectrum_data(&packet.payload);
                        spectrum_count += 1;
                    }
                    _ => {
                        println!("Received packet type: {:?}", packet.packet_type);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to parse packet: {}", e);
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available, continue
            }
            Err(e) => {
                eprintln!("Receive error: {}", e);
            }
        }

        // Send periodic ping
        if last_ping.elapsed() > Duration::from_secs(30) {
            let ping_packet = create_ping_packet(sequence);
            socket.send(&ping_packet)?;
            println!("Sent PING (sequence {})", sequence);
            sequence += 1;
            last_ping = Instant::now();
        }

        // Send periodic commands for testing
        if last_command.elapsed() > Duration::from_secs(5) {
            // Alternate between different effects
            effect_id = (effect_id + 1) % 10;

            if effect_id % 2 == 0 {
                // Send effect change
                let cmd_packet = create_command_set_effect(sequence, effect_id);
                socket.send(&cmd_packet)?;
                println!("Sent command: SetEffect({})", effect_id);
            } else {
                // Send color change
                let r = ((effect_id as f32) * 0.1) % 1.0;
                let g = ((effect_id as f32) * 0.2) % 1.0;
                let b = ((effect_id as f32) * 0.3) % 1.0;
                let cmd_packet = create_command_set_color(sequence, r, g, b);
                socket.send(&cmd_packet)?;
                println!("Sent command: SetColor({:.2}, {:.2}, {:.2})", r, g, b);
            }

            sequence += 1;
            last_command = Instant::now();
        }

        // Print statistics every 10 seconds
        if start_time.elapsed().as_secs() > 0 && start_time.elapsed().as_secs() % 10 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let fps = frame_count as f64 / elapsed;
            let sps = spectrum_count as f64 / elapsed;

            println!("\n--- Statistics ---");
            println!("Frames: {} ({:.1} FPS)", frame_count, fps);
            println!("Spectrums: {} ({:.1} SPS)", spectrum_count, sps);
            println!("------------------\n");

            thread::sleep(Duration::from_secs(1)); // Avoid printing multiple times
        }

        // Small sleep to avoid busy waiting
        thread::sleep(Duration::from_millis(1));
    }
}
