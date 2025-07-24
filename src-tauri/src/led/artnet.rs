use anyhow::Result;
use std::net::UdpSocket;
use std::time::Instant;

/// Client Art-Net pour l'envoi de données DMX
pub struct ArtNetClient {
    socket: UdpSocket,
    target_ip: String,
    packets_sent: u64,
    bytes_sent: u64,
    last_log_time: Option<Instant>,
}

impl ArtNetClient {
    /// Crée un nouveau client Art-Net
    pub fn new(target_ip: &str) -> Result<Self> {
        println!("🌐 [ART-NET] Client créé pour {}", target_ip);

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(false)?;

        Ok(Self {
            socket,
            target_ip: target_ip.to_string(),
            packets_sent: 0,
            bytes_sent: 0,
            last_log_time: None,
        })
    }

    /// Envoie des données DMX pour un univers spécifique
    pub fn send_universe(&mut self, universe: u16, dmx_data: &[u8]) -> Result<usize> {
        // Validation des données
        if dmx_data.len() > 512 {
            let error = format!("DMX data too large: {} bytes (max 512)", dmx_data.len());
            println!("❌ [ART-NET] Error: {}", error);
            anyhow::bail!(error);
        }

        // Log périodique (toutes les 60 frames ou les 5 premières)
        let should_log = self.packets_sent % 60 == 0 || self.packets_sent < 5;

        if should_log {
            println!("📡 [ART-NET] Send #{} -> Universe: {}, Size: {} bytes",
                     self.packets_sent + 1, universe, dmx_data.len());
        }

        // Création du paquet Art-Net
        let packet = self.create_artnet_packet(universe, dmx_data);
        let addr = format!("{}:6454", self.target_ip);

        // Envoi du paquet
        match self.socket.send_to(&packet, &addr) {
            Ok(bytes) => {
                // Mise à jour des statistiques
                self.packets_sent += 1;
                self.bytes_sent += bytes as u64;

                if should_log {
                    if let Some(last_time) = self.last_log_time {
                        let elapsed = last_time.elapsed().as_secs_f32();
                        if elapsed > 0.0 {
                            let fps = 60.0 / elapsed;
                            println!("  ✅ [ART-NET] FPS: {:.1}, Total: {} KB", fps, self.bytes_sent / 1024);
                        }
                    }
                    self.last_log_time = Some(Instant::now());
                }

                Ok(bytes)
            }
            Err(e) => {
                let error = format!("Failed to send to {}: {}", addr, e);
                println!("❌ [ART-NET] Send Error: {}", error);
                Err(anyhow::anyhow!(error))
            }
        }
    }

    /// Crée un paquet Art-Net complet
    fn create_artnet_packet(&self, universe: u16, dmx_data: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(18 + dmx_data.len());

        // Header Art-Net selon la spécification
        packet.extend_from_slice(b"Art-Net\0"); // 8 bytes - ID
        packet.extend_from_slice(&[0x00, 0x50]); // 2 bytes - OpCode (ArtDMX = 0x5000)
        packet.extend_from_slice(&[0x00, 0x0E]); // 2 bytes - ProtVer (version 14)
        packet.push(0); // 1 byte - Sequence (0 = pas de séquençage)
        packet.push(0); // 1 byte - Physical (port physique)
        packet.extend_from_slice(&universe.to_le_bytes()); // 2 bytes - Universe (little endian)
        packet.extend_from_slice(&(dmx_data.len() as u16).to_be_bytes()); // 2 bytes - Length (big endian)

        // Données DMX
        packet.extend_from_slice(dmx_data);

        packet
    }

    /// Obtient l'IP cible
    pub fn target_ip(&self) -> &str {
        &self.target_ip
    }

    /// Obtient les statistiques d'envoi
    pub fn get_stats(&self) -> (u64, u64) {
        (self.packets_sent, self.bytes_sent)
    }

    /// Test de connectivité de base
    pub fn test_connectivity(&mut self) -> Result<bool> {
        println!("🔍 [ART-NET] Test connectivité {}", self.target_ip);

        let test_data = vec![0; 12];

        match self.send_universe(0, &test_data) {
            Ok(bytes) => {
                println!("  ✅ [ART-NET] Test réussi: {} bytes", bytes);
                Ok(true)
            }
            Err(e) => {
                println!("  ❌ [ART-NET] Test échoué: {}", e);
                Ok(false)
            }
        }
    }
}

/// Utilitaires pour la manipulation des couleurs LED
pub mod utils {
    use std::sync::atomic::{AtomicU32, Ordering};

    static DEBUG_COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Applique la correction de luminosité sur un pixel RGB
    pub fn apply_brightness_rgb(r: u8, g: u8, b: u8, brightness: f32) -> (u8, u8, u8) {
        let brightness = brightness.clamp(0.0, 1.0);

        let counter = DEBUG_COUNTER.fetch_add(1, Ordering::Relaxed);
        if counter % 5000 == 0 {
            println!("🔆 [UTILS] Brightness: {:.2}", brightness);
        }

        (
            (r as f32 * brightness) as u8,
            (g as f32 * brightness) as u8,
            (b as f32 * brightness) as u8,
        )
    }

    /// Applique la correction gamma sur un pixel RGB
    pub fn apply_gamma_rgb(r: u8, g: u8, b: u8, gamma: f32) -> (u8, u8, u8) {
        let inv_gamma = 1.0 / gamma;

        let result = (
            ((r as f32 / 255.0).powf(inv_gamma) * 255.0) as u8,
            ((g as f32 / 255.0).powf(inv_gamma) * 255.0) as u8,
            ((b as f32 / 255.0).powf(inv_gamma) * 255.0) as u8,
        );

        result
    }

    /// Applique une température de couleur
    pub fn apply_color_temperature_rgb(r: u8, g: u8, b: u8, temperature: f32) -> (u8, u8, u8) {
        if temperature == 1.0 {
            return (r, g, b);
        }

        let result = if temperature < 1.0 {
            (r, g, (b as f32 / temperature) as u8)
        } else {
            ((r as f32 / temperature) as u8, g, b)
        };

        result
    }

    /// Convertit HSV en RGB
    pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
        let h = h % 360.0;
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        (
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }
}
