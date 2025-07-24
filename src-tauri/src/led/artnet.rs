use anyhow::Result;
use std::net::UdpSocket;
use std::time::Duration;

/// Client Art-Net pour l'envoi de données DMX
pub struct ArtNetClient {
    socket: UdpSocket,
    target_ip: String,
    packets_sent: u64,
    bytes_sent: u64,
}

impl ArtNetClient {
    /// Crée un nouveau client Art-Net
    pub fn new(target_ip: &str) -> Result<Self> {
        println!("🌐 [ART-NET] Client créé pour {}", target_ip);

        // Créer socket avec configuration améliorée
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(false)?;

        // Ajouter timeout pour éviter les blocages
        socket.set_write_timeout(Some(Duration::from_millis(10)))?;
        socket.set_broadcast(true)?; // Permet le broadcast si nécessaire

        Ok(Self {
            socket,
            target_ip: target_ip.to_string(),
            packets_sent: 0,
            bytes_sent: 0,
        })
    }

    /// Envoie des données DMX pour un univers spécifique
    pub fn send_universe(&mut self, universe: u16, dmx_data: &[u8]) -> Result<usize> {
        // Validation des données
        if dmx_data.len() > 512 {
            let error = format!("DMX data too large: {} bytes (max 512)", dmx_data.len());
            anyhow::bail!(error);
        }

        // Validation univers
        if universe > 32767 {
            let error = format!("Universe too large: {} (max 32767)", universe);
            anyhow::bail!(error);
        }

        // Création du paquet Art-Net
        let packet = self.create_artnet_packet(universe, dmx_data);
        let addr = format!("{}:6454", self.target_ip);

        // Envoi du paquet avec retry en cas d'échec temporaire
        let mut attempts = 0;
        let max_attempts = 2;

        loop {
            match self.socket.send_to(&packet, &addr) {
                Ok(bytes) => {
                    // Mise à jour des statistiques
                    self.packets_sent += 1;
                    self.bytes_sent += bytes as u64;
                    return Ok(bytes);
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        let error = format!("Failed to send to {} after {} attempts: {}", addr, attempts, e);
                        return Err(anyhow::anyhow!(error));
                    }
                    // Petit délai avant retry
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
    }

    /// Crée un paquet Art-Net conforme aux spécifications
    fn create_artnet_packet(&self, universe: u16, dmx_data: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(18 + dmx_data.len());

        // Header Art-Net selon la spécification officielle
        packet.extend_from_slice(b"Art-Net\0"); // 8 bytes - ID string
        packet.extend_from_slice(&[0x00, 0x50]); // 2 bytes - OpCode (ArtDMX = 0x5000 little endian)
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
}

/// Utilitaires pour la manipulation des couleurs LED
pub mod utils {
    /// Applique la correction de luminosité sur un pixel RGB
    pub fn apply_brightness_rgb(r: u8, g: u8, b: u8, brightness: f32) -> (u8, u8, u8) {
        let brightness = brightness.clamp(0.0, 1.0);
        (
            (r as f32 * brightness) as u8,
            (g as f32 * brightness) as u8,
            (b as f32 * brightness) as u8,
        )
    }

    /// Applique la correction gamma sur un pixel RGB
    pub fn apply_gamma_rgb(r: u8, g: u8, b: u8, gamma: f32) -> (u8, u8, u8) {
        let inv_gamma = 1.0 / gamma;
        (
            ((r as f32 / 255.0).powf(inv_gamma) * 255.0) as u8,
            ((g as f32 / 255.0).powf(inv_gamma) * 255.0) as u8,
            ((b as f32 / 255.0).powf(inv_gamma) * 255.0) as u8,
        )
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
