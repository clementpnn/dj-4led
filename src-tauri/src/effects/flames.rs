// src-tauri/src/effects/flames.rs

use std::f32::consts::PI;
use super::{Effect, get_color_config, hsv_to_rgb, rand};

pub struct Flames {
    particles: Vec<FlameParticle>,
    heat_sources: Vec<f32>,
    time: f32,
    sound_history: Vec<f32>,
    base_temperature: f32,
}

#[derive(Clone)]
struct FlameParticle {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    temperature: f32,
    age: f32,
    max_age: f32,
    size: f32,
    turbulence_offset: f32,
}

impl FlameParticle {
    fn new(x: f32, y: f32, temperature: f32) -> Self {
        Self {
            x,
            y,
            velocity_x: (rand() - 0.5),
            velocity_y: -rand() * 1.5 - 0.5, // Toujours vers le haut
            temperature,
            age: 0.0,
            max_age: 15.0 + rand() * 30.0,
            size: 0.5 + rand() * 1.5,
            turbulence_offset: rand() * 2.0 * PI,
        }
    }

    fn update(&mut self, time: f32, wind_force: f32, sound_intensity: f32) {
        self.age += 1.0;

        // Turbulence atmosphérique
        let turbulence_x = (time * 0.1 + self.turbulence_offset).sin() * 0.3;
        let _turbulence_y = (time * 0.08 + self.turbulence_offset * 1.3).cos() * 0.2;

        // Force vers le haut (convection) avec boost audio
        self.velocity_y -= 0.15 + sound_intensity * 0.1;
        self.velocity_x += turbulence_x + wind_force;

        // Friction de l'air
        self.velocity_x *= 0.98;
        self.velocity_y *= 0.995;

        // Mise à jour de la position
        self.x += self.velocity_x;
        self.y += self.velocity_y;

        // Refroidissement progressif
        let cooling_rate = 0.02 + (self.age / self.max_age) * 0.08;
        self.temperature *= 1.0 - cooling_rate;

        // Changement de taille avec l'âge et l'audio
        let age_factor = 1.0 - (self.age / self.max_age);
        let sound_boost = 1.0 + sound_intensity * 0.5;
        self.size = self.size * 0.999 * age_factor * sound_boost;
    }

    fn is_alive(&self) -> bool {
        self.age < self.max_age
            && self.temperature > 0.05
            && self.y > -10.0
            && self.x > -10.0
            && self.x < 138.0
    }
}

impl Flames {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            heat_sources: vec![0.0; 128],
            time: 0.0,
            sound_history: vec![0.0; 10],
            base_temperature: 0.0,
        }
    }

    fn get_flame_color(&self, temperature: f32, age_factor: f32) -> (f32, f32, f32) {
        let color_config = get_color_config();
        let t = temperature.clamp(0.0, 1.0);

        match color_config.mode.as_str() {
            "fire" => {
                // Flamme réaliste : bleu -> blanc -> jaune -> orange -> rouge
                if t < 0.15 {
                    (0.8 + t * 1.3, 0.8 + t * 1.3, 1.0)
                } else if t < 0.3 {
                    let factor = (t - 0.15) / 0.15;
                    (1.0, 0.8 - factor * 0.3, 1.0 - factor * 0.8)
                } else if t < 0.6 {
                    let factor = (t - 0.3) / 0.3;
                    (1.0, 0.5 + factor * 0.5, 0.2 * factor)
                } else if t < 0.85 {
                    let factor = (t - 0.6) / 0.25;
                    (1.0, 1.0, 0.2 + factor * 0.6)
                } else {
                    (1.0 - (t - 0.85) * 0.3, 1.0 - (t - 0.85) * 0.2, 0.8)
                }
            }
            "ocean" => {
                // Flammes bleues/cyan comme de l'eau en feu
                if t < 0.3 {
                    (0.0, t * 0.5, 0.8 + t * 0.7)
                } else if t < 0.7 {
                    let factor = (t - 0.3) / 0.4;
                    (factor * 0.3, 0.15 + factor * 0.5, 1.0)
                } else {
                    let factor = (t - 0.7) / 0.3;
                    (0.3 + factor * 0.7, 0.65 + factor * 0.35, 1.0)
                }
            }
            "rainbow" => {
                let hue = (self.time * 0.02 + age_factor * 0.5) % 1.0;
                let saturation = 0.8 + t * 0.2;
                let value = t;
                hsv_to_rgb(hue, saturation, value)
            }
            "sunset" => {
                // Couleurs chaudes du coucher de soleil
                if t < 0.4 {
                    (0.4 + t * 0.6, 0.1, 0.6 + t * 0.4)
                } else if t < 0.7 {
                    let factor = (t - 0.4) / 0.3;
                    (1.0, 0.1 + factor * 0.5, 1.0 - factor * 0.8)
                } else {
                    let factor = (t - 0.7) / 0.3;
                    (1.0, 0.6 + factor * 0.4, 0.2 * (1.0 - factor))
                }
            }
            "custom" => {
                let (r, g, b) = color_config.custom_color;
                let intensity = t.powf(0.7); // Courbe plus naturelle
                (r * intensity, g * intensity, b * intensity)
            }
            _ => {
                // Flamme classique orange/rouge
                if t < 0.3 {
                    (t * 3.0, 0.0, 0.0)
                } else if t < 0.7 {
                    let factor = (t - 0.3) / 0.4;
                    (1.0, factor * 0.8, 0.0)
                } else {
                    let factor = (t - 0.7) / 0.3;
                    (1.0, 0.8 + factor * 0.2, factor * 0.6)
                }
            }
        }
    }

    fn create_flame_base(&mut self, sound_intensity: f32) {
        let base_particles = 2;
        let sound_particles = (sound_intensity * 15.0) as usize;
        let total_new_particles = base_particles + sound_particles;

        let base_width = 8.0 + sound_intensity * 20.0;
        let base_center = 64.0;

        for _ in 0..total_new_particles {
            let x_offset = (rand() - 0.5) * base_width * 2.0;
            let x = base_center + x_offset;
            let y = 127.0 + (rand() - 0.5) * 4.0;

            // Plus chaud au centre
            let distance_from_center = (x - base_center).abs() / base_width;
            let center_boost = 1.0 - distance_from_center.clamp(0.0, 1.0);
            let temperature = 0.7 + sound_intensity * 0.3 + center_boost * 0.2;

            if x >= 0.0 && x < 128.0 {
                let particle = FlameParticle::new(x, y, temperature);
                self.particles.push(particle);
            }
        }
    }

    fn add_sparks(&mut self, sound_intensity: f32) {
        if sound_intensity > 0.3 && rand() < sound_intensity * 0.5 {
            let spark_count = (sound_intensity * 5.0) as usize;

            for _ in 0..spark_count {
                let x = 64.0 + (rand() - 0.5) * 60.0;
                let y = 127.0 - rand() * 40.0;

                let mut spark = FlameParticle::new(x, y, 0.9);
                spark.velocity_x = (rand() - 0.5) * 6.0;
                spark.velocity_y = -rand() * 4.0 - 1.0;
                spark.max_age = 8.0 + rand() * 12.0;
                spark.size = 0.3 + rand() * 0.7;

                self.particles.push(spark);
            }
        }
    }
}

impl Effect for Flames {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        let bass_energy = (spectrum[..8].iter().sum::<f32>() / 8.0) * 3.0;
        let mid_energy = (spectrum[8..24].iter().sum::<f32>() / 16.0) * 2.0;
        let high_energy = (spectrum[24..].iter().sum::<f32>() / 40.0) * 1.5;

        let raw_intensity = bass_energy * 0.6 + mid_energy * 0.3 + high_energy * 0.1;
        let sound_intensity = raw_intensity.clamp(0.0, 1.0);

        // Lissage de l'intensité audio
        self.sound_history.remove(0);
        self.sound_history.push(sound_intensity);
        let smoothed_intensity =
            self.sound_history.iter().sum::<f32>() / self.sound_history.len() as f32;

        self.time += 1.0 + sound_intensity * 2.0;

        // Créer la base des flammes
        self.create_flame_base(smoothed_intensity);

        // Ajouter des étincelles
        self.add_sparks(sound_intensity);

        // Force du vent basée sur les hautes fréquences
        let wind_force = (high_energy - 0.1).max(0.0) * 0.3 * (self.time * 0.05).sin();

        // Mettre à jour toutes les particules
        for particle in &mut self.particles {
            particle.update(self.time, wind_force, sound_intensity);
        }

        // Supprimer les particules mortes
        self.particles.retain(|p| p.is_alive());

        // Limiter le nombre de particules selon l'intensité
        let max_particles = 300 + (sound_intensity * 200.0) as usize;
        if self.particles.len() > max_particles {
            self.particles
                .drain(0..self.particles.len() - max_particles);
        }

        // Effacer la frame
        for pixel in frame.iter_mut() {
            *pixel = 0;
        }

        // Buffer de température pour le rendu
        let mut temperature_buffer = vec![0.0f32; 128 * 128];

        // Rasteriser les particules dans le buffer de température
        for particle in &self.particles {
            let px = particle.x as i32;
            let py = particle.y as i32;

            let radius = particle.size.max(1.0);
            let radius_sq = radius * radius;

            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    let x = px + dx;
                    let y = py + dy;

                    if x >= 0 && x < 128 && y >= 0 && y < 128 {
                        let dist_sq = (dx * dx + dy * dy) as f32;

                        if dist_sq <= radius_sq {
                            let attenuation = (1.0 - dist_sq / radius_sq).max(0.0);
                            let contrib = particle.temperature * attenuation;

                            let idx = (y * 128 + x) as usize;
                            temperature_buffer[idx] = temperature_buffer[idx].max(contrib);
                        }
                    }
                }
            }
        }

        // Convertir le buffer de température en couleurs
        for y in 0..128 {
            for x in 0..128 {
                let idx = y * 128 + x;
                let temperature = temperature_buffer[idx];

                if temperature > 0.01 {
                    let age_factor = 1.0 - (y as f32 / 128.0);
                    let (r, g, b) = self.get_flame_color(temperature, age_factor);

                    let frame_idx = idx * 3;
                    frame[frame_idx] = (r * 255.0).clamp(0.0, 255.0) as u8;
                    frame[frame_idx + 1] = (g * 255.0).clamp(0.0, 255.0) as u8;
                    frame[frame_idx + 2] = (b * 255.0).clamp(0.0, 255.0) as u8;
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
        "Flames"
    }

    fn description(&self) -> &'static str {
        "Realistic flame simulation with particle physics and audio reactivity"
    }

    fn reset(&mut self) {
        self.particles.clear();
        self.heat_sources.fill(0.0);
        self.time = 0.0;
        self.sound_history.fill(0.0);
        self.base_temperature = 0.0;
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flames_creation() {
        let flames = Flames::new();
        assert_eq!(flames.particles.len(), 0);
        assert_eq!(flames.heat_sources.len(), 128);
        assert_eq!(flames.time, 0.0);
        assert_eq!(flames.sound_history.len(), 10);
    }

    #[test]
    fn test_flames_reset() {
        let mut flames = Flames::new();
        flames.time = 100.0;
        flames.particles.push(FlameParticle::new(64.0, 64.0, 0.8));
        flames.sound_history[0] = 0.5;

        flames.reset();

        assert_eq!(flames.particles.len(), 0);
        assert_eq!(flames.time, 0.0);
        assert_eq!(flames.sound_history[0], 0.0);
        assert_eq!(flames.heat_sources[0], 0.0);
    }

    #[test]
    fn test_flame_particle_creation() {
        let particle = FlameParticle::new(50.0, 60.0, 0.8);
        assert_eq!(particle.x, 50.0);
        assert_eq!(particle.y, 60.0);
        assert_eq!(particle.temperature, 0.8);
        assert_eq!(particle.age, 0.0);
        assert!(particle.max_age > 0.0);
        assert!(particle.size > 0.0);
    }

    #[test]
    fn test_flame_particle_update() {
        let mut particle = FlameParticle::new(64.0, 100.0, 0.8);
        let initial_y = particle.y;
        let initial_temp = particle.temperature;

        particle.update(1.0, 0.0, 0.1);

        // La particule devrait monter (y diminue)
        assert!(particle.y < initial_y);

        // La température devrait diminuer légèrement
        assert!(particle.temperature < initial_temp);

        // L'âge devrait augmenter
        assert_eq!(particle.age, 1.0);
    }

    #[test]
    fn test_flame_particle_lifetime() {
        let mut particle = FlameParticle::new(64.0, 100.0, 0.8);
        particle.max_age = 10.0;

        assert!(particle.is_alive());

        // Vieillir la particule
        particle.age = 15.0;
        assert!(!particle.is_alive());

        // Tester avec température faible
        particle.age = 0.0;
        particle.temperature = 0.01;
        assert!(!particle.is_alive());
    }

    #[test]
    fn test_flame_color() {
        let flames = Flames::new();

        let (r, g, b) = flames.get_flame_color(0.8, 0.5);
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);

        // Température nulle devrait donner du noir
        let (r, g, b) = flames.get_flame_color(0.0, 0.5);
        assert_eq!(r, 0.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
    }
}
