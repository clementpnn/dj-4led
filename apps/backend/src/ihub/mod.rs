use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

pub mod protocol;
pub mod router;

use protocol::{Entity, EntityRange};

pub struct IHubController {
    socket: UdpSocket,
    target_address: String,
    universe: u8,
    entities: HashMap<u16, Entity>,
    entity_ranges: Vec<EntityRange>,
    last_config_time: Instant,
    send_buffer: Vec<u8>,
    compression_buffer: Vec<u8>,
    entity_buffer: Vec<(u16, Entity)>,
    dirty_entities: Vec<u16>,
    use_differential_updates: bool,
}

impl IHubController {
    pub fn new(target_address: &str, universe: u8) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            target_address: target_address.to_string(),
            universe,
            entities: HashMap::with_capacity(20000),
            entity_ranges: Vec::with_capacity(64),
            last_config_time: Instant::now(),
            send_buffer: Vec::with_capacity(65507),
            compression_buffer: Vec::with_capacity(32768),
            entity_buffer: Vec::with_capacity(20000),
            dirty_entities: Vec::with_capacity(1000),
            use_differential_updates: true,
        })
    }

    pub fn configure_entities(&mut self, ranges: Vec<EntityRange>) {
        self.entity_ranges = ranges;
        self.send_config();
    }

    pub fn update_entities(&mut self, entities: &[(u16, u8, u8, u8, u8)]) {
        self.dirty_entities.clear();

        for &(id, r, g, b, w) in entities {
            let entity = Entity::new_rgbw(id, r, g, b, w);

            if let Some(existing) = self.entities.get(&id) {
                if existing.r != r || existing.g != g || existing.b != b || existing.w != w {
                    self.entities.insert(id, entity);
                    self.dirty_entities.push(id);
                }
            } else {
                self.entities.insert(id, entity);
                self.dirty_entities.push(id);
            }
        }

        if !self.dirty_entities.is_empty() {
            if self.use_differential_updates && self.dirty_entities.len() < self.entities.len() / 4
            {
                self.send_differential_update();
            } else {
                self.send_full_update();
            }
        }
    }

    fn send_differential_update(&mut self) {
        self.compression_buffer.clear();

        self.dirty_entities.sort_unstable();

        for &id in &self.dirty_entities {
            if let Some(entity) = self.entities.get(&id) {
                self.compression_buffer
                    .extend_from_slice(&entity.to_sextet());
            }
        }

        self.compress_and_send(3, self.dirty_entities.len() as u16);
    }

    fn send_full_update(&mut self) {
        self.compression_buffer.clear();
        self.entity_buffer.clear();

        self.entity_buffer
            .extend(self.entities.iter().map(|(&id, entity)| (id, *entity)));
        self.entity_buffer.sort_unstable_by_key(|(id, _)| *id);

        for (_, entity) in &self.entity_buffer {
            self.compression_buffer
                .extend_from_slice(&entity.to_sextet());
        }

        self.compress_and_send(2, self.entity_buffer.len() as u16);
    }

    fn compress_and_send(&mut self, msg_type: u8, entity_count: u16) {
        let mut encoder = GzEncoder::new(
            Vec::with_capacity(self.compression_buffer.len() / 2),
            Compression::fast(),
        );
        encoder.write_all(&self.compression_buffer).unwrap();
        let compressed = encoder.finish().unwrap();

        self.send_buffer.clear();
        self.send_buffer.extend_from_slice(b"iHuB");
        self.send_buffer.push(msg_type);
        self.send_buffer.push(self.universe);
        self.send_buffer
            .extend_from_slice(&entity_count.to_le_bytes());
        self.send_buffer
            .extend_from_slice(&(compressed.len() as u16).to_le_bytes());
        self.send_buffer.extend_from_slice(&compressed);

        let _ = self.socket.send_to(&self.send_buffer, &self.target_address);
    }

    fn send_config(&mut self) {
        self.compression_buffer.clear();

        for range in &self.entity_ranges {
            self.compression_buffer
                .extend_from_slice(&range.sextet_start.to_le_bytes());
            self.compression_buffer
                .extend_from_slice(&range.entity_start.to_le_bytes());
            self.compression_buffer
                .extend_from_slice(&range.sextet_end.to_le_bytes());
            self.compression_buffer
                .extend_from_slice(&range.entity_end.to_le_bytes());
        }

        let mut encoder = GzEncoder::new(
            Vec::with_capacity(self.compression_buffer.len()),
            Compression::fast(),
        );
        encoder.write_all(&self.compression_buffer).unwrap();
        let compressed = encoder.finish().unwrap();

        self.send_buffer.clear();
        self.send_buffer.extend_from_slice(b"iHuB");
        self.send_buffer.push(1); // Type: config
        self.send_buffer.push(self.universe);
        self.send_buffer
            .extend_from_slice(&(self.entity_ranges.len() as u16).to_le_bytes());
        self.send_buffer
            .extend_from_slice(&(compressed.len() as u16).to_le_bytes());
        self.send_buffer.extend_from_slice(&compressed);

        let _ = self.socket.send_to(&self.send_buffer, &self.target_address);
    }

    pub fn tick(&mut self) {
        if self.last_config_time.elapsed() >= Duration::from_secs(1) {
            self.send_config();
            self.last_config_time = Instant::now();
        }
    }

    pub fn set_differential_updates(&mut self, enabled: bool) {
        self.use_differential_updates = enabled;
    }
}

pub fn frame_to_entities_optimized(
    frame: &[u8],
    width: usize,
    height: usize,
    output: &mut Vec<(u16, u8, u8, u8, u8)>,
) {
    output.clear();
    output.reserve(64 * 259);

    const QUARTER_BASES: [u16; 4] = [100, 5100, 10100, 15100];

    for quarter in 0..4 {
        let base_entity = QUARTER_BASES[quarter];
        let quarter_x_offset = quarter * 32;

        for band in 0..16 {
            let col_start = quarter_x_offset + band * 2;
            let entity_base = base_entity + (band as u16) * 300;

            if col_start + 1 >= width {
                continue;
            }

            output.push((entity_base, 0, 0, 0, 0));

            let col1 = col_start;
            for i in 0..128 {
                let y = height.saturating_sub(1 + i);
                let pixel_idx = (y * width + col1) * 3;

                if pixel_idx + 2 < frame.len() {
                    unsafe {
                        output.push((
                            entity_base + 1 + (i as u16),
                            *frame.get_unchecked(pixel_idx),
                            *frame.get_unchecked(pixel_idx + 1),
                            *frame.get_unchecked(pixel_idx + 2),
                            0,
                        ));
                    }
                }
            }

            output.push((entity_base + 129, 0, 0, 0, 0));

            let col2 = col_start + 1;
            for i in 0..128 {
                let pixel_idx = (i * width + col2) * 3;

                if pixel_idx + 2 < frame.len() {
                    unsafe {
                        output.push((
                            entity_base + 130 + (i as u16),
                            *frame.get_unchecked(pixel_idx),
                            *frame.get_unchecked(pixel_idx + 1),
                            *frame.get_unchecked(pixel_idx + 2),
                            0,
                        ));
                    }
                }
            }

            output.push((entity_base + 258, 0, 0, 0, 0));
        }
    }
}

pub fn frame_to_entities(frame: &[u8], width: usize, height: usize) -> Vec<(u16, u8, u8, u8, u8)> {
    let mut entities = Vec::with_capacity(64 * 259);
    frame_to_entities_optimized(frame, width, height, &mut entities);
    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differential_updates() {
        let mut controller = IHubController::new("127.0.0.1:8080", 0).unwrap();

        let entities = vec![(1, 255, 0, 0, 0), (2, 0, 255, 0, 0), (3, 0, 0, 255, 0)];
        controller.update_entities(&entities);

        let updates = vec![(2, 0, 128, 0, 0)];
        controller.update_entities(&updates);

        assert_eq!(controller.dirty_entities.len(), 1);
        assert_eq!(controller.dirty_entities[0], 2);
    }

    #[test]
    fn test_frame_conversion_performance() {
        let frame = vec![0u8; 128 * 128 * 3];
        let mut output = Vec::new();

        let start = Instant::now();
        for _ in 0..100 {
            frame_to_entities_optimized(&frame, 128, 128, &mut output);
        }
        let duration = start.elapsed();

        assert!(duration.as_millis() < 100);
    }
}
