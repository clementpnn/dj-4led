use anyhow::Result;
use std::collections::HashMap;
use std::net::UdpSocket;

use super::protocol::{Entity, UniverseConfig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ControllerConfig {
    pub ip_address: String,
    pub start_universe: u16,
    pub universe_count: u16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityMapping {
    pub controller_ip: String,
    pub universe: u16,
    pub dmx_channel: u16,
}

pub struct IHubRouter {
    socket: UdpSocket,
    controllers: Vec<ControllerConfig>,
    entity_mappings: HashMap<u16, EntityMapping>,
    dmx_buffers: HashMap<(String, u16), Vec<u8>>,
}

impl IHubRouter {
    pub fn new() -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        Ok(Self {
            socket,
            controllers: Vec::new(),
            entity_mappings: HashMap::new(),
            dmx_buffers: HashMap::new(),
        })
    }

    pub fn configure_controllers(&mut self, controllers: Vec<ControllerConfig>) {
        self.controllers = controllers;
        self.build_entity_mappings();
    }

    fn build_entity_mappings(&mut self) {
        self.entity_mappings.clear();
        self.dmx_buffers.clear();

        let quarters = [
            (100u16, 4858u16, "192.168.1.45", 0u16),
            (5100u16, 9858u16, "192.168.1.46", 32u16),
            (10100u16, 14858u16, "192.168.1.47", 64u16),
            (15100u16, 19858u16, "192.168.1.48", 96u16),
        ];

        for (entity_start, entity_end, ip, universe_start) in quarters {
            for band in 0..16 {
                let band_entity_base = entity_start + band * 300;
                let universe_base = universe_start + band * 2;

                for led in 0..259 {
                    let entity_id = band_entity_base + led;
                    if entity_id > entity_end {
                        break;
                    }

                    let universe = if led < 170 {
                        universe_base
                    } else {
                        universe_base + 1
                    };

                    let dmx_channel = if led < 170 { led * 3 } else { (led - 170) * 3 };

                    self.entity_mappings.insert(
                        entity_id,
                        EntityMapping {
                            controller_ip: ip.to_string(),
                            universe,
                            dmx_channel,
                        },
                    );

                    let key = (ip.to_string(), universe);
                    self.dmx_buffers
                        .entry(key)
                        .or_insert_with(|| vec![0u8; 512]);
                }
            }
        }
    }

    pub fn route_entities(&mut self, entities: &[Entity]) -> Result<()> {
        for buffer in self.dmx_buffers.values_mut() {
            buffer.fill(0);
        }

        for entity in entities {
            if let Some(mapping) = self.entity_mappings.get(&entity.id) {
                let key = (mapping.controller_ip.clone(), mapping.universe);
                if let Some(buffer) = self.dmx_buffers.get_mut(&key) {
                    let ch = mapping.dmx_channel as usize;
                    if ch + 2 < 512 {
                        buffer[ch] = entity.r;
                        buffer[ch + 1] = entity.g;
                        buffer[ch + 2] = entity.b;
                    }
                }
            }
        }

        self.send_artnet_packets()?;

        Ok(())
    }

    fn send_artnet_packets(&mut self) -> Result<()> {
        for ((ip, universe), dmx_data) in &self.dmx_buffers {
            let mut packet = vec![
                b'A',
                b'r',
                b't',
                b'-',
                b'N',
                b'e',
                b't',
                0,
                0x00,
                0x50,
                0,
                14,
                0,
                0,
                (*universe & 0xFF) as u8,
                (*universe >> 8) as u8,
                0x02,
                0x00,
            ];

            packet.extend_from_slice(dmx_data);

            let addr = format!("{}:6454", ip);
            self.socket.send_to(&packet, &addr)?;
        }

        Ok(())
    }

    pub fn route_frame(&mut self, frame: &[u8], width: usize, height: usize) -> Result<()> {
        let entities = super::frame_to_entities(frame, width, height);

        let entities: Vec<Entity> = entities
            .into_iter()
            .map(|(id, r, g, b, w)| Entity { id, r, g, b, w })
            .collect();

        self.route_entities(&entities)?;

        Ok(())
    }

    pub fn apply_patch(&mut self, _patch_map: HashMap<u16, u16>) {}

    pub fn get_stats(&self) -> RouterStats {
        RouterStats {
            entity_count: self.entity_mappings.len(),
            controller_count: self.controllers.len(),
            universe_count: self.dmx_buffers.len(),
            total_dmx_channels: self.dmx_buffers.values().map(|b| b.len()).sum(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouterStats {
    pub entity_count: usize,
    pub controller_count: usize,
    pub universe_count: usize,
    pub total_dmx_channels: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallationConfig {
    pub name: String,
    pub controllers: Vec<ControllerConfig>,
    pub universe_configs: Vec<UniverseConfig>,
}

impl InstallationConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_initialization() {
        let mut router = IHubRouter::new().unwrap();

        let controllers = vec![ControllerConfig {
            ip_address: "192.168.1.45".to_string(),
            start_universe: 0,
            universe_count: 32,
        }];

        router.configure_controllers(controllers);
        let stats = router.get_stats();

        assert!(stats.entity_count > 0);
        assert_eq!(stats.controller_count, 1);
    }
}
