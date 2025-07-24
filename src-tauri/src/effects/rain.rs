// src-tauri/src/effects/rain.rs

use super::{Effect, get_color_config, hsv_to_rgb, rand};

pub struct Rain {
    drops: Vec<RainDrop>,
    animation_counter: f32,
}

struct RainDrop {
    x: f32,
    y: f32,
    length: f32,
    speed: f32,
    brightness: f32,
}

impl Rain {
    pub fn new() -> Self {
        let mut drops = Vec::with_capacity(100);

        // Créer quelques gouttes initiales
        for _ in 0..50 {
            drops.push(RainDrop {
                x: rand() * 128.0,
                y: rand() * 128.0,
                length: 3.0 + rand() * 10.0,
                speed: 1.0 + rand() * 3.0,
                brightness: 0.3 + rand() * 0.7,
            });
        }

        Self {
            drops,
            animation_counter: 0.0,
        }
    }

    fn get_rain_color(&self, brightness: f32, y_pos: f32) -> (f32, f32, f32) {
        let color_config = get_color_config();

        match color_config.mode.as_str() {
            "custom" => {
                let (r, g, b) = color_config.custom_color;
                (r * brightness, g * brightness, b * brightness)
            }
            "rainbow" => {
                let hue = (y_pos / 128.0) * 360.0;
                let (r, g, b) = hsv_to_rgb(hue / 360.0, 0.7, brightness);
                (r, g, b)
            }
            "fire" => {
                let r = brightness;
                let g = brightness * 0.5;
                let b = brightness * 0.1;
                (r, g, b)
            }
            "ocean" => {
                let r = brightness * 0.1;
                let g = brightness * 0.5;
                let b = brightness;
                (r, g, b)
            }
            "sunset" => {
                let r = brightness;
                let g = brightness * 0.6;
                let b = brightness * 0.8;
                (r, g, b)
            }
            "matrix" => {
                // Vert style Matrix
                let r = brightness * 0.1;
                let g = brightness;
                let b = brightness * 0.1;
                (r, g, b)
            }
            _ => {
                // Mode océan par défaut
                let r = brightness * 0.1;
                let g = brightness * 0.5;
                let b = brightness;
                (r, g, b)
            }
        }
    }
}

impl Effect for Rain {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;

        // Augmenter la sensibilité pour rendre l'effet plus réactif
        let sensitivity = 4.0;
        let bass_energy = (bass_energy * sensitivity).min(1.0);
        let mid_energy = (mid_energy * sensitivity).min(1.0);
        let high_energy = (high_energy * sensitivity).min(1.0);

        let total_energy = (bass_energy * 0.5 + mid_energy * 0.3 + high_energy * 0.2).min(1.0);

        // Effacer la frame
        for pixel in frame.chunks_exact_mut(3) {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        }

        self.animation_counter += 0.1;

        // Générer de nouvelles gouttes basées sur l'audio
        let drop_chance = 0.2 + total_energy * 0.5;
        if rand() < drop_chance {
            let num_new_drops = (1.0 + total_energy * 5.0) as usize;

            for _ in 0..num_new_drops {
                if self.drops.len() < 200 {
                    self.drops.push(RainDrop {
                        x: rand() * 128.0,
                        y: -10.0 - rand() * 10.0,
                        length: 3.0 + rand() * 12.0 + total_energy * 10.0,
                        speed: 1.0 + rand() * 2.0 + total_energy * 3.0,
                        brightness: 0.3 + rand() * 0.5 + total_energy * 0.2,
                    });
                }
            }
        }

        // Mettre à jour et dessiner les gouttes
        let mut i = 0;
        while i < self.drops.len() {
            {
                let drop = &mut self.drops[i];
                drop.y += drop.speed;

                // Effet de vent basé sur les fréquences moyennes
                let wind_effect = (self.animation_counter * 0.05).sin() * mid_energy * 0.5;
                drop.x += wind_effect;

                // Garder les gouttes dans l'écran horizontalement
                if drop.x < 0.0 {
                    drop.x = 0.0;
                } else if drop.x >= 128.0 {
                    drop.x = 127.9;
                }
            }

            let drop = &self.drops[i];
            let x = drop.x as usize;
            let start_y = (drop.y - drop.length).max(0.0) as usize;
            let end_y = drop.y.min(127.0) as usize;

            // Dessiner la goutte comme une ligne verticale
            for y in start_y..=end_y {
                if y < 128 {
                    let relative_pos =
                        (y as f32 - start_y as f32) / (end_y as f32 - start_y as f32 + 1.0);
                    let brightness_factor = drop.brightness * (0.5 + relative_pos * 0.5);

                    let (r, g, b) = self.get_rain_color(brightness_factor, y as f32);

                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        frame[idx] = (r * 255.0) as u8;
                        frame[idx + 1] = (g * 255.0) as u8;
                        frame[idx + 2] = (b * 255.0) as u8;
                    }
                }
            }

            // Supprimer les gouttes qui sont sorties de l'écran
            if drop.y - drop.length > 128.0 {
                self.drops.swap_remove(i);
            } else {
                i += 1;
            }
        }

        // Effet d'éclaboussures au sol lors d'intensité élevée
        if total_energy > 0.3 {
            for _i in 0..5 {
                let splash_x = rand() * 128.0;
                let splash_y = 127.0 - rand() * 3.0;
                let splash_size = 1.0 + rand() * 2.0 + total_energy * 3.0;

                for dx in -splash_size as i32..=splash_size as i32 {
                    let x = (splash_x as i32 + dx).max(0).min(127) as usize;
                    let y = splash_y as usize;

                    let brightness = (1.0 - (dx.abs() as f32 / splash_size)) * total_energy;
                    let (r, g, b) = self.get_rain_color(brightness, y as f32);

                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        frame[idx] = (r * 255.0) as u8;
                        frame[idx + 1] = (g * 255.0) as u8;
                        frame[idx + 2] = (b * 255.0) as u8;
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
        "Rain"
    }

    fn description(&self) -> &'static str {
        "Animated rain drops falling with wind effects and splashes"
    }

    fn reset(&mut self) {
        self.drops.clear();
        self.animation_counter = 0.0;

        // Recréer quelques gouttes initiales
        for _ in 0..50 {
            self.drops.push(RainDrop {
                x: rand() * 128.0,
                y: rand() * 128.0,
                length: 3.0 + rand() * 10.0,
                speed: 1.0 + rand() * 3.0,
                brightness: 0.3 + rand() * 0.7,
            });
        }
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rain_creation() {
        let rain = Rain::new();
        assert_eq!(rain.drops.len(), 50);
        assert_eq!(rain.animation_counter, 0.0);
    }

    #[test]
    fn test_rain_reset() {
        let mut rain = Rain::new();
        rain.animation_counter = 100.0;
        rain.drops.clear();

        rain.reset();

        assert_eq!(rain.drops.len(), 50);
        assert_eq!(rain.animation_counter, 0.0);
    }

    #[test]
    fn test_rain_drop_creation() {
        let drop = RainDrop {
            x: 50.0,
            y: 10.0,
            length: 5.0,
            speed: 2.0,
            brightness: 0.8,
        };

        assert_eq!(drop.x, 50.0);
        assert_eq!(drop.y, 10.0);
        assert_eq!(drop.length, 5.0);
        assert_eq!(drop.speed, 2.0);
        assert_eq!(drop.brightness, 0.8);
    }

    #[test]
    fn test_rain_color() {
        let rain = Rain::new();

        let (r, g, b) = rain.get_rain_color(1.0, 64.0);
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);

        // Test avec brightness nulle
        let (r, g, b) = rain.get_rain_color(0.0, 64.0);
        assert_eq!(r, 0.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
    }

    #[test]
    fn test_rain_render() {
        let mut rain = Rain::new();
        let spectrum = vec![0.5; 64];
        let mut frame = vec![0u8; 128 * 128 * 3];

        rain.render(&spectrum, &mut frame);

        // L'animation counter devrait avoir progressé
        assert!(rain.animation_counter > 0.0);

        // Des gouttes pourraient avoir été ajoutées
        assert!(rain.drops.len() >= 50);
    }
}
