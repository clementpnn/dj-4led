// src-tauri/src/lib.rs
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
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

// Default server configuration
const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:8081";

// Global connection state
type ConnectionState = Arc<Mutex<Option<UdpSocket>>>;

// Retry configuration
#[derive(Clone)]
struct RetryConfig {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

// Enhanced error types
#[derive(Debug, Clone)]
enum NetworkError {
    Unreachable(String),      // OS error 51, 65
    ResourceUnavailable,      // OS error 35
    Timeout,                  // Connection timeout
    PermissionDenied,         // OS error 13
    ConnectionRefused,        // OS error 61
    Other(String),
}

impl NetworkError {
    fn from_io_error(error: &std::io::Error) -> Self {
        match error.raw_os_error() {
            Some(51) | Some(65) => NetworkError::Unreachable(error.to_string()),
            Some(35) => NetworkError::ResourceUnavailable,
            Some(13) => NetworkError::PermissionDenied,
            Some(61) => NetworkError::ConnectionRefused,
            _ => match error.kind() {
                std::io::ErrorKind::TimedOut => NetworkError::Timeout,
                _ => NetworkError::Other(error.to_string()),
            }
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            NetworkError::ResourceUnavailable | NetworkError::Timeout => true,
            NetworkError::Unreachable(_) => true,  // Might be temporary
            NetworkError::ConnectionRefused => true,  // Device might be booting
            NetworkError::PermissionDenied => false,  // Won't change with retry
            NetworkError::Other(_) => false,
        }
    }

    fn to_user_message(&self) -> String {
        match self {
            NetworkError::Unreachable(msg) => {
                format!("❌ Réseau inaccessible: {}. Vérifiez l'adresse IP et la connectivité réseau.", msg)
            }
            NetworkError::ResourceUnavailable => {
                "⏳ Ressource temporairement indisponible. Nouvelle tentative...".to_string()
            }
            NetworkError::Timeout => {
                "⏰ Timeout de connexion. L'appareil ne répond pas.".to_string()
            }
            NetworkError::PermissionDenied => {
                "🚫 Permission refusée. Vérifiez les permissions réseau.".to_string()
            }
            NetworkError::ConnectionRefused => {
                "🔒 Connexion refusée. L'appareil est peut-être éteint.".to_string()
            }
            NetworkError::Other(msg) => {
                format!("❌ Erreur réseau: {}", msg)
            }
        }
    }
}

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

fn create_socket_with_timeout(timeout_secs: u64) -> Result<UdpSocket, NetworkError> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| NetworkError::from_io_error(&e))?;

    socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))
        .map_err(|e| NetworkError::from_io_error(&e))?;

    socket.set_write_timeout(Some(Duration::from_secs(timeout_secs)))
        .map_err(|e| NetworkError::from_io_error(&e))?;

    Ok(socket)
}

async fn retry_with_backoff<T, F, Fut>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, NetworkError>>,
{
    let mut attempt = 1;
    let mut delay = config.base_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                println!(
                    "🔄 {} - Tentative {}/{} échouée: {}",
                    operation_name, attempt, config.max_attempts, error.to_user_message()
                );

                if attempt >= config.max_attempts || !error.is_retryable() {
                    return Err(error.to_user_message());
                }

                println!("⏳ Attente de {:?} avant nouvelle tentative...", delay);
                tokio::time::sleep(delay).await;

                // Exponential backoff
                delay = std::cmp::min(
                    Duration::from_millis((delay.as_millis() as f64 * config.backoff_multiplier) as u64),
                    config.max_delay,
                );
                attempt += 1;
            }
        }
    }
}

fn get_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

#[tauri::command]
async fn dj_connect(server_address: Option<String>, connection: State<'_, ConnectionState>) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let config = RetryConfig::default();

    let connection_clone = connection.inner().clone();
    
    let result = retry_with_backoff(
        move || {
            let address = address.clone();
            let connection = connection_clone.clone();
            async move {
                let socket = create_socket_with_timeout(3)?;
                let connect_packet = create_packet(CONNECT, 0x00, 0, vec![]);

                socket.send_to(&connect_packet, &address)
                    .map_err(|e| NetworkError::from_io_error(&e))?;

                let mut buf = [0; 1024];
                match socket.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        if len >= 1 && buf[0] == ACK {
                            if let Ok(mut conn) = connection.lock() {
                                *conn = Some(socket);
                            }
                            Ok(format!("✅ Connected to DJ-4LED server {} ({})", address, addr))
                        } else {
                            Ok(format!("⚠️ Unexpected response: type {:#04x}", buf[0]))
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::TimedOut {
                            Err(NetworkError::Timeout)
                        } else {
                            Err(NetworkError::from_io_error(&e))
                        }
                    }
                }
            }
        },
        config,
        "Connexion DJ-4LED"
    ).await;

    result
}

#[tauri::command]
async fn dj_disconnect(server_address: Option<String>, connection: State<'_, ConnectionState>) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let config = RetryConfig {
        max_attempts: 2,  // Fewer retries for disconnect
        ..RetryConfig::default()
    };

    let result = retry_with_backoff(
        move || {
            let address = address.clone();
            async move {
                let socket = create_socket_with_timeout(2)?;
                let disconnect_packet = create_packet(DISCONNECT, 0x00, get_timestamp(), vec![]);

                socket.send_to(&disconnect_packet, &address)
                    .map_err(|e| NetworkError::from_io_error(&e))?;

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
                            Err(NetworkError::from_io_error(&e))
                        }
                    }
                }
            }
        },
        config,
        "Déconnexion DJ-4LED"
    ).await;

    // Clear connection state regardless of result
    if let Ok(mut conn) = connection.lock() {
        *conn = None;
    }

    result
}

#[tauri::command]
async fn dj_ping(server_address: Option<String>) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let config = RetryConfig {
        max_attempts: 2,  // Fewer retries for ping
        base_delay: Duration::from_millis(200),
        ..RetryConfig::default()
    };

    retry_with_backoff(
        move || {
            let address = address.clone();
            async move {
                let socket = create_socket_with_timeout(3)?;
                let ping_packet = create_packet(PING, 0x00, get_timestamp(), vec![]);

                socket.send_to(&ping_packet, &address)
                    .map_err(|e| NetworkError::from_io_error(&e))?;

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
                            Err(NetworkError::Timeout)
                        } else {
                            Err(NetworkError::from_io_error(&e))
                        }
                    }
                }
            }
        },
        config,
        "Ping DJ-4LED"
    ).await
}

#[tauri::command]
async fn dj_set_effect(server_address: Option<String>, effect_id: u32) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(300),
        ..RetryConfig::default()
    };

    retry_with_backoff(
        move || {
            let address = address.clone();
            async move {
                let socket = create_socket_with_timeout(2)?;
                let mut payload = vec![SET_EFFECT];
                payload.extend_from_slice(&effect_id.to_le_bytes());
                let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

                socket.send_to(&packet, &address)
                    .map_err(|e| NetworkError::from_io_error(&e))?;

                Ok(format!("✅ Effect {} applied", effect_id))
            }
        },
        config,
        "Set Effect"
    ).await
}

#[tauri::command]
async fn dj_set_color_mode(server_address: Option<String>, mode: u32) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(300),
        ..RetryConfig::default()
    };

    retry_with_backoff(
        move || {
            let address = address.clone();
            async move {
                let socket = create_socket_with_timeout(2)?;
                let mut payload = vec![SET_COLOR_MODE];
                payload.extend_from_slice(&mode.to_le_bytes());
                let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

                socket.send_to(&packet, &address)
                    .map_err(|e| NetworkError::from_io_error(&e))?;

                Ok(format!("✅ Color mode {} applied", mode))
            }
        },
        config,
        "Set Color Mode"
    ).await
}

#[tauri::command]
async fn dj_set_custom_color(server_address: Option<String>, r: f32, g: f32, b: f32) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(300),
        ..RetryConfig::default()
    };

    retry_with_backoff(
        move || {
            let address = address.clone();
            async move {
                let socket = create_socket_with_timeout(2)?;
                let mut payload = vec![SET_CUSTOM_COLOR];
                payload.extend_from_slice(&r.to_le_bytes());
                payload.extend_from_slice(&g.to_le_bytes());
                payload.extend_from_slice(&b.to_le_bytes());
                let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

                socket.send_to(&packet, &address)
                    .map_err(|e| NetworkError::from_io_error(&e))?;

                Ok(format!("✅ Custom color ({:.2}, {:.2}, {:.2}) applied", r, g, b))
            }
        },
        config,
        "Set Custom Color"
    ).await
}

#[tauri::command]
async fn dj_diagnose_network(server_address: Option<String>) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let mut diagnostics = Vec::new();

    // Parse address
    let (host, port) = if let Some(pos) = address.rfind(':') {
        let host = &address[..pos];
        let port_str = &address[pos + 1..];
        match port_str.parse::<u16>() {
            Ok(port) => (host.to_string(), port),
            Err(_) => {
                return Err("❌ Format d'adresse invalide. Utilisez 'host:port'".to_string());
            }
        }
    } else {
        return Err("❌ Format d'adresse invalide. Utilisez 'host:port'".to_string());
    };

    diagnostics.push(format!("🔍 Diagnostic réseau pour {}:{}", host, port));

    // Test 1: Basic socket creation
    match create_socket_with_timeout(1) {
        Ok(_) => diagnostics.push("✅ Création de socket UDP: OK".to_string()),
        Err(e) => {
            diagnostics.push(format!("❌ Création de socket UDP: {}", e.to_user_message()));
            return Ok(diagnostics.join("\n"));
        }
    }

    // Test 2: Quick ping test with immediate timeout
    diagnostics.push("🏓 Test de ping rapide...".to_string());
    match create_socket_with_timeout(1) {
        Ok(socket) => {
            let ping_packet = create_packet(PING, 0x00, get_timestamp(), vec![]);
            match socket.send_to(&ping_packet, &address) {
                Ok(_) => {
                    diagnostics.push("✅ Envoi du paquet: OK".to_string());
                    
                    let mut buf = [0; 1024];
                    match socket.recv_from(&mut buf) {
                        Ok((len, _)) if len >= 1 && buf[0] == PONG => {
                            diagnostics.push("✅ Réponse PONG reçue: OK".to_string());
                        }
                        Ok((len, _)) => {
                            diagnostics.push(format!("⚠️ Réponse inattendue: type {:#04x}", buf[0]));
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                            diagnostics.push("⏰ Timeout: L'appareil ne répond pas".to_string());
                        }
                        Err(e) => {
                            let net_err = NetworkError::from_io_error(&e);
                            diagnostics.push(format!("❌ Erreur de réception: {}", net_err.to_user_message()));
                        }
                    }
                }
                Err(e) => {
                    let net_err = NetworkError::from_io_error(&e);
                    diagnostics.push(format!("❌ Erreur d'envoi: {}", net_err.to_user_message()));
                }
            }
        }
        Err(e) => {
            diagnostics.push(format!("❌ Socket pour ping: {}", e.to_user_message()));
        }
    }

    // Test 3: IP reachability info
    if host.starts_with("192.168.") || host.starts_with("10.") || host.starts_with("172.") {
        diagnostics.push("🏠 Adresse IP privée détectée (réseau local)".to_string());
    } else if host == "127.0.0.1" || host == "localhost" {
        diagnostics.push("💻 Adresse locale détectée".to_string());
    } else {
        diagnostics.push("🌐 Adresse IP publique détectée".to_string());
    }

    // Test 4: Port info
    match port {
        6454 => diagnostics.push("🎨 Port ArtNet standard détecté".to_string()),
        8081 => diagnostics.push("🎛️ Port DJ-4LED standard détecté".to_string()),
        _ => diagnostics.push(format!("🔌 Port personnalisé: {}", port)),
    }

    Ok(diagnostics.join("\n"))
}

#[tauri::command]
async fn dj_listen_data(server_address: Option<String>, window: Window) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(1000),
        ..RetryConfig::default()
    };

    retry_with_backoff(
        move || {
            let address = address.clone();
            let window = window.clone();
            async move {
                let socket = create_socket_with_timeout(8)?;
                let connect_packet = create_packet(CONNECT, 0x01, 0, vec![]);
                socket.send_to(&connect_packet, &address)
                    .map_err(|e| NetworkError::from_io_error(&e))?;

                let mut buf = [0; 2048];
                let mut packets = 0;
                let mut frames = 0;
                let mut spectrum = 0;
                let mut ack_received = false;

                let start_time = std::time::Instant::now();
                let max_duration = Duration::from_secs(8);

                // Écouter pendant maximum 8 secondes
                while start_time.elapsed() < max_duration {
                    match socket.recv_from(&mut buf) {
                        Ok((len, _)) => {
                            if len >= 12 {
                                packets += 1;
                                match buf[0] {
                                    ACK => {
                                        ack_received = true;
                                    }
                                    FRAME_DATA => {
                                        frames += 1;
                                        if len >= 17 {
                                            let width = u16::from_le_bytes([buf[12], buf[13]]);
                                            let height = u16::from_le_bytes([buf[14], buf[15]]);
                                            let format = buf[16];

                                            let frame_data = json!({
                                                "width": width,
                                                "height": height,
                                                "format": format,
                                                "data": &buf[17..len]
                                            });

                                            let _ = window.emit("frame_data", frame_data);
                                        }
                                    }
                                    FRAME_DATA_COMPRESSED => {
                                        frames += 1;
                                        let compressed_data = &buf[12..len];
                                        let _ = window.emit("frame_data_compressed", compressed_data);
                                    }
                                    SPECTRUM_DATA => {
                                        spectrum += 1;
                                        if len >= 14 {
                                            let band_count = u16::from_le_bytes([buf[12], buf[13]]);
                                            if len >= 14 + (band_count as usize * 4) {
                                                let mut spectrum_values = Vec::with_capacity(band_count as usize);
                                                for i in 0..band_count {
                                                    let offset = 14 + (i as usize * 4);
                                                    let value = f32::from_le_bytes([
                                                        buf[offset], buf[offset + 1],
                                                        buf[offset + 2], buf[offset + 3]
                                                    ]);
                                                    spectrum_values.push(value);
                                                }
                                                let _ = window.emit("spectrum_data", spectrum_values);
                                            }
                                        }
                                    }
                                    _ => {
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::TimedOut {
                                continue;
                            } else {
                                return Err(NetworkError::from_io_error(&e));
                            }
                        }
                    }
                }

                if !ack_received {
                    return Ok("⚠️ No connection confirmation received".to_string());
                }

                if packets == 1 && frames == 0 && spectrum == 0 {
                    Ok("📡 Connected but no data received (silent server)".to_string())
                } else {
                    Ok(format!("📡 Stream received: {} packets ({} frames, {} spectrum)", packets, frames, spectrum))
                }
            }
        },
        config,
        "Listen Data Stream"
    ).await
}

#[tauri::command]
async fn dj_get_server_info(server_address: Option<String>) -> Result<String, String> {
    let address = server_address.unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string());
    Ok(format!("🖥️ DJ-4LED Server: {}", address))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let connection_state: ConnectionState = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(connection_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            dj_connect,
            dj_disconnect,
            dj_ping,
            dj_set_effect,
            dj_set_color_mode,
            dj_set_custom_color,
            dj_listen_data,
            dj_get_server_info,
            dj_diagnose_network
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
