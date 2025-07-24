// src-tauri/src/effects/particle_system.rs

use super::{Effect, get_color_config, hsv_to_rgb, rand};

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    color: (f32, f32, f32),
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(1000),
        }
    }

    fn get_particle_color(
        &self,
        particle_index: usize,
        base_particles: usize,
        _bass_energy: f32,
        _mid_energy: f32,
        _high_energy: f32,
    ) -> (f32, f32, f32) {
        let color_config = get_color_config();

        match color_config.mode.as_str() {
            "rainbow" => {
                let hue = if particle_index < base_particles {
                    rand()
                } else if particle_index % 3 == 0 {
                    rand() * 0.1 // Rouge
                } else if particle_index % 3 == 1 {
                    0.3 + rand() * 0.3 // Vert
                } else {
                    0.7 + rand() * 0.3 // Bleu
                };
                hsv_to_rgb(hue, 1.0, 1.0)
            }
            "fire" => {
                let hue = rand() * 0.15; // Tons chauds
                let saturation = 0.8 + rand() * 0.2;
                let brightness = 0.7 + rand() * 0.3;
                hsv_to_rgb(hue, saturation, brightness)
            }
            "ocean" => {
                let hue = 0.5 + rand() * 0.17; // Bleus et cyans
                let saturation = 0.6 + rand() * 0.4;
                let brightness = 0.6 + rand() * 0.4;
                hsv_to_rgb(hue, saturation, brightness)
            }
            "sunset" => {
                let hue = if rand() > 0.5 {
                    0.833 + rand() * 0.167 // Violets
                } else {
                    rand() * 0.167 // Rouges/oranges
                };
                hsv_to_rgb(hue, 1.0, 1.0)
            }
            "custom" => {
                let (r, g, b) = color_config.custom_color;
                let variation = 0.8 + rand() * 0.4;
                (
                    (r * variation).min(1.0),
                    (g * variation).min(1.0),
                    (b * variation).min(1.0),
                )
            }
            _ => hsv_to_rgb(rand(), 1.0, 1.0),
        }
    }
}

impl Effect for ParticleSystem {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;
        let total_energy = (bass_energy + mid_energy + high_energy) / 3.0;

        // Générer de nouvelles particules
        let base_particles = if self.particles.len() < 100 { 2 } else { 0 };
        let audio_particles = if total_energy > 0.05 && self.particles.len() < 2000 {
            ((bass_energy * 50.0).min(20.0)
                + (mid_energy * 30.0).min(10.0)
                + (high_energy * 20.0).min(5.0)) as usize
        } else {
            0
        };

        let num_particles = base_particles + audio_particles;

        for i in 0..num_particles {
            let (spawn_x, spawn_y) = if i < base_particles {
                // Particules de base depuis le bas
                (rand() * 128.0, 100.0 + rand() * 28.0)
            } else if i % 3 == 0 && bass_energy > 0.1 {
                // Particules de basse depuis le bas
                (rand() * 128.0, 120.0 + rand() * 8.0)
            } else if i % 3 == 1 && mid_energy > 0.1 {
                // Particules moyennes depuis les côtés
                if rand() > 0.5 {
                    (0.0 + rand() * 8.0, 64.0 + (rand() - 0.5) * 64.0)
                } else {
                    (120.0 + rand() * 8.0, 64.0 + (rand() - 0.5) * 64.0)
                }
            } else {
                // Particules d'aigus aléatoirement
                (rand() * 128.0, rand() * 128.0)
            };

            let (vx, vy) = if i < base_particles {
                ((rand() - 0.5) * 5.0, -rand() * 8.0 - 2.0)
            } else if i % 3 == 0 {
                // Particules de basse - vers le haut
                (
                    (rand() - 0.5) * bass_energy * 10.0,
                    -bass_energy * 15.0 - rand() * 5.0,
                )
            } else if i % 3 == 1 {
                // Particules moyennes - horizontales
                (
                    (rand() - 0.5) * mid_energy * 15.0,
                    (rand() - 0.5) * mid_energy * 10.0,
                )
            } else {
                // Particules d'aigus - rapides et erratiques
                (
                    (rand() - 0.5) * high_energy * 20.0,
                    (rand() - 0.5) * high_energy * 20.0,
                )
            };

            let color = self.get_particle_color(i, base_particles, bass_energy, mid_energy, high_energy);

            self.particles.push(Particle {
                x: spawn_x,
                y: spawn_y,
                vx,
                vy,
                life: 0.5 + total_energy * 0.5,
                color,
            });
        }

        // Mettre à jour les particules existantes
        self.particles.retain_mut(|p| {
            p.x += p.vx;
            p.y += p.vy;

            // Gravité avec influence audio
            p.vy += 0.3 - total_energy * 0.2;

            // Friction
            let friction = 0.97 - total_energy * 0.02;
            p.vx *= friction;
            p.vy *= friction;

            // Décrémenter la vie
            p.life -= 0.02 - total_energy * 0.01;

            // Garder seulement les particules vivantes et dans l'écran
            p.life > 0.0 && p.x >= -5.0 && p.x < 133.0 && p.y >= -5.0 && p.y < 133.0
        });

        // Effacer la frame
        frame.fill(0);

        // Dessiner les particules
        for particle in &self.particles {
            let x = particle.x as i32;
            let y = particle.y as i32;

            // Taille de la particule basée sur sa vie
            let size = if particle.life > 0.7 { 2 } else { 1 };

            for dy in -size..=size {
                for dx in -size..=size {
                    let px = x + dx;
                    let py = y + dy;

                    if px >= 0 && px < 128 && py >= 0 && py < 128 {
                        let idx = (py as usize * 128 + px as usize) * 3;
                        if idx + 2 < frame.len() {
                            let factor = if dx == 0 && dy == 0 { 1.0 } else { 0.5 };
                            frame[idx] = ((particle.color.0 * particle.life * 255.0 * factor) as u8)
                                .saturating_add(frame[idx]);
                            frame[idx + 1] = ((particle.color.1 * particle.life * 255.0 * factor) as u8)
                                .saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((particle.color.2 * particle.life * 255.0 * factor) as u8)
                                .saturating_add(frame[idx + 2]);
                        }
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
        "Particle System"
    }

    fn description(&self) -> &'static str {
        "Dynamic particle effects that react to different frequency ranges"
    }

    fn reset(&mut self) {
        self.particles.clear();
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_system_creation() {
        let system = ParticleSystem::new();
        assert_eq!(system.particles.len(), 0);
    }

    #[test]
    fn test_particle_system_reset() {
        let mut system = ParticleSystem::new();

        // Ajouter quelques particules
        system.particles.push(Particle {
            x: 50.0,
            y: 50.0,
            vx: 1.0,
            vy: -1.0,
            life: 1.0,
            color: (1.0, 0.0, 0.0),
        });

        assert_eq!(system.particles.len(), 1);

        system.reset();
        assert_eq!(system.particles.len(), 0);
    }

    #[test]
    fn test_particle_system_render() {
        let mut system = ParticleSystem::new();
        let spectrum = vec![0.5; 64];
        let mut frame = vec![0u8; 128 * 128 * 3];

        system.render(&spectrum, &mut frame);

        // Des particules devraient avoir été créées
        assert!(system.particles.len() > 0);
    }

    #[test]
    fn test_particle_color_generation() {
        let system = ParticleSystem::new();

        let (r, g, b) = system.get_particle_color(0, 1, 0.5, 0.5, 0.5);
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);
    }
}
