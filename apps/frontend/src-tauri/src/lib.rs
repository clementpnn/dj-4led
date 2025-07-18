// src-tauri/src/lib.rs
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{State, Window, Emitter};
use serde_json::json;

// Enhanced packet types selon la doc DJ-4LED
const CONNECT: u8 = 0x01;
const DISCONNECT: u8 = 0x02;
const PING: u8 = 0x03;
const PONG: u8 = 0x04;
const ACK: u8 = 0x05;
const NACK: u8 = 0x06;
const COMMAND: u8 = 0x10;
const FRAME_DATA: u8 = 0x20;
const FRAME_DATA_COMPRESSED: u8 = 0x21;
const SPECTRUM_DATA: u8 = 0x30;

// Command IDs
const SET_EFFECT: u8 = 0x01;
const SET_COLOR_MODE: u8 = 0x02;
const SET_CUSTOM_COLOR: u8 = 0x03;

// Enhanced server configuration
const SERVER_ADDRESS: &str = "127.0.0.1:8081";
const SOCKET_TIMEOUT_SECS: u64 = 1;
const MAX_PACKET_SIZE: usize = 4096;
const STREAM_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const MAX_STREAM_DURATION: Duration = Duration::from_secs(120); // 2 minutes

// Enhanced global state
type ConnectionState = Arc<Mutex<Option<UdpSocket>>>;
type StreamState = Arc<Mutex<StreamContext>>;

#[derive(Debug, Clone)]
struct StreamContext {
    is_active: bool,
    start_time: Option<Instant>,
    packets_received: u32,
    frames_received: u32,
    spectrum_received: u32,
    bytes_received: u64,
    packets_lost: u32,
    last_sequence: u32,
}

impl Default for StreamContext {
    fn default() -> Self {
        Self {
            is_active: false,
            start_time: None,
            packets_received: 0,
            frames_received: 0,
            spectrum_received: 0,
            bytes_received: 0,
            packets_lost: 0,
            last_sequence: 0,
        }
    }
}

// Enhanced packet structure with validation
#[derive(Debug)]
struct PacketHeader {
    packet_type: u8,
    flags: u8,
    sequence: u32,
    fragment_id: u16,
    fragment_count: u16,
    payload_size: u16,
}

impl PacketHeader {
    fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 12 {
            return Err("Packet too short for header".to_string());
        }

        Ok(PacketHeader {
            packet_type: data[0],
            flags: data[1],
            sequence: u32::from_le_bytes([data[2], data[3], data[4], data[5]]),
            fragment_id: u16::from_le_bytes([data[6], data[7]]),
            fragment_count: u16::from_le_bytes([data[8], data[9]]),
            payload_size: u16::from_le_bytes([data[10], data[11]]),
        })
    }

    fn validate(&self, packet_len: usize) -> Result<(), String> {
        if packet_len < 12 + self.payload_size as usize {
            return Err(format!(
                "Packet length mismatch: expected {}, got {}",
                12 + self.payload_size,
                packet_len
            ));
        }
        Ok(())
    }
}

// Enhanced packet creation with better error handling
fn create_packet(packet_type: u8, flags: u8, sequence: u32, payload: Vec<u8>) -> Vec<u8> {
    let mut packet = Vec::with_capacity(12 + payload.len());
    packet.push(packet_type);
    packet.push(flags);
    packet.extend_from_slice(&sequence.to_le_bytes());
    packet.extend_from_slice(&0u16.to_le_bytes()); // fragment_id
    packet.extend_from_slice(&1u16.to_le_bytes()); // fragment_count
    packet.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    packet.extend_from_slice(&payload);
    packet
}

fn create_socket_with_timeout(timeout_secs: u64) -> Result<UdpSocket, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Socket creation error: {}", e))?;

    socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))
        .map_err(|e| format!("Timeout configuration error: {}", e))?;

    socket.set_write_timeout(Some(Duration::from_secs(timeout_secs)))
        .map_err(|e| format!("Write timeout configuration error: {}", e))?;

    Ok(socket)
}

fn get_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

// Enhanced frame data parsing with validation
fn parse_frame_data(data: &[u8]) -> Result<serde_json::Value, String> {
    if data.len() < 5 {
        return Err("Frame data too short for header".to_string());
    }

    let width = u16::from_le_bytes([data[0], data[1]]);
    let height = u16::from_le_bytes([data[2], data[3]]);
    let format = data[4];

    // Validate dimensions
    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err(format!("Invalid frame dimensions: {}x{}", width, height));
    }

    let expected_size = match format {
        1 => (width as usize) * (height as usize) * 3, // RGB
        2 => (width as usize) * (height as usize) * 4, // RGBA
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    if data.len() < 5 + expected_size {
        return Err(format!(
            "Insufficient frame data: expected {}, got {}",
            5 + expected_size,
            data.len()
        ));
    }

    let rgb_data: Vec<u8> = data[5..5 + expected_size].to_vec();

    Ok(json!({
        "width": width,
        "height": height,
        "format": format,
        "data": rgb_data,
        "timestamp": get_timestamp()
    }))
}

// Enhanced spectrum data parsing with normalization
fn parse_spectrum_data(data: &[u8]) -> Result<Vec<f32>, String> {
    if data.len() < 2 {
        return Err("Spectrum data too short for header".to_string());
    }

    let band_count = u16::from_le_bytes([data[0], data[1]]);

    // Validate band count
    if band_count == 0 || band_count > 1024 {
        return Err(format!("Invalid band count: {}", band_count));
    }

    let expected_size = 2 + (band_count as usize * 4);

    if data.len() < expected_size {
        return Err(format!(
            "Insufficient spectrum data: expected {}, got {}",
            expected_size,
            data.len()
        ));
    }

    let mut spectrum_values = Vec::with_capacity(band_count as usize);
    for i in 0..band_count {
        let offset = 2 + (i as usize * 4);
        let value = f32::from_le_bytes([
            data[offset], data[offset + 1],
            data[offset + 2], data[offset + 3]
        ]);

        // Clamp and normalize values
        let normalized_value = value.clamp(0.0, 1.0);
        spectrum_values.push(normalized_value);
    }

    Ok(spectrum_values)
}

// Enhanced connection commands
#[tauri::command]
async fn dj_connect(connection: State<'_, ConnectionState>) -> Result<String, String> {
    println!("🔌 dj_connect: Initiating connection...");

    let socket = create_socket_with_timeout(3)?;
    let connect_packet = create_packet(CONNECT, 0x00, get_timestamp(), vec![]);

    socket.send_to(&connect_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Connection failed: {}", e))?;

    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            if len >= 1 && buf[0] == ACK {
                if let Ok(mut conn) = connection.lock() {
                    *conn = Some(socket);
                }
                println!("✅ dj_connect: Connected successfully to {}", addr);
                Ok(format!("✅ Connected to DJ-4LED server ({})", addr))
            } else if len >= 1 && buf[0] == NACK {
                println!("❌ dj_connect: Server rejected connection");
                Err("Server rejected connection".to_string())
            } else {
                println!("⚠️ dj_connect: Unexpected response: {:#04x}", buf[0]);
                Ok(format!("⚠️ Unexpected response: type {:#04x}", buf[0]))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                println!("⏰ dj_connect: Connection timeout");
                Ok("⏰ Timeout - DJ-4LED server offline".to_string())
            } else {
                println!("❌ dj_connect: Reception error: {}", e);
                Err(format!("Reception error: {}", e))
            }
        }
    }
}

#[tauri::command]
async fn dj_disconnect(
    connection: State<'_, ConnectionState>,
    stream_state: State<'_, StreamState>
) -> Result<String, String> {
    println!("🔌 dj_disconnect: Initiating disconnection...");

    // Stop streaming first
    if let Ok(mut stream_ctx) = stream_state.lock() {
        stream_ctx.is_active = false;
        println!("🛑 dj_disconnect: Stream stopped");
    }

    let socket = create_socket_with_timeout(2)?;
    let disconnect_packet = create_packet(DISCONNECT, 0x00, get_timestamp(), vec![]);

    socket.send_to(&disconnect_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Disconnection failed: {}", e))?;

    if let Ok(mut conn) = connection.lock() {
        *conn = None;
    }

    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, _)) => {
            if len >= 1 && buf[0] == ACK {
                println!("✅ dj_disconnect: Clean disconnection confirmed");
                Ok("✅ Cleanly disconnected from DJ-4LED server".to_string())
            } else {
                println!("✅ dj_disconnect: Disconnection sent (no confirmation)");
                Ok("✅ Disconnection sent (no confirmation)".to_string())
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                println!("✅ dj_disconnect: Disconnection sent (timeout on confirmation)");
                Ok("✅ Disconnection sent (timeout on confirmation)".to_string())
            } else {
                println!("❌ dj_disconnect: Disconnection error: {}", e);
                Err(format!("Disconnection error: {}", e))
            }
        }
    }
}

#[tauri::command]
async fn dj_ping() -> Result<String, String> {
    println!("🏓 dj_ping: Sending ping...");

    let socket = create_socket_with_timeout(3)?;
    let ping_start = Instant::now();
    let ping_packet = create_packet(PING, 0x00, get_timestamp(), vec![]);

    socket.send_to(&ping_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Ping failed: {}", e))?;

    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            let ping_duration = ping_start.elapsed();
            let ping_ms = ping_duration.as_millis();

            if len >= 1 && buf[0] == PONG {
                println!("🏓 dj_ping: PONG received in {}ms", ping_ms);
                Ok(format!("🏓 PONG received from {} ({}ms)", addr, ping_ms))
            } else {
                println!("⚠️ dj_ping: Unexpected ping response: {:#04x}", buf[0]);
                Ok(format!("⚠️ Unexpected ping response: type {:#04x}", buf[0]))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                println!("⏰ dj_ping: Ping timeout");
                Ok("⏰ Timeout - server doesn't respond to ping".to_string())
            } else {
                println!("❌ dj_ping: Ping error: {}", e);
                Err(format!("Ping error: {}", e))
            }
        }
    }
}

// Enhanced command functions
#[tauri::command]
async fn dj_set_effect(effect_id: u32) -> Result<String, String> {
    println!("🎇 dj_set_effect: Setting effect {}", effect_id);

    let socket = create_socket_with_timeout(2)?;
    let mut payload = vec![SET_EFFECT];
    payload.extend_from_slice(&effect_id.to_le_bytes());
    let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

    socket.send_to(&packet, SERVER_ADDRESS)
        .map_err(|e| format!("Effect command failed: {}", e))?;

    println!("✅ dj_set_effect: Effect {} applied", effect_id);
    Ok(format!("✅ Effect {} applied", effect_id))
}

#[tauri::command]
async fn dj_set_color_mode(mode: String) -> Result<String, String> {
    println!("🌈 dj_set_color_mode: Setting mode '{}'", mode);

    let socket = create_socket_with_timeout(2)?;
    let mut payload = vec![SET_COLOR_MODE];
    payload.extend_from_slice(mode.as_bytes());
    let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

    socket.send_to(&packet, SERVER_ADDRESS)
        .map_err(|e| format!("Color mode command failed: {}", e))?;

    println!("✅ dj_set_color_mode: Mode '{}' applied", mode);
    Ok(format!("✅ Color mode '{}' applied", mode))
}

#[tauri::command]
async fn dj_set_custom_color(r: f32, g: f32, b: f32) -> Result<String, String> {
    println!("🎨 dj_set_custom_color: Setting RGB({:.3}, {:.3}, {:.3})", r, g, b);

    let socket = create_socket_with_timeout(2)?;
    let mut payload = vec![SET_CUSTOM_COLOR];
    payload.extend_from_slice(&r.to_le_bytes());
    payload.extend_from_slice(&g.to_le_bytes());
    payload.extend_from_slice(&b.to_le_bytes());
    let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

    socket.send_to(&packet, SERVER_ADDRESS)
        .map_err(|e| format!("Custom color command failed: {}", e))?;

    println!("✅ dj_set_custom_color: Color applied");
    Ok(format!("✅ Color RGB({:.3}, {:.3}, {:.3}) applied", r, g, b))
}

// Enhanced streaming with better error handling and monitoring
#[tauri::command]
async fn dj_start_stream(
    window: Window,
    stream_state: State<'_, StreamState>
) -> Result<String, String> {
    println!("🚀 dj_start_stream: Starting enhanced stream...");

    // Check if already streaming
    if let Ok(stream_ctx) = stream_state.lock() {
        if stream_ctx.is_active {
            println!("⚠️ dj_start_stream: Stream already active");
            return Ok("📡 Stream already active".to_string());
        }
    }

    println!("🔌 dj_start_stream: Creating socket...");
    let socket = create_socket_with_timeout(SOCKET_TIMEOUT_SECS)?;

    // Enhanced connect packet with compression support
    println!("📡 dj_start_stream: Sending connect packet to {}", SERVER_ADDRESS);
    let connect_packet = create_packet(CONNECT, 0x01, get_timestamp(), vec![]);
    socket.send_to(&connect_packet, SERVER_ADDRESS)
        .map_err(|e| {
            println!("❌ dj_start_stream: Connection failed: {}", e);
            format!("Stream connection failed: {}", e)
        })?;

    // Wait for ACK with timeout
    println!("⏳ dj_start_stream: Waiting for ACK...");
    let mut buf = [0; MAX_PACKET_SIZE];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            println!("📥 dj_start_stream: Received {} bytes from {}", len, addr);
            if len < 1 || buf[0] != ACK {
                println!("❌ dj_start_stream: No ACK received, got packet type: {:#04x}", buf[0]);
                return Err("No ACK received for stream connection".to_string());
            }
            println!("✅ dj_start_stream: ACK received successfully");
        }
        Err(e) => {
            println!("❌ dj_start_stream: Timeout waiting for ACK: {}", e);
            return Err("Timeout waiting for stream ACK".to_string());
        }
    }

    // Initialize stream context
    if let Ok(mut stream_ctx) = stream_state.lock() {
        *stream_ctx = StreamContext {
            is_active: true,
            start_time: Some(Instant::now()),
            ..Default::default()
        };
        println!("🎯 dj_start_stream: Stream context initialized");
    }

    let stream_state_clone = stream_state.inner().clone();
    let window_clone = window.clone();

    println!("🧵 dj_start_stream: Starting enhanced streaming thread...");

    // Enhanced streaming thread with better monitoring
    thread::spawn(move || {
        println!("🔄 Stream thread: Starting enhanced main loop...");
        let mut last_health_check = Instant::now();
        let mut last_stats_report = Instant::now();

        // Déclarer stream_ctx en dehors de la boucle pour qu'elle soit accessible après
        let mut stream_ctx = StreamContext::default();

        loop {
            // Check if we should continue streaming et récupérer stream_ctx
            let should_continue;

            {
                if let Ok(ctx) = stream_state_clone.lock() {
                    should_continue = ctx.is_active;
                    stream_ctx = ctx.clone(); // Mettre à jour notre variable locale
                } else {
                    should_continue = false;
                    stream_ctx = StreamContext::default();
                }
            }

            if !should_continue {
                println!("🛑 Stream thread: Stopping loop (should_continue = false)");
                break;
            }

            // Auto-stop after maximum duration
            if let Some(start_time) = stream_ctx.start_time {
                if start_time.elapsed() > MAX_STREAM_DURATION {
                    println!("⏰ Stream thread: Auto-stopping after maximum duration");
                    if let Ok(mut ctx) = stream_state_clone.lock() {
                        ctx.is_active = false;
                    }
                    let _ = window_clone.emit("stream_status", json!({
                        "status": "auto_stopped",
                        "message": "Stream auto-stopped after maximum duration",
                        "stats": {
                            "packets": stream_ctx.packets_received,
                            "frames": stream_ctx.frames_received,
                            "spectrum": stream_ctx.spectrum_received,
                            "bytes": stream_ctx.bytes_received,
                            "lost": stream_ctx.packets_lost,
                            "duration": start_time.elapsed().as_secs()
                        }
                    }));
                    break;
                }
            }

            // Receive data with enhanced error handling
            match socket.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    stream_ctx.packets_received += 1;
                    stream_ctx.bytes_received += len as u64;

                    // Parse packet header
                    match PacketHeader::parse(&buf[..len]) {
                        Ok(header) => {
                            if let Err(e) = header.validate(len) {
                                println!("⚠️ Stream thread: Invalid packet: {}", e);
                                stream_ctx.packets_lost += 1;
                                continue;
                            }

                            // Check for sequence gaps (simple packet loss detection)
                            if stream_ctx.last_sequence > 0 && header.sequence > stream_ctx.last_sequence + 1 {
                                let lost_packets = header.sequence - stream_ctx.last_sequence - 1;
                                stream_ctx.packets_lost += lost_packets;
                                println!("⚠️ Stream thread: Detected {} lost packets (gap in sequence)", lost_packets);
                            }
                            stream_ctx.last_sequence = header.sequence;

                            let payload = &buf[12..12 + header.payload_size as usize];

                            match header.packet_type {
                                FRAME_DATA => {
                                    stream_ctx.frames_received += 1;
                                    if stream_ctx.frames_received % 30 == 0 { // Log every 30th frame
                                        println!("🖼️ Stream thread: Processing FRAME_DATA #{}", stream_ctx.frames_received);
                                    }
                                    match parse_frame_data(payload) {
                                        Ok(frame_data) => {
                                            if let Err(e) = window_clone.emit("frame_data", frame_data) {
                                                println!("❌ Stream thread: Failed to emit frame_data: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            println!("❌ Stream thread: Error parsing frame data: {}", e);
                                            stream_ctx.packets_lost += 1;
                                        }
                                    }
                                }
                                FRAME_DATA_COMPRESSED => {
                                    stream_ctx.frames_received += 1;
                                    if stream_ctx.frames_received % 30 == 0 {
                                        println!("🗜️ Stream thread: Processing FRAME_DATA_COMPRESSED #{}", stream_ctx.frames_received);
                                    }
                                    let compressed_data: Vec<u8> = payload.to_vec();
                                    if let Err(e) = window_clone.emit("frame_data_compressed", compressed_data) {
                                        println!("❌ Stream thread: Failed to emit frame_data_compressed: {}", e);
                                    }
                                }
                                SPECTRUM_DATA => {
                                    stream_ctx.spectrum_received += 1;
                                    if stream_ctx.spectrum_received % 50 == 0 { // Log every 50th spectrum
                                        println!("🎵 Stream thread: Processing SPECTRUM_DATA #{}", stream_ctx.spectrum_received);
                                    }
                                    match parse_spectrum_data(payload) {
                                        Ok(spectrum_values) => {
                                            if let Err(e) = window_clone.emit("spectrum_data", spectrum_values) {
                                                println!("❌ Stream thread: Failed to emit spectrum_data: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            println!("❌ Stream thread: Error parsing spectrum data: {}", e);
                                            stream_ctx.packets_lost += 1;
                                        }
                                    }
                                }
                                _ => {
                                    if stream_ctx.packets_received % 100 == 0 { // Log unknown packets occasionally
                                        println!("❓ Stream thread: Unknown packet type: {:#04x}", header.packet_type);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Stream thread: Error parsing packet header: {}", e);
                            stream_ctx.packets_lost += 1;
                        }
                    }

                    // Update stream context
                    if let Ok(mut ctx) = stream_state_clone.lock() {
                        *ctx = stream_ctx.clone();
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        // Timeout is normal, continue listening
                        continue;
                    } else {
                        println!("❌ Stream thread: Receive error: {}", e);
                        // Increment error counter but don't break immediately
                        stream_ctx.packets_lost += 1;
                        if let Ok(mut ctx) = stream_state_clone.lock() {
                            *ctx = stream_ctx.clone();
                        }

                        // Break only on persistent errors
                        if stream_ctx.packets_lost > 100 {
                            println!("❌ Stream thread: Too many errors, stopping stream");
                            let _ = window_clone.emit("stream_status", json!({
                                "status": "error",
                                "message": "Stream stopped due to persistent errors",
                                "error": format!("Receive error: {}", e)
                            }));
                            break;
                        }
                        continue;
                    }
                }
            }

            // Periodic health check and stats reporting
            let now = Instant::now();
            if now.duration_since(last_health_check) > STREAM_HEALTH_CHECK_INTERVAL {
                last_health_check = now;

                // Calculate health metrics
                let packet_loss_rate = if stream_ctx.packets_received > 0 {
                    (stream_ctx.packets_lost as f32 / (stream_ctx.packets_received + stream_ctx.packets_lost) as f32) * 100.0
                } else {
                    0.0
                };

                println!("📊 Stream health: {} packets, {} frames, {} spectrum, {:.1}% loss",
                    stream_ctx.packets_received,
                    stream_ctx.frames_received,
                    stream_ctx.spectrum_received,
                    packet_loss_rate
                );

                // Emit health status if loss rate is concerning
                if packet_loss_rate > 10.0 {
                    let _ = window_clone.emit("stream_status", json!({
                        "status": "warning",
                        "message": format!("High packet loss detected: {:.1}%", packet_loss_rate),
                        "stats": {
                            "packets": stream_ctx.packets_received,
                            "frames": stream_ctx.frames_received,
                            "spectrum": stream_ctx.spectrum_received,
                            "bytes": stream_ctx.bytes_received,
                            "lost": stream_ctx.packets_lost,
                            "loss_rate": packet_loss_rate
                        }
                    }));
                }
            }

            // Periodic stats reporting
            if now.duration_since(last_stats_report) > Duration::from_secs(10) {
                last_stats_report = now;
                let _ = window_clone.emit("stream_status", json!({
                    "status": "running",
                    "message": "Stream active",
                    "stats": {
                        "packets": stream_ctx.packets_received,
                        "frames": stream_ctx.frames_received,
                        "spectrum": stream_ctx.spectrum_received,
                        "bytes": stream_ctx.bytes_received,
                        "lost": stream_ctx.packets_lost,
                        "duration": stream_ctx.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
                    }
                }));
            }
        }

        // Maintenant stream_ctx est accessible ici car elle est déclarée en dehors de la boucle
        // Emit final stats with enhanced information
        let final_duration = stream_ctx.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        let final_loss_rate = if stream_ctx.packets_received > 0 {
            (stream_ctx.packets_lost as f32 / (stream_ctx.packets_received + stream_ctx.packets_lost) as f32) * 100.0
        } else {
            0.0
        };

        println!("📊 Stream thread: Final stats - packets: {}, frames: {}, spectrum: {}, bytes: {}, lost: {} ({:.1}%), duration: {}s",
                stream_ctx.packets_received,
                stream_ctx.frames_received,
                stream_ctx.spectrum_received,
                stream_ctx.bytes_received,
                stream_ctx.packets_lost,
                final_loss_rate,
                final_duration
        );

        let _ = window_clone.emit("stream_status", json!({
            "status": "stopped",
            "message": "Stream stopped",
            "stats": {
                "packets": stream_ctx.packets_received,
                "frames": stream_ctx.frames_received,
                "spectrum": stream_ctx.spectrum_received,
                "bytes": stream_ctx.bytes_received,
                "lost": stream_ctx.packets_lost,
                "loss_rate": final_loss_rate,
                "duration": final_duration,
                "avg_fps": if final_duration > 0 { stream_ctx.frames_received as f32 / final_duration as f32 } else { 0.0 },
                "data_rate_kbps": if final_duration > 0 { (stream_ctx.bytes_received as f32 / final_duration as f32) / 1024.0 } else { 0.0 }
            }
        }));

        println!("🏁 Stream thread: Enhanced thread ended");
    });

    println!("✅ dj_start_stream: Enhanced command completed successfully");
    Ok("📡 Enhanced stream started - listening for LED data and audio spectrum".to_string())
}

#[tauri::command]
async fn dj_stop_stream(stream_state: State<'_, StreamState>) -> Result<String, String> {
    println!("🛑 dj_stop_stream: Stopping stream...");

    if let Ok(mut stream_ctx) = stream_state.lock() {
        if !stream_ctx.is_active {
            return Ok("📡 Stream was not active".to_string());
        }

        stream_ctx.is_active = false;
        let duration = stream_ctx.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);

        println!("✅ dj_stop_stream: Stream stopped after {}s", duration);
        Ok(format!("📡 Stream stopped (ran for {}s)", duration))
    } else {
        Err("Failed to access stream state".to_string())
    }
}

#[tauri::command]
async fn dj_get_server_info() -> Result<String, String> {
    Ok(format!("🖥️ DJ-4LED Server: {} (Enhanced Protocol)", SERVER_ADDRESS))
}

#[tauri::command]
async fn dj_get_stream_stats(stream_state: State<'_, StreamState>) -> Result<serde_json::Value, String> {
    if let Ok(stream_ctx) = stream_state.lock() {
        let duration = stream_ctx.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        let loss_rate = if stream_ctx.packets_received > 0 {
            (stream_ctx.packets_lost as f32 / (stream_ctx.packets_received + stream_ctx.packets_lost) as f32) * 100.0
        } else {
            0.0
        };

        Ok(json!({
            "is_active": stream_ctx.is_active,
            "packets_received": stream_ctx.packets_received,
            "frames_received": stream_ctx.frames_received,
            "spectrum_received": stream_ctx.spectrum_received,
            "bytes_received": stream_ctx.bytes_received,
            "packets_lost": stream_ctx.packets_lost,
            "loss_rate": loss_rate,
            "duration": duration,
            "avg_fps": if duration > 0 { stream_ctx.frames_received as f32 / duration as f32 } else { 0.0 },
            "data_rate_kbps": if duration > 0 { (stream_ctx.bytes_received as f32 / duration as f32) / 1024.0 } else { 0.0 }
        }))
    } else {
        Err("Failed to access stream statistics".to_string())
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust and enhanced DJ-4LED!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("🚀 Starting enhanced DJ-4LED application...");

    let connection_state: ConnectionState = Arc::new(Mutex::new(None));
    let stream_state: StreamState = Arc::new(Mutex::new(StreamContext::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(connection_state)
        .manage(stream_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            dj_connect,
            dj_disconnect,
            dj_ping,
            dj_set_effect,
            dj_set_color_mode,
            dj_set_custom_color,
            dj_start_stream,
            dj_stop_stream,
            dj_get_server_info,
            dj_get_stream_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running enhanced tauri application");
}
