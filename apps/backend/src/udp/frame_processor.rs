use super::protocol::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

pub struct UdpFrameProcessor {
    frame_buffer: Vec<u8>,
    compression_buffer: Vec<u8>,
    last_frame_hash: u64,
    last_spectrum_hash: u64,
    frame_counter: u32,
}

impl UdpFrameProcessor {
    pub fn new() -> Self {
        Self {
            frame_buffer: Vec::with_capacity(128 * 128 * 3),
            compression_buffer: Vec::with_capacity(64 * 1024),
            last_frame_hash: 0,
            last_spectrum_hash: 0,
            frame_counter: 0,
        }
    }

    pub fn prepare_packets(
        &mut self,
        frame: &[u8],
        spectrum: &[f32],
        sequence_base: u32,
        use_compression: bool,
    ) -> Vec<UdpPacket> {
        let mut packets = Vec::new();
        let mut current_sequence = sequence_base;

        // Traiter la frame si elle a changé
        let frame_hash = Self::fast_hash(frame);
        if frame_hash != self.last_frame_hash || self.frame_counter % 60 == 0 {
            self.last_frame_hash = frame_hash;

            // Downscale à 64x64 pour réduire la bande passante
            self.downscale_frame(frame, 128, 64, 64);

            let frame_data = FrameData {
                width: 64,
                height: 64,
                format: FrameFormat::RGB,
                data: self.frame_buffer.clone(),
            };

            let payload = frame_data.to_payload();

            // Essayer la compression si activée et utile
            let (final_payload, packet_type) = if use_compression && payload.len() > 1024 {
                if let Some(compressed) = self.compress_data(&payload) {
                    if compressed.len() < payload.len() * 3 / 4 {
                        (compressed, PacketType::FrameDataCompressed)
                    } else {
                        (payload, PacketType::FrameData)
                    }
                } else {
                    (payload, PacketType::FrameData)
                }
            } else {
                (payload, PacketType::FrameData)
            };

            // Fragmenter si nécessaire
            if final_payload.len() <= MAX_PACKET_SIZE - 12 {
                // Un seul paquet
                packets.push(UdpPacket::new(packet_type, current_sequence, final_payload));
                current_sequence = current_sequence.wrapping_add(1);
            } else {
                // Fragmenter en plusieurs paquets
                let chunk_size = MAX_PACKET_SIZE - 12;
                let chunks: Vec<_> = final_payload.chunks(chunk_size).collect();
                let fragment_count = chunks.len() as u16;

                for (i, chunk) in chunks.iter().enumerate() {
                    let mut packet = UdpPacket::new(packet_type, current_sequence, chunk.to_vec());

                    packet.flags |= PacketFlags::FRAGMENTED;
                    packet.fragment_id = i as u16;
                    packet.fragment_count = fragment_count;

                    if i == chunks.len() - 1 {
                        packet.flags |= PacketFlags::LAST_FRAGMENT;
                    }

                    packets.push(packet);
                    current_sequence = current_sequence.wrapping_add(1);
                }
            }
        }

        // Traiter le spectre si il a changé
        let spectrum_hash = Self::fast_hash_f32(spectrum);
        if spectrum_hash != self.last_spectrum_hash {
            self.last_spectrum_hash = spectrum_hash;

            // Réduire le spectre à 32 bandes pour économiser la bande passante
            let reduced_spectrum = Self::reduce_spectrum(spectrum, 32);

            let spectrum_data = SpectrumData {
                bands: reduced_spectrum,
            };

            let payload = spectrum_data.to_payload();
            packets.push(UdpPacket::new(
                PacketType::SpectrumData,
                current_sequence,
                payload,
            ));
        }

        self.frame_counter = self.frame_counter.wrapping_add(1);
        packets
    }

    // Hash rapide pour détecter les changements
    fn fast_hash(data: &[u8]) -> u64 {
        data.chunks(8).enumerate().fold(0u64, |acc, (i, chunk)| {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            acc.wrapping_add(u64::from_le_bytes(bytes).wrapping_mul(i as u64 + 1))
        })
    }

    fn fast_hash_f32(data: &[f32]) -> u64 {
        data.iter().enumerate().fold(0u64, |acc, (i, &value)| {
            let bits = value.to_bits() as u64;
            acc.wrapping_add(bits.wrapping_mul(i as u64 + 1))
        })
    }

    // Downscale optimisé pour réduire la résolution
    fn downscale_frame(
        &mut self,
        src: &[u8],
        src_width: usize,
        dst_width: usize,
        dst_height: usize,
    ) {
        self.frame_buffer.clear();
        let scale = src_width / dst_width;

        // Version optimisée avec moins d'accès mémoire
        self.frame_buffer.reserve_exact(dst_width * dst_height * 3);

        for y in 0..dst_height {
            let src_y = y * scale;
            for x in 0..dst_width {
                let src_x = x * scale;
                let src_idx = (src_y * src_width + src_x) * 3;

                if src_idx + 2 < src.len() {
                    unsafe {
                        // Utilisation de pointeurs pour éviter les bounds checks
                        let ptr = src.as_ptr().add(src_idx);
                        self.frame_buffer.push(*ptr);
                        self.frame_buffer.push(*ptr.add(1));
                        self.frame_buffer.push(*ptr.add(2));
                    }
                } else {
                    self.frame_buffer.extend_from_slice(&[0, 0, 0]);
                }
            }
        }
    }

    // Compression rapide avec gzip
    fn compress_data(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        self.compression_buffer.clear();

        let mut encoder = GzEncoder::new(&mut self.compression_buffer, Compression::fast());

        if encoder.write_all(data).is_ok() && encoder.finish().is_ok() {
            Some(self.compression_buffer.clone())
        } else {
            None
        }
    }

    // Réduction du spectre audio
    fn reduce_spectrum(spectrum: &[f32], target_bands: usize) -> Vec<f32> {
        if spectrum.len() <= target_bands {
            return spectrum.to_vec();
        }

        let mut reduced = Vec::with_capacity(target_bands);
        let bin_size = spectrum.len() / target_bands;

        for i in 0..target_bands {
            let start = i * bin_size;
            let end = if i == target_bands - 1 {
                spectrum.len()
            } else {
                (i + 1) * bin_size
            };

            // Moyenne des valeurs dans chaque bande
            let sum: f32 = spectrum[start..end].iter().sum();
            let avg = sum / (end - start) as f32;

            // Arrondir à 2 décimales pour réduire la taille
            reduced.push((avg * 100.0).round() / 100.0);
        }

        reduced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downscale() {
        let mut processor = UdpFrameProcessor::new();
        let src = vec![255u8; 128 * 128 * 3];

        processor.downscale_frame(&src, 128, 64, 64);

        assert_eq!(processor.frame_buffer.len(), 64 * 64 * 3);
        assert!(processor.frame_buffer.iter().all(|&x| x == 255));
    }

    #[test]
    fn test_reduce_spectrum() {
        let spectrum: Vec<f32> = (0..128).map(|i| i as f32).collect();
        let reduced = UdpFrameProcessor::reduce_spectrum(&spectrum, 32);

        assert_eq!(reduced.len(), 32);
        // Vérifier que les valeurs sont moyennées correctement
        assert_eq!(reduced[0], 1.5); // Moyenne de 0,1,2,3
        assert_eq!(reduced[1], 5.5); // Moyenne de 4,5,6,7
    }

    #[test]
    fn test_compression() {
        let mut processor = UdpFrameProcessor::new();
        let data = vec![0u8; 1024]; // Données très compressibles

        let compressed = processor.compress_data(&data);
        assert!(compressed.is_some());

        let compressed_data = compressed.unwrap();
        assert!(compressed_data.len() < data.len());
    }
}
