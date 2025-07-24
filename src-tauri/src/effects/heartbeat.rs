// src-tauri/src/effects/heartbeat.rs

use super::{Effect, get_color_config, hsv_to_rgb, rand};

pub struct Heartbeat {
    beat_phase: f32,
    beat_intensity: f32,
    heart_size: f32,
    pulse_rings: Vec<PulseRing>,
    animation_time: f32,
    last_beat_time: f32,
    beat_frequency: f32,
}

struct PulseRing {
    radius: f32,
    life: f32,
    intensity: f32,
    color: (f32, f32, f32),
}

impl Heartbeat {
    pub fn new() -> Self {
        Self {
            beat_phase: 0.0,
            beat_intensity: 0.5,
            heart_size: 20.0,
            pulse_rings: Vec::new(),
            animation_time: 0.0,
            last_beat_time: 0.0,
            beat_frequency: 60.0,
        }
    }

    fn get_heart_color(&self, intensity: f32) -> (f32, f32, f32) {
        let color_config = get_color_config();

        match color_config.mode.as_str() {
            "rainbow" => {
                let hue = (self.animation_time * 0.01) % 1.0;
                hsv_to_rgb(hue, 0.8, intensity)
            }
            "fire" => {
                if intensity < 0.5 {
                    (intensity * 2.0, 0.0, 0.0)
                } else {
                    (1.0, (intensity - 0.5) * 2.0, 0.0)
                }
            }
            "ocean" => (intensity * 0.2, intensity * 0.6, intensity),
            "sunset" => (intensity, intensity * 0.4, intensity * 0.6),
            "custom" => {
                let (r, g, b) = color_config.custom_color;
                (r * intensity, g * intensity, b * intensity)
            }
            _ => (intensity, intensity * 0.2, intensity * 0.2),
        }
    }

    fn is_inside_heart(&self, x: f32, y: f32, center_x: f32, center_y: f32, scale: f32) -> f32 {
        let nx = (x - center_x) / scale;
        let ny = -(y - center_y) / scale;

        let x2 = nx * nx;
        let y2 = ny * ny;
        let heart_eq = (x2 + y2 - 1.0).powi(3) - x2 * ny.powi(3);

        if heart_eq <= 0.0 {
            let distance_factor = (-heart_eq).min(0.5) / 0.5;
            return distance_factor.max(0.3);
        }

        0.0
    }
}

impl Effect for Heartbeat {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;

        let sensitivity = 5.0;
        let bass = (bass_energy * sensitivity).min(1.0);
        let mid = (mid_energy * sensitivity).min(1.0);
        let high = (high_energy * sensitivity).min(1.0);

        let total_energy = (bass * 0.6 + mid * 0.3 + high * 0.1).min(1.0);

        // Ajuster la fréquence cardiaque selon l'énergie audio
        self.beat_frequency = 40.0 + total_energy * 100.0;
        let beat_interval = 60.0 / self.beat_frequency;

        self.animation_time += 1.0 / 60.0;

        // Créer un nouveau battement si c'est le moment
        if self.animation_time - self.last_beat_time >= beat_interval {
            self.last_beat_time = self.animation_time;
            self.beat_phase = 0.0;

            // Ajouter un anneau de pulsation
            self.pulse_rings.push(PulseRing {
                radius: 0.0,
                life: 1.0,
                intensity: 0.3 + total_energy * 0.7,
                color: self.get_heart_color(1.0),
            });
        }

        self.beat_phase += (self.beat_frequency / 60.0) * 0.15;

        // Double battement cardiaque
        let double_beat = if self.beat_phase % 1.0 < 0.3 {
            ((self.beat_phase % 1.0) * 10.0).sin().max(0.0)
        } else if self.beat_phase % 1.0 < 0.5 {
            (((self.beat_phase % 1.0) - 0.3) * 15.0).sin().max(0.0) * 0.6
        } else {
            0.2
        };

        self.beat_intensity = 0.4 + double_beat * (0.3 + total_energy * 0.3);

        // Taille du cœur basée sur l'audio
        let base_size = 15.0 + total_energy * 25.0;
        self.heart_size = base_size * (0.8 + self.beat_intensity * 0.4);

        // Mettre à jour les anneaux de pulsation
        self.pulse_rings.retain_mut(|ring| {
            ring.radius += 2.0 + total_energy * 3.0;
            ring.life -= 0.02;
            ring.intensity *= 0.98;
            ring.life > 0.0 && ring.radius < 100.0
        });

        // Effacer la frame
        frame.fill(0);

        let center_x = 64.0;
        let center_y = 64.0;

        // Dessiner les anneaux de pulsation
        for ring in &self.pulse_rings {
            for angle_step in 0..64 {
                let angle = (angle_step as f32 / 64.0) * 6.28;
                let x = center_x + angle.cos() * ring.radius;
                let y = center_y + angle.sin() * ring.radius;

                if x >= 0.0 && x < 128.0 && y >= 0.0 && y < 128.0 {
                    let px = x as usize;
                    let py = y as usize;
                    let idx = (py * 128 + px) * 3;

                    if idx + 2 < frame.len() {
                        let ring_intensity = ring.intensity * ring.life * 0.3;

                        frame[idx] = ((ring.color.0 * ring_intensity * 255.0) as u8)
                            .saturating_add(frame[idx]);
                        frame[idx + 1] = ((ring.color.1 * ring_intensity * 255.0) as u8)
                            .saturating_add(frame[idx + 1]);
                        frame[idx + 2] = ((ring.color.2 * ring_intensity * 255.0) as u8)
                            .saturating_add(frame[idx + 2]);
                    }
                }
            }
        }

        // Dessiner le cœur principal
        for y in 0..128 {
            for x in 0..128 {
                let heart_intensity =
                    self.is_inside_heart(x as f32, y as f32, center_x, center_y, self.heart_size);

                if heart_intensity > 0.0 {
                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        let final_intensity = heart_intensity * self.beat_intensity;

                        // Effet de lueur au centre
                        let distance_from_center =
                            ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                        let center_glow =
                            (1.0 - (distance_from_center / self.heart_size).min(1.0)) * 0.3 + 0.7;

                        let (r, g, b) = self.get_heart_color(final_intensity * center_glow);

                        // Pulsation lumineuse
                        let pulse_glow = 1.0 + (self.beat_phase * 12.56).sin() * total_energy * 0.3;

                        frame[idx] = (r * pulse_glow * 255.0).min(255.0) as u8;
                        frame[idx + 1] = (g * pulse_glow * 255.0).min(255.0) as u8;
                        frame[idx + 2] = (b * pulse_glow * 255.0).min(255.0) as u8;
                    }
                }
            }
        }

        // Étincelles autour du cœur lors de battements intenses
        if self.beat_intensity > 0.8 && total_energy > 0.5 {
            let sparkle_count = (total_energy * 15.0) as usize;

            for _ in 0..sparkle_count {
                let angle = rand() * 6.28;
                let distance = self.heart_size + 5.0 + rand() * 15.0;

                let sparkle_x = center_x + angle.cos() * distance;
                let sparkle_y = center_y + angle.sin() * distance;

                if sparkle_x >= 0.0 && sparkle_x < 128.0 && sparkle_y >= 0.0 && sparkle_y < 128.0 {
                    let px = sparkle_x as usize;
                    let py = sparkle_y as usize;
                    let idx = (py * 128 + px) * 3;

                    if idx + 2 < frame.len() {
                        let sparkle_intensity = 0.5 + rand() * 0.5;
                        let (r, g, b) = self.get_heart_color(sparkle_intensity);

                        frame[idx] = ((r * 200.0) as u8).saturating_add(frame[idx]);
                        frame[idx + 1] = ((g * 200.0) as u8).saturating_add(frame[idx + 1]);
                        frame[idx + 2] = ((b * 200.0) as u8).saturating_add(frame[idx + 2]);
                    }
                }
            }
        }

        // Flash lors de battements très intenses
        if self.beat_intensity > 0.9 && total_energy > 0.7 {
            let flash_intensity = (self.beat_intensity - 0.9) * 10.0 * total_energy;
            let (flash_r, flash_g, flash_b) = self.get_heart_color(1.0);

            for y in (center_y as usize).saturating_sub(50)..((center_y as usize + 50).min(128)) {
                for x in (center_x as usize).saturating_sub(50)..((center_x as usize + 50).min(128)) {
                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        let distance =
                            ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                        let flash_falloff = (1.0 - distance / 50.0).max(0.0);
                        let flash_add = flash_intensity * flash_falloff * 50.0;

                        frame[idx] = ((flash_r * flash_add) as u8).saturating_add(frame[idx]);
                        frame[idx + 1] = ((flash_g * flash_add) as u8).saturating_add(frame[idx + 1]);
                        frame[idx + 2] = ((flash_b * flash_add) as u8).saturating_add(frame[idx + 2]);
                    }
                }
            }
        }
    }

    fn set_color_mode(&mut self, _mode: &str) {
        // La configuration couleur est gérée globalement
    }

    fn set_custom_color(&mut self, _r: f32, _g: f32, _b: f32) {
        // La configuration couleur est gérée globalement
    }

    fn name(&self) -> &'static str {
        "Heartbeat"
    }

    fn description(&self) -> &'static str {
        "Animated heart with pulsating rings that beat to the rhythm of the music"
    }

    fn reset(&mut self) {
        self.beat_phase = 0.0;
        self.beat_intensity = 0.5;
        self.heart_size = 20.0;
        self.pulse_rings.clear();
        self.animation_time = 0.0;
        self.last_beat_time = 0.0;
        self.beat_frequency = 60.0;
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_creation() {
        let heartbeat = Heartbeat::new();
        assert_eq!(heartbeat.beat_phase, 0.0);
        assert_eq!(heartbeat.pulse_rings.len(), 0);
        assert_eq!(heartbeat.beat_frequency, 60.0);
    }

    #[test]
    fn test_heartbeat_reset() {
        let mut heartbeat = Heartbeat::new();
        heartbeat.beat_phase = 1.0;
        heartbeat.animation_time = 100.0;
        heartbeat.pulse_rings.push(PulseRing {
            radius: 10.0,
            life: 0.5,
            intensity: 0.8,
            color: (1.0, 0.0, 0.0),
        });

        heartbeat.reset();

        assert_eq!(heartbeat.beat_phase, 0.0);
        assert_eq!(heartbeat.animation_time, 0.0);
        assert_eq!(heartbeat.pulse_rings.len(), 0);
    }

    #[test]
    fn test_is_inside_heart() {
        let heartbeat = Heartbeat::new();

        // Point au centre du cœur
        let intensity = heartbeat.is_inside_heart(64.0, 64.0, 64.0, 64.0, 20.0);
        assert!(intensity > 0.0);

        // Point très loin du cœur
        let intensity = heartbeat.is_inside_heart(0.0, 0.0, 64.0, 64.0, 20.0);
        assert_eq!(intensity, 0.0);
    }
}
