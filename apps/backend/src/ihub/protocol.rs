use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IHubConfig {
    pub magic: [u8; 4],
    pub msg_type: u8,
    pub universe: u8,
    pub range_count: u16,
    pub payload_size: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IHubUpdate {
    pub magic: [u8; 4],
    pub msg_type: u8,
    pub universe: u8,
    pub entity_count: u16,
    pub payload_size: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntityRange {
    pub sextet_start: u16,
    pub entity_start: u16,
    pub sextet_end: u16,
    pub entity_end: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub id: u16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub w: u8,
}

impl Entity {
    #[inline(always)]
    pub fn new(id: u16, r: u8, g: u8, b: u8) -> Self {
        Self { id, r, g, b, w: 0 }
    }

    #[inline(always)]
    pub fn new_rgbw(id: u16, r: u8, g: u8, b: u8, w: u8) -> Self {
        Self { id, r, g, b, w }
    }

    #[inline(always)]
    pub fn to_sextet(&self) -> [u8; 6] {
        let id_bytes = self.id.to_le_bytes();
        [id_bytes[0], id_bytes[1], self.r, self.g, self.b, self.w]
    }

    #[inline(always)]
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

    #[inline(always)]
    pub fn is_lit(&self) -> bool {
        self.r > 0 || self.g > 0 || self.b > 0 || self.w > 0
    }

    #[inline(always)]
    pub fn brightness(&self) -> u16 {
        self.r as u16 + self.g as u16 + self.b as u16 + self.w as u16
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseConfig {
    pub universe_id: u8,
    pub entity_ranges: Vec<EntityRange>,
    #[serde(skip)]
    lookup_cache: Option<Vec<(u16, u16, u16)>>,
}

impl UniverseConfig {
    pub fn new(universe_id: u8) -> Self {
        Self {
            universe_id,
            entity_ranges: Vec::with_capacity(64),
            lookup_cache: None,
        }
    }

    pub fn add_range(&mut self, entity_start: u16, entity_end: u16, sextet_start: u16) {
        let count = entity_end - entity_start + 1;
        let sextet_end = sextet_start + count - 1;

        self.entity_ranges.push(EntityRange {
            sextet_start,
            entity_start,
            sextet_end,
            entity_end,
        });

        self.lookup_cache = None;
    }

    fn build_cache(&mut self) {
        if self.lookup_cache.is_none() {
            let mut cache: Vec<(u16, u16, u16)> = self
                .entity_ranges
                .iter()
                .map(|r| (r.entity_start, r.entity_end, r.sextet_start))
                .collect();
            cache.sort_unstable_by_key(|&(start, _, _)| start);
            self.lookup_cache = Some(cache);
        }
    }

    pub fn get_sextet_position(&mut self, entity_id: u16) -> Option<u16> {
        self.build_cache();

        if let Some(ref cache) = self.lookup_cache {
            let result = cache.binary_search_by(|&(start, end, _)| {
                if entity_id < start {
                    std::cmp::Ordering::Greater
                } else if entity_id > end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            });

            if let Ok(idx) = result {
                let (start, _, sextet_start) = cache[idx];
                let offset = entity_id - start;
                return Some(sextet_start + offset);
            }
        }

        None
    }

    pub fn total_entities(&self) -> u32 {
        self.entity_ranges
            .iter()
            .map(|r| (r.entity_end - r.entity_start + 1) as u32)
            .sum()
    }
}

pub mod constants {
    pub const IHUB_MAGIC: &[u8; 4] = b"iHuB";
    pub const MSG_TYPE_CONFIG: u8 = 1;
    pub const MSG_TYPE_UPDATE: u8 = 2;
    pub const MSG_TYPE_DIFFERENTIAL: u8 = 3;
    pub const MAX_UDP_SIZE: usize = 65507;
    pub const UPDATE_FREQUENCY: u64 = 40;
    pub const CONFIG_FREQUENCY: u64 = 1;
    pub const COMPRESSION_LEVEL: i32 = 1;
}

pub struct EntityBatch {
    entities: Vec<Entity>,
    dirty_mask: Vec<bool>,
}

impl EntityBatch {
    pub fn new(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            dirty_mask: vec![false; capacity],
        }
    }

    pub fn update(&mut self, entity: Entity) -> bool {
        if let Some(pos) = self.entities.iter().position(|e| e.id == entity.id) {
            if self.entities[pos] != entity {
                self.entities[pos] = entity;
                self.dirty_mask[pos] = true;
                true
            } else {
                false
            }
        } else {
            self.entities.push(entity);
            if self.dirty_mask.len() <= self.entities.len() {
                self.dirty_mask.resize(self.entities.len() + 100, false);
            }
            self.dirty_mask[self.entities.len() - 1] = true;
            true
        }
    }

    pub fn get_dirty_entities(&self) -> Vec<&Entity> {
        self.entities
            .iter()
            .enumerate()
            .filter(|(i, _)| self.dirty_mask[*i])
            .map(|(_, e)| e)
            .collect()
    }

    pub fn clear_dirty_flags(&mut self) {
        self.dirty_mask.fill(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_sextet_conversion() {
        let entity = Entity::new(1234, 255, 128, 64);
        let sextet = entity.to_sextet();

        assert_eq!(sextet[0], 210);
        assert_eq!(sextet[1], 4);
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
    fn test_universe_config_with_cache() {
        let mut config = UniverseConfig::new(0);
        config.add_range(1, 170, 0);
        config.add_range(200, 370, 170);

        assert_eq!(config.get_sextet_position(1), Some(0));
        assert_eq!(config.get_sextet_position(170), Some(169));
        assert_eq!(config.get_sextet_position(200), Some(170));
        assert_eq!(config.get_sextet_position(370), Some(340));
        assert_eq!(config.get_sextet_position(171), None);

        assert!(config.lookup_cache.is_some());
    }

    #[test]
    fn test_entity_batch() {
        let mut batch = EntityBatch::new(10);

        let e1 = Entity::new(1, 255, 0, 0);
        let e2 = Entity::new(2, 0, 255, 0);

        assert!(batch.update(e1));
        assert!(batch.update(e2));

        let dirty = batch.get_dirty_entities();
        assert_eq!(dirty.len(), 2);

        assert!(!batch.update(e1));

        let e1_modified = Entity::new(1, 128, 0, 0);
        assert!(batch.update(e1_modified));
    }

    #[test]
    fn test_entity_brightness() {
        let entity = Entity::new_rgbw(1, 100, 150, 200, 50);
        assert_eq!(entity.brightness(), 500);
        assert!(entity.is_lit());

        let dark = Entity::new(2, 0, 0, 0);
        assert_eq!(dark.brightness(), 0);
        assert!(!dark.is_lit());
    }
}
