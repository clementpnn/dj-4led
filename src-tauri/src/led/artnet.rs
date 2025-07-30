use std::net::UdpSocket;

/// Client Art-Net optimisé pour production
pub struct ArtNetClient {
    socket: UdpSocket,
    packets_sent: u64,
    bytes_sent: u64,
}

impl ArtNetClient {
    /// Créer client Art-Net pour production
    pub fn new() -> Result<Self, String> {
        println!("🌐 [ARTNET] Création client Art-Net PRODUCTION");

        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| {
                let error = format!("Erreur socket UDP PRODUCTION: {}", e);
                println!("❌ [ARTNET] {}", error);
                error
            })?;

        // Configuration optimale pour production
        socket.set_broadcast(true).map_err(|e| {
            format!("Erreur broadcast PRODUCTION: {}", e)
        })?;

        println!("✅ [ARTNET] Socket UDP PRODUCTION créé");

        Ok(Self {
            socket,
            packets_sent: 0,
            bytes_sent: 0,
        })
    }

    /// Envoyer données DMX - Format exact pour BC216
    pub fn send_universe(&mut self, universe: u16, dmx_data: &[u8], target: &str) -> Result<usize, String> {
        if dmx_data.len() > 512 {
            return Err(format!("Données DMX trop larges: {} bytes (max 512)", dmx_data.len()));
        }

        // Créer paquet Art-Net BC216-compatible
        let packet = self.create_artnet_packet(universe, dmx_data);

        // Ajouter port Art-Net si manquant
        let target_with_port = if target.contains(':') {
            target.to_string()
        } else {
            format!("{}:6454", target)
        };

        match self.socket.send_to(&packet, &target_with_port) {
            Ok(bytes) => {
                self.packets_sent += 1;
                self.bytes_sent += bytes as u64;

                // Log adaptatif selon le target
                if target_with_port.starts_with("192.168.1") {
                    if self.packets_sent % 2000 == 0 {
                        println!("📡 [ARTNET] PRODUCTION {} packets → {}", self.packets_sent, target_with_port);
                    }
                } else {
                    if self.packets_sent % 1000 == 0 {
                        println!("📡 [ARTNET] SIMULATEUR {} packets → {}", self.packets_sent, target_with_port);
                    }
                }

                Ok(bytes)
            }
            Err(e) => {
                let error = format!("Erreur PRODUCTION {} univers {}: {}", target_with_port, universe, e);
                if self.packets_sent % 100 == 0 {  // Moins de logs d'erreur
                    println!("❌ [ARTNET] {}", error);
                }
                Err(error)
            }
        }
    }

    /// Créer paquet Art-Net compatible BC216 - Format exact
    fn create_artnet_packet(&self, universe: u16, dmx_data: &[u8]) -> Vec<u8> {
        // Header Art-Net BC216 - Format validé avec l'ancien code
        let mut packet = vec![
            b'A', b'r', b't', b'-', b'N', b'e', b't', 0,  // "Art-Net\0"
            0x00, 0x50,                                     // OpCode (ArtDMX)
            0,                                              // ProtVer MSB
            14,                                             // ProtVer LSB (version 14)
            0,                                              // Sequence
            0,                                              // Physical
            (universe & 0xFF) as u8,                       // Universe LSB
            (universe >> 8) as u8,                         // Universe MSB
            0x02,                                           // Length MSB (512 = 0x0200)
            0x00,                                           // Length LSB
        ];

        // Toujours 512 bytes de données DMX pour BC216
        let mut dmx_data_padded = vec![0u8; 512];
        let copy_len = dmx_data.len().min(512);
        dmx_data_padded[..copy_len].copy_from_slice(&dmx_data[..copy_len]);

        packet.extend_from_slice(&dmx_data_padded);
        packet
    }

    /// Test connectivité production - Test projecteur
    pub fn test_connectivity(&mut self, target: &str) -> Result<(), String> {
        println!("🔍 [ARTNET] Test PRODUCTION connectivité: {}", target);

        // Test avec projecteur si c'est le premier contrôleur
        if target.starts_with("192.168.1.45") {
            println!("🎥 [ARTNET] Test projecteur sur {}", target);

            let mut test_data = vec![0u8; 512];
            test_data[0] = 128; // Rouge modéré
            test_data[1] = 0;
            test_data[2] = 0;

            self.send_universe(200, &test_data, target)?;

            // Petit délai puis extinction
            std::thread::sleep(std::time::Duration::from_millis(500));

            let black_data = vec![0u8; 512];
            self.send_universe(200, &black_data, target)?;

            println!("✅ [ARTNET] Test projecteur PRODUCTION OK: {}", target);
        } else {
            // Test standard pour autres contrôleurs
            let mut test_data = vec![0u8; 512];
            test_data[0] = 64;  // Rouge faible

            self.send_universe(0, &test_data, target)?;
            println!("✅ [ARTNET] Test contrôleur PRODUCTION OK: {}", target);
        }

        Ok(())
    }

    /// Statistiques production
    pub fn get_stats(&self) -> (u64, u64) {
        (self.packets_sent, self.bytes_sent)
    }

    /// Reset stats
    pub fn reset_stats(&mut self) {
        let old_packets = self.packets_sent;
        let old_bytes = self.bytes_sent;

        self.packets_sent = 0;
        self.bytes_sent = 0;

        println!("📊 [ARTNET] PRODUCTION Stats reset - Ancien: {} packets, {} bytes",
                 old_packets, old_bytes);
    }
}
