// src-tauri/src/lib.rs
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{State, Window, Emitter};
use serde_json::json;

// Packet types selon la doc DJ-4LED
const CONNECT: u8 = 0x01;
const DISCONNECT: u8 = 0x02;
const PING: u8 = 0x03;
const PONG: u8 = 0x04;
const ACK: u8 = 0x05;
const COMMAND: u8 = 0x10;
const FRAME_DATA: u8 = 0x20;
const FRAME_DATA_COMPRESSED: u8 = 0x21;
const SPECTRUM_DATA: u8 = 0x30;

// Command IDs
const SET_EFFECT: u8 = 0x01;
const SET_COLOR_MODE: u8 = 0x02;
const SET_CUSTOM_COLOR: u8 = 0x03;

// Server configuration
const SERVER_ADDRESS: &str = "127.0.0.1:8081";

// Global connection state
type ConnectionState = Arc<Mutex<Option<UdpSocket>>>;
type StreamState = Arc<Mutex<bool>>;

// Structure de paquet UDP (12 bytes header + payload)
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

    Ok(socket)
}

fn get_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

// Parse frame data from UDP packet
fn parse_frame_data(data: &[u8]) -> Result<serde_json::Value, String> {
    if data.len() < 5 {
        return Err("Frame data too short".to_string());
    }

    let width = u16::from_le_bytes([data[0], data[1]]);
    let height = u16::from_le_bytes([data[2], data[3]]);
    let format = data[4];

    let expected_size = match format {
        1 => (width as usize) * (height as usize) * 3, // RGB
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    if data.len() < 5 + expected_size {
        return Err("Insufficient frame data".to_string());
    }

    let rgb_data: Vec<u8> = data[5..5 + expected_size].to_vec();

    Ok(json!({
        "width": width,
        "height": height,
        "format": format,
        "data": rgb_data
    }))
}

// Parse spectrum data from UDP packet
fn parse_spectrum_data(data: &[u8]) -> Result<Vec<f32>, String> {
    if data.len() < 2 {
        return Err("Spectrum data too short".to_string());
    }

    let band_count = u16::from_le_bytes([data[0], data[1]]);
    let expected_size = 2 + (band_count as usize * 4);

    if data.len() < expected_size {
        return Err("Insufficient spectrum data".to_string());
    }

    let mut spectrum_values = Vec::with_capacity(band_count as usize);
    for i in 0..band_count {
        let offset = 2 + (i as usize * 4);
        let value = f32::from_le_bytes([
            data[offset], data[offset + 1],
            data[offset + 2], data[offset + 3]
        ]);
        spectrum_values.push(value.clamp(0.0, 1.0)); // Ensure values are normalized
    }

    Ok(spectrum_values)
}

#[tauri::command]
async fn dj_connect(connection: State<'_, ConnectionState>) -> Result<String, String> {
    let socket = create_socket_with_timeout(3)?;

    // Packet Connect selon la doc
    let connect_packet = create_packet(CONNECT, 0x00, 0, vec![]);

    socket.send_to(&connect_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Connection failed: {}", e))?;

    // Attendre ACK
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            if len >= 1 && buf[0] == ACK {
                if let Ok(mut conn) = connection.lock() {
                    *conn = Some(socket);
                }
                Ok(format!("✅ Connected to DJ-4LED server ({})", addr))
            } else {
                Ok(format!("⚠️ Unexpected response: type {:#04x}", buf[0]))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok("⏰ Timeout - DJ-4LED server offline".to_string())
            } else {
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
    // Stop streaming first
    if let Ok(mut streaming) = stream_state.lock() {
        *streaming = false;
    }

    let socket = create_socket_with_timeout(2)?;

    // Packet Disconnect selon la doc
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
                Ok("✅ Cleanly disconnected from DJ-4LED server".to_string())
            } else {
                Ok("✅ Disconnection sent (no confirmation)".to_string())
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok("✅ Disconnection sent (timeout on confirmation)".to_string())
            } else {
                Err(format!("Disconnection error: {}", e))
            }
        }
    }
}

#[tauri::command]
async fn dj_ping() -> Result<String, String> {
    let socket = create_socket_with_timeout(3)?;

    let ping_packet = create_packet(PING, 0x00, get_timestamp(), vec![]);

    socket.send_to(&ping_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Ping failed: {}", e))?;

    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            if len >= 1 && buf[0] == PONG {
                Ok(format!("🏓 PONG received from {}", addr))
            } else {
                Ok(format!("⚠️ Unexpected ping response: type {:#04x}", buf[0]))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok("⏰ Timeout - server doesn't respond to ping".to_string())
            } else {
                Err(format!("Ping error: {}", e))
            }
        }
    }
}

#[tauri::command]
async fn dj_set_effect(effect_id: u32) -> Result<String, String> {
    let socket = create_socket_with_timeout(2)?;

    let mut payload = vec![SET_EFFECT];
    payload.extend_from_slice(&effect_id.to_le_bytes());

    let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

    socket.send_to(&packet, SERVER_ADDRESS)
        .map_err(|e| format!("Effect command failed: {}", e))?;

    Ok(format!("✅ Effect {} applied", effect_id))
}

#[tauri::command]
async fn dj_set_color_mode(mode: String) -> Result<String, String> {
    let socket = create_socket_with_timeout(2)?;

    let mut payload = vec![SET_COLOR_MODE];
    payload.extend_from_slice(mode.as_bytes());

    let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

    socket.send_to(&packet, SERVER_ADDRESS)
        .map_err(|e| format!("Color mode command failed: {}", e))?;

    Ok(format!("✅ Color mode '{}' applied", mode))
}

#[tauri::command]
async fn dj_set_custom_color(r: f32, g: f32, b: f32) -> Result<String, String> {
    let socket = create_socket_with_timeout(2)?;

    let mut payload = vec![SET_CUSTOM_COLOR];
    payload.extend_from_slice(&r.to_le_bytes());
    payload.extend_from_slice(&g.to_le_bytes());
    payload.extend_from_slice(&b.to_le_bytes());

    let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

    socket.send_to(&packet, SERVER_ADDRESS)
        .map_err(|e| format!("Custom color command failed: {}", e))?;

    Ok(format!("✅ Color RGB({:.3}, {:.3}, {:.3}) applied", r, g, b))
}

#[tauri::command]
async fn dj_start_stream(
    window: Window,
    stream_state: State<'_, StreamState>
) -> Result<String, String> {
    println!("🚀 dj_start_stream: Starting stream command...");

    // Check if already streaming
    if let Ok(streaming) = stream_state.lock() {
        if *streaming {
            println!("⚠️ dj_start_stream: Stream already active");
            return Ok("📡 Stream already active".to_string());
        }
    }

    println!("🔌 dj_start_stream: Creating socket...");
    let socket = create_socket_with_timeout(1)?;

    // Send connect packet with compression support
    println!("📡 dj_start_stream: Sending connect packet to {}", SERVER_ADDRESS);
    let connect_packet = create_packet(CONNECT, 0x01, get_timestamp(), vec![]);
    socket.send_to(&connect_packet, SERVER_ADDRESS)
        .map_err(|e| {
            println!("❌ dj_start_stream: Connection failed: {}", e);
            format!("Stream connection failed: {}", e)
        })?;

    // Wait for ACK
    println!("⏳ dj_start_stream: Waiting for ACK...");
    let mut buf = [0; 2048];
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

    // Set streaming state
    if let Ok(mut streaming) = stream_state.lock() {
        *streaming = true;
        println!("🎯 dj_start_stream: Streaming state set to true");
    }

    let stream_state_clone = stream_state.inner().clone();
    let window_clone = window.clone();

    println!("🧵 dj_start_stream: Starting streaming thread...");

    // Start streaming thread
    thread::spawn(move || {
        println!("🔄 Stream thread: Starting main loop...");
        let mut packets_received = 0;
        let mut frames_received = 0;
        let mut spectrum_received = 0;
        let start_time = std::time::Instant::now();

        loop {
            // Check if we should continue streaming
            let should_continue = {
                if let Ok(streaming) = stream_state_clone.lock() {
                    *streaming
                } else {
                    false
                }
            };

            if !should_continue {
                println!("🛑 Stream thread: Stopping loop (should_continue = false)");
                break;
            }

            match socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    // Uncomment this line for very verbose debugging (will spam logs)
                    // println!("📥 Stream thread: Received {} bytes from {}", len, addr);

                    if len >= 12 {
                        packets_received += 1;
                        let packet_type = buf[0];
                        let _flags = buf[1];
                        let _sequence = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]);
                        let payload_size = u16::from_le_bytes([buf[10], buf[11]]) as usize;

                        if len >= 12 + payload_size {
                            let payload = &buf[12..12 + payload_size];

                            match packet_type {
                                FRAME_DATA => {
                                    frames_received += 1;
                                    println!("🖼️ Stream thread: Processing FRAME_DATA ({})", frames_received);
                                    match parse_frame_data(payload) {
                                        Ok(frame_data) => {
                                            println!("✅ Stream thread: Parsed frame data, emitting event...");
                                            if let Err(e) = window_clone.emit("frame_data", frame_data) {
                                                println!("❌ Stream thread: Failed to emit frame_data: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            println!("❌ Stream thread: Error parsing frame data: {}", e);
                                        }
                                    }
                                }
                                FRAME_DATA_COMPRESSED => {
                                    frames_received += 1;
                                    println!("🗜️ Stream thread: Processing FRAME_DATA_COMPRESSED ({})", frames_received);
                                    let compressed_data: Vec<u8> = payload.to_vec();
                                    if let Err(e) = window_clone.emit("frame_data_compressed", compressed_data) {
                                        println!("❌ Stream thread: Failed to emit frame_data_compressed: {}", e);
                                    }
                                }
                                SPECTRUM_DATA => {
                                    spectrum_received += 1;
                                    if spectrum_received % 10 == 0 { // Log every 10th spectrum packet
                                        println!("🎵 Stream thread: Processing SPECTRUM_DATA ({})", spectrum_received);
                                    }
                                    match parse_spectrum_data(payload) {
                                        Ok(spectrum_values) => {
                                            if let Err(e) = window_clone.emit("spectrum_data", spectrum_values) {
                                                println!("❌ Stream thread: Failed to emit spectrum_data: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            println!("❌ Stream thread: Error parsing spectrum data: {}", e);
                                        }
                                    }
                                }
                                _ => {
                                    // Log unknown packet types for debugging
                                    println!("❓ Stream thread: Unknown packet type: {:#04x}", packet_type);
                                }
                            }
                        } else {
                            println!("⚠️ Stream thread: Packet too short for payload (len={}, expected={})", len, 12 + payload_size);
                        }
                    } else {
                        println!("⚠️ Stream thread: Packet too short for header (len={})", len);
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        // Timeout is normal, continue listening
                        continue;
                    } else {
                        println!("❌ Stream thread: Receive error: {}", e);
                        break;
                    }
                }
            }

            // Auto-stop after 60 seconds of streaming
            if start_time.elapsed() > Duration::from_secs(60) {
                println!("⏰ Stream thread: Auto-stopping after 60 seconds");
                if let Ok(mut streaming) = stream_state_clone.lock() {
                    *streaming = false;
                }
                let _ = window_clone.emit("stream_status", json!({
                    "status": "auto_stopped",
                    "message": "Stream auto-stopped after 60 seconds",
                    "stats": {
                        "packets": packets_received,
                        "frames": frames_received,
                        "spectrum": spectrum_received
                    }
                }));
                break;
            }
        }

        // Emit final stats
        println!("📊 Stream thread: Final stats - packets: {}, frames: {}, spectrum: {}",
                packets_received, frames_received, spectrum_received);
        let _ = window_clone.emit("stream_status", json!({
            "status": "stopped",
            "message": "Stream stopped",
            "stats": {
                "packets": packets_received,
                "frames": frames_received,
                "spectrum": spectrum_received,
                "duration": start_time.elapsed().as_secs()
            }
        }));

        println!("🏁 Stream thread: Thread ended");
    });

    println!("✅ dj_start_stream: Command completed successfully");
    Ok("📡 Stream started - listening for LED data and audio spectrum".to_string())
}

#[tauri::command]
async fn dj_stop_stream(stream_state: State<'_, StreamState>) -> Result<String, String> {
    if let Ok(mut streaming) = stream_state.lock() {
        *streaming = false;
        Ok("📡 Stream stopped".to_string())
    } else {
        Err("Failed to stop stream".to_string())
    }
}

#[tauri::command]
async fn dj_get_server_info() -> Result<String, String> {
    Ok(format!("🖥️ DJ-4LED Server: {}", SERVER_ADDRESS))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let connection_state: ConnectionState = Arc::new(Mutex::new(None));
    let stream_state: StreamState = Arc::new(Mutex::new(false));

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
            dj_get_server_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
