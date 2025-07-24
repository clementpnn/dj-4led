// src-tauri/src/effects/circular_wave.rs

use rayon::prelude::*;
use std::f32::consts::PI;
use super::{Effect, get_color_config, hsv_to_rgb};

pub struct CircularWave {
    time: f32,
}

impl CircularWave {
    pub fn new() -> Self {
        Self { time: 0.0 }
    }

    fn get_color_for_wave(
        &self,
        angle: f32,
        dist: f32,
        brightness: f32,
        bass_energy: f32,
        mid_energy: f32,
    ) -> (f32, f32, f32) {
        let color_config = get_color_config();

        match color_config.mode.as_str() {
            "rainbow" => {
                let hue_shift = bass_energy * 0.2;
                let hue = (angle + PI) / (2.0 * PI) + self.time * 0.1 + hue_shift;
                let saturation = 0.9 + (bass_energy + mid_energy) * 0.1;
                hsv_to_rgb(hue % 1.0, saturation.min(1.0), brightness)
            }
            "fire" => {
                let hue = (self.time * 0.05 + dist * 0.1) % 1.0 * 60.0 / 360.0;
                let saturation = 0.8 + bass_energy * 0.2;
                hsv_to_rgb(hue, saturation.min(1.0), brightness)
            }
            "ocean" => {
                let hue = (180.0 + (self.time * 0.03 + angle * 30.0).sin() * 40.0) / 360.0;
                let saturation = 0.7 + mid_energy * 0.3;
                hsv_to_rgb(hue, saturation.min(1.0), brightness)
            }
            "sunset" => {
                let progress = (angle + PI) / (2.0 * PI);
                let hue = if progress < 0.5 {
                    300.0 / 360.0 + progress * 120.0 / 360.0
                } else {
                    60.0 / 360.0 * (1.0 - (progress - 0.5) * 2.0)
                };
                hsv_to_rgb(hue, 1.0, brightness)
            }
            "custom" => {
                let (r, g, b) = color_config.custom_color;
                let modulation = 1.0 + (self.time * 0.1 + dist * 0.5).sin() * 0.3;
                (
                    (r * brightness * modulation).min(1.0),
                    (g * brightness * modulation).min(1.0),
                    (b * brightness * modulation).min(1.0),
                )
            }
            _ => hsv_to_rgb(0.5, 1.0, brightness),
        }
    }
}

impl Effect for CircularWave {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        let total_energy = spectrum.iter().sum::<f32>() / spectrum.len() as f32;
        self.time += 0.05 + total_energy * 0.2;

        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;

        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = ((i % 128) as f32 - 64.0) / 64.0;
            let y = ((i / 128) as f32 - 64.0) / 64.0;

            let dist = (x * x + y * y).sqrt();
            let angle = y.atan2(x);

            let speed_mod = 1.0 + bass_energy * 3.0;

            // Trois ondes concentriques avec différentes fréquences
            let wave1 = ((dist * 20.0 - self.time * 8.0 * speed_mod).sin() + 1.0) / 2.0;
            let wave2 = ((dist * 10.0 - self.time * 4.0 * speed_mod).cos() + 1.0) / 2.0;
            let wave3 = ((dist * 5.0 - self.time * 2.0 * speed_mod).sin() + 1.0) / 2.0;

            // Intensité de base plus intensité audio
            let base_intensity = 0.3;
            let audio_intensity =
                wave1 * bass_energy * 2.0 + wave2 * mid_energy * 1.5 + wave3 * high_energy;

            let intensity = (base_intensity + audio_intensity).min(1.0);

            // Pattern d'onde combiné
            let wave_pattern = (wave1 * 0.4 + wave2 * 0.3 + wave3 * 0.3).min(1.0);
            let brightness = (base_intensity + intensity * wave_pattern).min(1.0);

            let (r, g, b) =
                self.get_color_for_wave(angle, dist, brightness, bass_energy, mid_energy);

            pixel[0] = (r * 255.0) as u8;
            pixel[1] = (g * 255.0) as u8;
            pixel[2] = (b * 255.0) as u8;
        });
    }

    fn set_color_mode(&mut self, _mode: &str) {
        // La configuration couleur est gérée globalement
    }

    fn set_custom_color(&mut self, _r: f32, _g: f32, _b: f32) {
        // La configuration couleur est gérée globalement
    }

    fn name(&self) -> &'static str {
        "Circular Wave"
    }

    fn description(&self) -> &'static str {
        "Concentric waves emanating from the center, responding to audio frequencies"
    }

    fn reset(&mut self) {
        self.time = 0.0;
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_wave_creation() {
        let wave = CircularWave::new();
        assert_eq!(wave.time, 0.0);
    }

    #[test]
    fn test_circular_wave_reset() {
        let mut wave = CircularWave::new();
        wave.time = 100.0;

        wave.reset();
        assert_eq!(wave.time, 0.0);
    }

    #[test]
    fn test_circular_wave_render() {
        let mut wave = CircularWave::new();
        let spectrum = vec![0.5; 64];
        let mut frame = vec![0u8; 128 * 128 * 3];

        wave.render(&spectrum, &mut frame);

        // Vérifier que le temps a avancé
        assert!(wave.time > 0.0);

        // Vérifier que la frame n'est pas complètement noire
        let non_zero = frame.iter().filter(|&&x| x > 0).count();
        assert!(non_zero > 0);
    }

    #[test]
    fn test_color_for_wave() {
        let wave = CircularWave::new();

        let (r, g, b) = wave.get_color_for_wave(0.0, 0.5, 1.0, 0.3, 0.3);
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);
    }
}
