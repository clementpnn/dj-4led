use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;
use std::net::UdpSocket;

pub mod protocol;
pub mod router;

use protocol::EntityRange;

pub struct EHubController {
    socket: UdpSocket,
    target_address: String,
    universe: u8,
    entities: HashMap<u16, (u8, u8, u8, u8)>, // id -> (R, G, B, W)
    entity_ranges: Vec<EntityRange>,
    last_config_time: std::time::Instant,
}

impl EHubController {
    pub fn new(target_address: &str, universe: u8) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        Ok(Self {
            socket,
            target_address: target_address.to_string(),
            universe,
            entities: HashMap::new(),
            entity_ranges: Vec::new(),
            last_config_time: std::time::Instant::now(),
        })
    }

    /// Configure les plages d'entités pour ce contrôleur
    pub fn configure_entities(&mut self, ranges: Vec<EntityRange>) {
        self.entity_ranges = ranges;
        self.send_config();
    }

    /// Met à jour l'état des entités
    pub fn update_entities(&mut self, entities: Vec<(u16, u8, u8, u8, u8)>) {
        for (id, r, g, b, w) in entities {
            self.entities.insert(id, (r, g, b, w));
        }
        self.send_update();
    }

    /// Envoie un message de mise à jour eHub
    fn send_update(&self) {
        // Préparer le payload
        let mut payload = Vec::new();

        // Trier les entités par ID pour un encodage cohérent
        let mut sorted_entities: Vec<_> = self.entities.iter().collect();
        sorted_entities.sort_by_key(|(id, _)| *id);

        // Encoder chaque entité (6 octets)
        for (id, (r, g, b, w)) in sorted_entities {
            payload.extend_from_slice(&id.to_le_bytes());
            payload.push(*r);
            payload.push(*g);
            payload.push(*b);
            payload.push(*w);
        }

        // Compresser le payload
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&payload).unwrap();
        let compressed = encoder.finish().unwrap();

        // Construire le message eHub
        let mut message = Vec::new();
        message.extend_from_slice(b"eHuB");
        message.push(2); // Type: update
        message.push(self.universe);
        message.extend_from_slice(&(self.entities.len() as u16).to_le_bytes());
        message.extend_from_slice(&(compressed.len() as u16).to_le_bytes());
        message.extend_from_slice(&compressed);

        // Envoyer
        let _ = self.socket.send_to(&message, &self.target_address);
    }

    /// Envoie un message de configuration eHub
    fn send_config(&self) {
        let mut payload = Vec::new();

        // Encoder chaque plage
        for range in &self.entity_ranges {
            payload.extend_from_slice(&range.sextet_start.to_le_bytes());
            payload.extend_from_slice(&range.entity_start.to_le_bytes());
            payload.extend_from_slice(&range.sextet_end.to_le_bytes());
            payload.extend_from_slice(&range.entity_end.to_le_bytes());
        }

        // Compresser
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&payload).unwrap();
        let compressed = encoder.finish().unwrap();

        // Construire le message
        let mut message = Vec::new();
        message.extend_from_slice(b"eHuB");
        message.push(1); // Type: config
        message.push(self.universe);
        message.extend_from_slice(&(self.entity_ranges.len() as u16).to_le_bytes());
        message.extend_from_slice(&(compressed.len() as u16).to_le_bytes());
        message.extend_from_slice(&compressed);

        // Envoyer
        let _ = self.socket.send_to(&message, &self.target_address);
    }

    /// Envoie périodiquement la configuration (1 fois par seconde)
    pub fn tick(&mut self) {
        if self.last_config_time.elapsed().as_secs() >= 1 {
            self.send_config();
            self.last_config_time = std::time::Instant::now();
        }
    }
}

/// Convertit un frame RGB en entités eHub pour l'écran LED
pub fn frame_to_entities(frame: &[u8], width: usize, height: usize) -> Vec<(u16, u8, u8, u8, u8)> {
    let mut entities = Vec::new();

    // Mapping selon la documentation du Groupe LAPS
    // 64 bandes de 259 LEDs organisées en 4 quartiers
    for quarter in 0..4 {
        let base_entity: u16 = match quarter {
            0 => 100,   // Quartier 1: 100-4858
            1 => 5100,  // Quartier 2: 5100-9858
            2 => 10100, // Quartier 3: 10100-14858
            3 => 15100, // Quartier 4: 15100-19858
            _ => unreachable!(),
        };

        // 16 bandes physiques par quartier
        for band in 0..16 {
            let col_start = quarter * 32 + band * 2;

            // Entité de départ pour cette bande
            let entity_base = base_entity + (band as u16) * 300;

            // Bande montante (colonne paire)
            let col1 = col_start;
            if col1 < width {
                // LED 1 invisible
                entities.push((entity_base, 0, 0, 0, 0));

                // LEDs 2-129 visibles (montée)
                for i in 0..128 {
                    let y = height - 1 - i; // Du bas vers le haut
                    let pixel_idx = (y * width + col1) * 3;
                    if pixel_idx + 2 < frame.len() {
                        entities.push((
                            entity_base + 1 + (i as u16),
                            frame[pixel_idx],
                            frame[pixel_idx + 1],
                            frame[pixel_idx + 2],
                            0, // W
                        ));
                    }
                }

                // LED 130 invisible
                entities.push((entity_base + 129, 0, 0, 0, 0));
            }

            // Bande descendante (colonne impaire)
            let col2 = col_start + 1;
            if col2 < width {
                // LEDs 131-258 visibles (descente)
                for i in 0..128 {
                    let y = i; // Du haut vers le bas
                    let pixel_idx = (y * width + col2) * 3;
                    if pixel_idx + 2 < frame.len() {
                        entities.push((
                            entity_base + 130 + (i as u16),
                            frame[pixel_idx],
                            frame[pixel_idx + 1],
                            frame[pixel_idx + 2],
                            0, // W
                        ));
                    }
                }

                // LED 259 invisible
                entities.push((entity_base + 258, 0, 0, 0, 0));
            }
        }
    }

    entities
}
