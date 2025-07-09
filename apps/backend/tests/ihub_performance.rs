use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;
use std::time::Instant;

// Mock structures pour les tests
#[derive(Clone, Copy)]
struct Entity {
    id: u16,
    r: u8,
    g: u8,
    b: u8,
    w: u8,
}

impl Entity {
    fn new_rgbw(id: u16, r: u8, g: u8, b: u8, w: u8) -> Self {
        Self { id, r, g, b, w }
    }

    fn to_sextet(&self) -> [u8; 6] {
        let id_bytes = self.id.to_le_bytes();
        [id_bytes[0], id_bytes[1], self.r, self.g, self.b, self.w]
    }
}

// Benchmark de la conversion frame vers entités
fn benchmark_frame_conversion(c: &mut Criterion) {
    let frame = vec![128u8; 128 * 128 * 3]; // Frame RGB 128x128

    c.bench_function("frame_to_entities_optimized", |b| {
        b.iter(|| {
            let mut output = Vec::with_capacity(64 * 259);
            frame_to_entities_optimized(black_box(&frame), 128, 128, &mut output);
        })
    });
}

// Benchmark de la compression
fn benchmark_compression(c: &mut Criterion) {
    let mut data = Vec::with_capacity(20000 * 6);
    for i in 0..20000 {
        let entity = Entity::new_rgbw(
            i,
            (i % 256) as u8,
            ((i * 2) % 256) as u8,
            ((i * 3) % 256) as u8,
            0,
        );
        data.extend_from_slice(&entity.to_sextet());
    }

    c.bench_function("gzip_compression_fast", |b| {
        b.iter(|| {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
            encoder.write_all(black_box(&data)).unwrap();
            encoder.finish().unwrap()
        })
    });

    c.bench_function("gzip_compression_default", |b| {
        b.iter(|| {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&data)).unwrap();
            encoder.finish().unwrap()
        })
    });
}

// Benchmark des mises à jour différentielles
fn benchmark_differential_updates(c: &mut Criterion) {
    let mut entities: HashMap<u16, Entity> = HashMap::with_capacity(20000);
    for i in 0..20000 {
        entities.insert(i, Entity::new_rgbw(i, 100, 100, 100, 0));
    }

    // Simuler 5% de changements
    let changes: Vec<(u16, u8, u8, u8, u8)> =
        (0..1000).map(|i| ((i * 20) as u16, 255, 0, 0, 0)).collect();

    c.bench_function("differential_update_5_percent", |b| {
        b.iter(|| {
            let mut dirty_entities = Vec::with_capacity(1000);
            for &(id, r, g, b, w) in &changes {
                if let Some(existing) = entities.get(&id) {
                    if existing.r != r || existing.g != g || existing.b != b || existing.w != w {
                        dirty_entities.push(id);
                    }
                }
            }
            black_box(dirty_entities);
        })
    });
}

// Benchmark du hash rapide
fn benchmark_fast_hash(c: &mut Criterion) {
    let data = vec![42u8; 12288]; // 64x64x3 bytes

    c.bench_function("fast_hash_64x64", |b| {
        b.iter(|| fast_hash(black_box(&data)))
    });

    let large_data = vec![42u8; 49152]; // 128x128x3 bytes

    c.bench_function("fast_hash_128x128", |b| {
        b.iter(|| fast_hash(black_box(&large_data)))
    });
}

// Benchmark de la sérialisation JSON
fn benchmark_json_serialization(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct WsMessage {
        #[serde(rename = "type")]
        msg_type: String,
        data: Vec<u8>,
    }

    let message = WsMessage {
        msg_type: "frame".to_string(),
        data: vec![128u8; 64 * 64 * 3],
    };

    c.bench_function("json_serialize_frame", |b| {
        b.iter(|| serde_json::to_string(black_box(&message)).unwrap())
    });

    let json_str = serde_json::to_string(&message).unwrap();

    c.bench_function("json_deserialize_frame", |b| {
        b.iter(|| {
            let _: WsMessage = serde_json::from_str(black_box(&json_str)).unwrap();
        })
    });
}

// Benchmark du protocole complet
fn benchmark_full_protocol(c: &mut Criterion) {
    let mut controller = MockIHubController::new();
    let frame = vec![128u8; 128 * 128 * 3];

    c.bench_function("full_protocol_pipeline", |b| {
        b.iter(|| {
            // Conversion frame -> entités
            let mut entities = Vec::with_capacity(64 * 259);
            frame_to_entities_optimized(&frame, 128, 128, &mut entities);

            // Mise à jour du contrôleur
            controller.update_entities(&entities);

            // Envoi (simulé)
            controller.send_update();
        })
    });
}

// Implémentations mockées pour les benchmarks
fn frame_to_entities_optimized(
    frame: &[u8],
    width: usize,
    height: usize,
    output: &mut Vec<(u16, u8, u8, u8, u8)>,
) {
    output.clear();
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

fn fast_hash(data: &[u8]) -> u64 {
    data.chunks(8).enumerate().fold(0u64, |acc, (i, chunk)| {
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        acc.wrapping_add(u64::from_le_bytes(bytes).wrapping_mul(i as u64 + 1))
    })
}

struct MockIHubController {
    entities: HashMap<u16, Entity>,
    send_buffer: Vec<u8>,
    compression_buffer: Vec<u8>,
}

impl MockIHubController {
    fn new() -> Self {
        Self {
            entities: HashMap::with_capacity(20000),
            send_buffer: Vec::with_capacity(65507),
            compression_buffer: Vec::with_capacity(32768),
        }
    }

    fn update_entities(&mut self, entities: &[(u16, u8, u8, u8, u8)]) {
        for &(id, r, g, b, w) in entities {
            self.entities.insert(id, Entity::new_rgbw(id, r, g, b, w));
        }
    }

    fn send_update(&mut self) {
        self.compression_buffer.clear();

        let mut sorted_entities: Vec<_> = self.entities.iter().collect();
        sorted_entities.sort_unstable_by_key(|(id, _)| *id);

        for (_, entity) in sorted_entities {
            self.compression_buffer
                .extend_from_slice(&entity.to_sextet());
        }

        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&self.compression_buffer).unwrap();
        let compressed = encoder.finish().unwrap();

        self.send_buffer.clear();
        self.send_buffer.extend_from_slice(b"iHuB");
        self.send_buffer.push(2);
        self.send_buffer.push(0);
        self.send_buffer
            .extend_from_slice(&(self.entities.len() as u16).to_le_bytes());
        self.send_buffer
            .extend_from_slice(&(compressed.len() as u16).to_le_bytes());
        self.send_buffer.extend_from_slice(&compressed);
    }
}

// Tests de performance supplémentaires
fn benchmark_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_allocation_patterns", |b| {
        b.iter(|| {
            // Test avec pré-allocation
            let mut vec_prealloc = Vec::with_capacity(20000);
            for i in 0..20000 {
                vec_prealloc.push(i as u16);
            }
            black_box(vec_prealloc);

            // Test sans pré-allocation
            let mut vec_no_prealloc = Vec::new();
            for i in 0..20000 {
                vec_no_prealloc.push(i as u16);
            }
            black_box(vec_no_prealloc);
        })
    });
}

fn benchmark_lookup_performance(c: &mut Criterion) {
    let mut lookup_table: Vec<(u16, u16, u16)> = Vec::with_capacity(100);
    for i in 0..100 {
        lookup_table.push((i * 100, (i + 1) * 100 - 1, i * 100));
    }

    c.bench_function("binary_search_lookup", |b| {
        b.iter(|| {
            for entity_id in (0..10000).step_by(100) {
                let result = lookup_table.binary_search_by(|&(start, end, _)| {
                    if entity_id < start {
                        std::cmp::Ordering::Greater
                    } else if entity_id > end {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
                black_box(result);
            }
        })
    });
}

// Configuration des benchmarks
criterion_group!(
    benches,
    benchmark_frame_conversion,
    benchmark_compression,
    benchmark_differential_updates,
    benchmark_fast_hash,
    benchmark_json_serialization,
    benchmark_full_protocol,
    benchmark_memory_usage,
    benchmark_lookup_performance
);

criterion_main!(benches);
