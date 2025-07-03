use serde::{Deserialize, Serialize};

/// Message de configuration eHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EHubConfig {
    pub magic: [u8; 4],    // 'eHuB'
    pub msg_type: u8,      // 1 = config
    pub universe: u8,      // Numéro d'univers eHub
    pub range_count: u16,  // Nombre de plages
    pub payload_size: u16, // Taille du payload compressé
    pub payload: Vec<u8>,  // Payload compressé (GZip)
}

/// Message de mise à jour eHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EHubUpdate {
    pub magic: [u8; 4],    // 'eHuB'
    pub msg_type: u8,      // 2 = update
    pub universe: u8,      // Numéro d'univers eHub
    pub entity_count: u16, // Nombre d'entités
    pub payload_size: u16, // Taille du payload compressé
    pub payload: Vec<u8>,  // Payload compressé (GZip)
}

/// Plage d'entités dans un message config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRange {
    pub sextet_start: u16, // Position de départ dans le payload update
    pub entity_start: u16, // ID d'entité de départ
    pub sextet_end: u16,   // Position de fin dans le payload update
    pub entity_end: u16,   // ID d'entité de fin
}

/// Entité avec ses couleurs RGBW
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: u16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub w: u8,
}

impl Entity {
    pub fn new(id: u16, r: u8, g: u8, b: u8) -> Self {
        Self { id, r, g, b, w: 0 }
    }

    pub fn new_rgbw(id: u16, r: u8, g: u8, b: u8, w: u8) -> Self {
        Self { id, r, g, b, w }
    }

    /// Convertit l'entité en sextet (6 octets)
    pub fn to_sextet(&self) -> [u8; 6] {
        let id_bytes = self.id.to_le_bytes();
        [id_bytes[0], id_bytes[1], self.r, self.g, self.b, self.w]
    }

    /// Crée une entité depuis un sextet
    pub fn from_sextet(data: &[u8]) -> Option<Self> {
        if data.len() < 6 {
            return None;
        }

        let id = u16::from_le_bytes([data[0], data[1]]);
        Some(Self {
            id,
            r: data[2],
            g: data[3],
            b: data[4],
            w: data[5],
        })
    }
}

/// Configuration d'un univers eHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseConfig {
    pub universe_id: u8,
    pub entity_ranges: Vec<EntityRange>,
}

impl UniverseConfig {
    pub fn new(universe_id: u8) -> Self {
        Self {
            universe_id,
            entity_ranges: Vec::new(),
        }
    }

    /// Ajoute une plage d'entités
    pub fn add_range(&mut self, entity_start: u16, entity_end: u16, sextet_start: u16) {
        let count = entity_end - entity_start + 1;
        let sextet_end = sextet_start + count - 1;

        self.entity_ranges.push(EntityRange {
            sextet_start,
            entity_start,
            sextet_end,
            entity_end,
        });
    }

    /// Calcule la position du sextet pour une entité donnée
    pub fn get_sextet_position(&self, entity_id: u16) -> Option<u16> {
        for range in &self.entity_ranges {
            if entity_id >= range.entity_start && entity_id <= range.entity_end {
                let offset = entity_id - range.entity_start;
                return Some(range.sextet_start + offset);
            }
        }
        None
    }
}

/// Constantes du protocole eHub
pub mod constants {
    pub const EHUB_MAGIC: &[u8; 4] = b"eHuB";
    pub const MSG_TYPE_CONFIG: u8 = 1;
    pub const MSG_TYPE_UPDATE: u8 = 2;
    pub const MAX_UDP_SIZE: usize = 65507; // Taille max d'un paquet UDP
    pub const UPDATE_FREQUENCY: u64 = 40; // 40 Hz
    pub const CONFIG_FREQUENCY: u64 = 1; // 1 Hz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_sextet_conversion() {
        let entity = Entity::new(1234, 255, 128, 64);
        let sextet = entity.to_sextet();

        assert_eq!(sextet[0], 210); // 1234 & 0xFF
        assert_eq!(sextet[1], 4); // 1234 >> 8
        assert_eq!(sextet[2], 255);
        assert_eq!(sextet[3], 128);
        assert_eq!(sextet[4], 64);
        assert_eq!(sextet[5], 0);

        let restored = Entity::from_sextet(&sextet).unwrap();
        assert_eq!(restored.id, 1234);
        assert_eq!(restored.r, 255);
        assert_eq!(restored.g, 128);
        assert_eq!(restored.b, 64);
        assert_eq!(restored.w, 0);
    }

    #[test]
    fn test_universe_config() {
        let mut config = UniverseConfig::new(0);
        config.add_range(1, 170, 0);
        config.add_range(200, 370, 170);

        assert_eq!(config.get_sextet_position(1), Some(0));
        assert_eq!(config.get_sextet_position(170), Some(169));
        assert_eq!(config.get_sextet_position(200), Some(170));
        assert_eq!(config.get_sextet_position(370), Some(340));
        assert_eq!(config.get_sextet_position(171), None);
    }
}
