// src-tauri/src/effects/spectrum_bars.rs

use rayon::prelude::*;
use super::{Effect, get_color_config, hsv_to_rgb};

pub struct SpectrumBars {
    smoothed: Vec<f32>,
    peak_hold: Vec<f32>,
    peak_decay: Vec<f32>,
    debug_counter: u32,
}

impl SpectrumBars {
    pub fn new() -> Self {
        Self {
            smoothed: vec![0.0; 64],
            peak_hold: vec![0.0; 64],
            peak_decay: vec![0.0; 64],
            debug_counter: 0,
        }
    }

    fn get_color_for_bar(&self, bar: usize, brightness: f32) -> (f32, f32, f32) {
        let color_config = get_color_config();

        match color_config.mode.as_str() {
            "rainbow" => {
                let hue = (bar as f32 / 64.0) * 360.0;
                let saturation = 0.8 + if bar < self.smoothed.len() {
                    self.smoothed[bar] * 0.2
                } else {
                    0.0
                };
                hsv_to_rgb(hue / 360.0, saturation.min(1.0), brightness)
            }
            "fire" => {
                let hue = (bar as f32 / 64.0) * 60.0;
                let saturation = 1.0;
                hsv_to_rgb(hue / 360.0, saturation, brightness)
            }
            "ocean" => {
                let hue = 180.0 + (bar as f32 / 64.0) * 60.0;
                let saturation = 0.8 + if bar < self.smoothed.len() {
                    self.smoothed[bar] * 0.2
                } else {
                    0.0
                };
                hsv_to_rgb(hue / 360.0, saturation.min(1.0), brightness)
            }
            "sunset" => {
                let hue = if bar < 32 {
                    300.0 + (bar as f32 / 32.0) * 60.0
                } else {
                    (bar as f32 - 32.0) / 32.0 * 60.0
                };
                hsv_to_rgb(hue / 360.0, 1.0, brightness)
            }
            "custom" => {
                let (r, g, b) = color_config.custom_color;
                (r * brightness, g * brightness, b * brightness)
            }
            _ => {
                let hue = (bar as f32 / 64.0) * 360.0;
                hsv_to_rgb(hue / 360.0, 1.0, brightness)
            }
        }
    }
}

impl Effect for SpectrumBars {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Lissage et maintien des pics
        for i in 0..64 {
            let target = spectrum[i];
            let current = self.smoothed[i];

            if target > current {
                self.smoothed[i] = current * 0.4 + target * 0.6;
            } else {
                self.smoothed[i] = current * 0.85 + target * 0.15;
            }

            if self.smoothed[i] > self.peak_hold[i] {
                self.peak_hold[i] = self.smoothed[i];
                self.peak_decay[i] = 0.0;
            } else {
                self.peak_decay[i] += 0.02;
                self.peak_hold[i] = (self.peak_hold[i] - self.peak_decay[i]).max(0.0);
            }
        }

        // Debug périodique
        self.debug_counter += 1;
        if self.debug_counter % 50 == 0 {
            let _max_level = self.smoothed.iter().cloned().fold(0.0f32, f32::max);
        }

        // Effacer la frame
        frame.fill(0);

        // Rendu parallélisé
        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i % 128) as f32;
            let y = (i / 128) as f32;

            // Déterminer quelle barre correspond à cette position x
            let bar = if x < 64.0 {
                (x * 32.0 / 64.0) as usize
            } else {
                (31.0 - (x - 64.0) * 32.0 / 64.0) as usize
            };

            if bar < 32 {
                let value = self.smoothed[bar.min(31)];
                let curved_value = if value > 0.0 { value.powf(0.6) } else { 0.0 };

                let height = curved_value * 120.0;
                let peak_height = self.peak_hold[bar] * 120.0;

                let bar_bottom = 128.0 - height;
                let distance_from_bottom = (y - bar_bottom).max(0.0);
                let gradient_factor = if y >= bar_bottom && y < 128.0 {
                    1.0 - (distance_from_bottom / height).min(1.0) * 0.3
                } else {
                    0.0
                };

                // Dessiner la barre principale
                if y >= bar_bottom && y < 128.0 {
                    let brightness = gradient_factor;
                    let (r, g, b) = self.get_color_for_bar(bar, brightness);
                    pixel[0] = (r * 255.0) as u8;
                    pixel[1] = (g * 255.0) as u8;
                    pixel[2] = (b * 255.0) as u8;
                }

                // Dessiner l'indicateur de pic
                let peak_y = 128.0 - peak_height;
                if (y - peak_y).abs() < 1.0 && peak_height > 5.0 {
                    let (r, g, b) = self.get_color_for_bar(bar, 0.8);
                    pixel[0] = (r * 255.0 * 0.8) as u8;
                    pixel[1] = (g * 255.0 * 0.8) as u8;
                    pixel[2] = (b * 255.0 * 0.8) as u8;
                }

                // Ajouter des bordures entre les barres
                let bar_pos = if x < 64.0 {
                    x * 32.0 / 64.0
                } else {
                    31.0 - (x - 64.0) * 32.0 / 64.0
                };
                let bar_boundary = (bar_pos - bar as f32).abs() * 64.0 / 32.0;
                if bar_boundary > 1.8 && y >= bar_bottom && y < 128.0 {
                    pixel[0] = (pixel[0] as f32 * 0.7) as u8;
                    pixel[1] = (pixel[1] as f32 * 0.7) as u8;
                    pixel[2] = (pixel[2] as f32 * 0.7) as u8;
                }

                // Ligne centrale
                if (x - 64.0).abs() < 0.5 && y >= bar_bottom && y < 128.0 {
                    let (r, g, b) = self.get_color_for_bar(bar, 0.3);
                    pixel[0] = (r * 255.0) as u8;
                    pixel[1] = (g * 255.0) as u8;
                    pixel[2] = (b * 255.0) as u8;
                }
            }
        });
    }

    fn set_color_mode(&mut self, _mode: &str) {
        // La configuration couleur est gérée globalement
    }

    fn set_custom_color(&mut self, _r: f32, _g: f32, _b: f32) {
        // La configuration couleur est gérée globalement
    }

    fn name(&self) -> &'static str {
        "Spectrum Bars"
    }

    fn description(&self) -> &'static str {
        "Classic spectrum analyzer with animated bars and peak hold indicators"
    }

    fn reset(&mut self) {
        self.smoothed.fill(0.0);
        self.peak_hold.fill(0.0);
        self.peak_decay.fill(0.0);
        self.debug_counter = 0;
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectrum_bars_creation() {
        let bars = SpectrumBars::new();
        assert_eq!(bars.smoothed.len(), 64);
        assert_eq!(bars.peak_hold.len(), 64);
        assert_eq!(bars.peak_decay.len(), 64);
        assert_eq!(bars.debug_counter, 0);
    }

    #[test]
    fn test_spectrum_bars_reset() {
        let mut bars = SpectrumBars::new();

        // Simuler quelques valeurs
        bars.smoothed[0] = 0.5;
        bars.peak_hold[0] = 0.8;
        bars.debug_counter = 100;

        bars.reset();

        assert_eq!(bars.smoothed[0], 0.0);
        assert_eq!(bars.peak_hold[0], 0.0);
        assert_eq!(bars.debug_counter, 0);
    }

    #[test]
    fn test_spectrum_bars_render() {
        let mut bars = SpectrumBars::new();
        let spectrum = vec![0.5; 64];
        let mut frame = vec![0u8; 128 * 128 * 3];

        bars.render(&spectrum, &mut frame);

        // Vérifier que la frame n'est pas complètement noire
        let non_zero = frame.iter().filter(|&&x| x > 0).count();
        assert!(non_zero > 0);
    }

    #[test]
    fn test_color_for_bar() {
        let bars = SpectrumBars::new();

        // Test avec différentes valeurs
        let (r, g, b) = bars.get_color_for_bar(0, 1.0);
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);

        // Test avec brightness nulle
        let (r, g, b) = bars.get_color_for_bar(0, 0.0);
        assert_eq!(r, 0.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
    }
}
