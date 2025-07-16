// src-tauri/src/lib.rs
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        .map_err(|e| format!("Erreur création socket: {}", e))?;

    socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))
        .map_err(|e| format!("Erreur configuration timeout: {}", e))?;

    Ok(socket)
}

fn get_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

#[tauri::command]
async fn dj_connect() -> Result<String, String> {
    let socket = create_socket_with_timeout(3)?;

    // Packet Connect selon la doc
    let connect_packet = create_packet(CONNECT, 0x00, 0, vec![]);

    socket.send_to(&connect_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Connexion échouée: {}", e))?;

    // Attendre ACK
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            if len >= 1 && buf[0] == ACK {
                Ok(format!("✅ Connecté au serveur DJ-4LED ({})", addr))
            } else {
                Ok(format!("⚠️ Réponse inattendue: type {:#04x}", buf[0]))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok("⏰ Timeout - serveur DJ-4LED hors ligne".to_string())
            } else {
                Err(format!("Erreur réception: {}", e))
            }
        }
    }
}

#[tauri::command]
async fn dj_disconnect() -> Result<String, String> {
    let socket = create_socket_with_timeout(2)?;

    // Packet Disconnect selon la doc
    let disconnect_packet = create_packet(DISCONNECT, 0x00, get_timestamp(), vec![]);

    socket.send_to(&disconnect_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Déconnexion échouée: {}", e))?;

    // Optionnel: attendre confirmation (ACK)
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, _)) => {
            if len >= 1 && buf[0] == ACK {
                Ok("✅ Déconnecté proprement du serveur DJ-4LED".to_string())
            } else {
                Ok("✅ Déconnexion envoyée (pas de confirmation)".to_string())
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok("✅ Déconnexion envoyée (timeout sur confirmation)".to_string())
            } else {
                Ok("✅ Déconnexion envoyée".to_string())
            }
        }
    }
}

#[tauri::command]
async fn dj_ping() -> Result<String, String> {
    let socket = create_socket_with_timeout(3)?;

    let ping_packet = create_packet(PING, 0x00, get_timestamp(), vec![]);

    socket.send_to(&ping_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Ping échoué: {}", e))?;

    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            if len >= 1 && buf[0] == PONG {
                Ok(format!("🏓 PONG reçu de {}", addr))
            } else {
                Ok(format!("⚠️ Réponse ping inattendue: type {:#04x}", buf[0]))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok("⏰ Timeout - serveur ne répond pas au ping".to_string())
            } else {
                Err(format!("Erreur ping: {}", e))
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
        .map_err(|e| format!("Commande effet échouée: {}", e))?;

    Ok(format!("✅ Effet {} appliqué", effect_id))
}

#[tauri::command]
async fn dj_set_color_mode(mode: String) -> Result<String, String> {
    let socket = create_socket_with_timeout(2)?;

    let mut payload = vec![SET_COLOR_MODE];
    payload.extend_from_slice(mode.as_bytes());

    let packet = create_packet(COMMAND, 0x00, get_timestamp(), payload);

    socket.send_to(&packet, SERVER_ADDRESS)
        .map_err(|e| format!("Commande mode couleur échouée: {}", e))?;

    Ok(format!("✅ Mode couleur '{}' appliqué", mode))
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
        .map_err(|e| format!("Commande couleur personnalisée échouée: {}", e))?;

    Ok(format!("✅ Couleur RGB({:.3}, {:.3}, {:.3}) appliquée", r, g, b))
}

#[tauri::command]
async fn dj_listen_data() -> Result<String, String> {
    let socket = create_socket_with_timeout(8)?;

    // Se connecter d'abord avec support compression
    let connect_packet = create_packet(CONNECT, 0x01, 0, vec![]);
    socket.send_to(&connect_packet, SERVER_ADDRESS)
        .map_err(|e| format!("Connexion stream échouée: {}", e))?;

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
                if len >= 1 {
                    packets += 1;
                    match buf[0] {
                        ACK => {
                            ack_received = true;
                        }
                        FRAME_DATA => {
                            frames += 1;
                        }
                        FRAME_DATA_COMPRESSED => {
                            frames += 1;
                        }
                        SPECTRUM_DATA => {
                            spectrum += 1;
                        }
                        _ => {
                            // Autres types de paquets ignorés
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    continue;
                } else {
                    return Err(format!("Erreur écoute stream: {}", e));
                }
            }
        }
    }

    if !ack_received {
        return Ok("⚠️ Aucune confirmation de connexion reçue".to_string());
    }

    if packets == 1 && frames == 0 && spectrum == 0 {
        Ok("📡 Connecté mais aucune donnée reçue (serveur silencieux)".to_string())
    } else {
        Ok(format!("📡 Stream reçu: {} paquets ({} frames, {} spectrum)", packets, frames, spectrum))
    }
}

#[tauri::command]
async fn dj_get_server_info() -> Result<String, String> {
    Ok(format!("🖥️ Serveur DJ-4LED: {}", SERVER_ADDRESS))
}

// Garde ta fonction greet
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            dj_connect,
            dj_disconnect,
            dj_ping,
            dj_set_effect,
            dj_set_color_mode,
            dj_set_custom_color,
            dj_listen_data,
            dj_get_server_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
