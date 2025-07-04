use rayon::prelude::*;

pub trait Effect: Send + Sync {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]);
}

pub struct EffectEngine {
    effects: Vec<Box<dyn Effect>>,
    current: usize,
    transition: f32,
}

impl EffectEngine {
    pub fn new() -> Self {
        Self {
            effects: vec![
                Box::new(SpectrumBars::new()),
                Box::new(CircularWave::new()),
                Box::new(ParticleSystem::new()),
            ],
            current: 0,
            transition: 0.0,
        }
    }

    pub fn render(&mut self, spectrum: &[f32]) -> Vec<u8> {
        let mut frame = vec![0u8; 128 * 128 * 3];

        if let Some(effect) = self.effects.get_mut(self.current) {
            effect.render(spectrum, &mut frame);
        } else {
            println!("⚠️  No effect at index {}", self.current);
        }

        frame
    }

    pub fn set_effect(&mut self, index: usize) {
        if index < self.effects.len() {
            self.current = index;
            println!("✨ Changed to effect {}", index);
        } else {
            println!(
                "❌ Invalid effect index: {} (max: {})",
                index,
                self.effects.len() - 1
            );
        }
    }
}

// Effet 1: Barres de spectre
pub struct SpectrumBars {
    smoothed: Vec<f32>,
}

impl SpectrumBars {
    pub fn new() -> Self {
        Self {
            smoothed: vec![0.0; 64],
        }
    }
}

impl Effect for SpectrumBars {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Lissage plus réactif
        for i in 0..64 {
            self.smoothed[i] = self.smoothed[i] * 0.3 + spectrum[i] * 0.7;
        }

        // Log de débogage pour vérifier l'audio
        let max_level = self.smoothed.iter().cloned().fold(0.0f32, f32::max);
        if max_level > 0.01 {
            println!(
                "🎵 [SpectrumBars] Audio level: {:.2}, spectrum[0]: {:.2}",
                max_level, self.smoothed[0]
            );
        }

        // Effacer l'écran
        frame.fill(0);

        // Rendu parallèle
        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i % 128) as f32;
            let y = (i / 128) as f32;

            // Calculer la barre correspondante (2 pixels de large)
            let bar = (x / 2.0) as usize;
            if bar < 64 {
                // Hauteur plus réactive avec boost des basses
                let boost = if bar < 8 {
                    1.5
                } else if bar < 16 {
                    1.2
                } else {
                    1.0
                };
                let height = (self.smoothed[bar] * boost).min(1.0) * 128.0;

                if y > 128.0 - height {
                    // Couleur arc-en-ciel selon la fréquence avec saturation variable
                    let hue = (bar as f32 / 64.0) * 360.0;
                    let brightness = 1.0 - (y - (128.0 - height)) / height;
                    let saturation = 0.8 + self.smoothed[bar] * 0.2; // Saturation élevée
                    let (r, g, b) = hsv_to_rgb(hue / 360.0, saturation.min(1.0), brightness);

                    pixel[0] = (r * 255.0) as u8;
                    pixel[1] = (g * 255.0) as u8;
                    pixel[2] = (b * 255.0) as u8;
                }
            }
        });
    }
}

// Effet 2: Onde circulaire
pub struct CircularWave {
    time: f32,
}

impl CircularWave {
    pub fn new() -> Self {
        Self { time: 0.0 }
    }
}

impl Effect for CircularWave {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Vitesse adaptative selon l'énergie
        let total_energy = spectrum.iter().sum::<f32>() / spectrum.len() as f32;
        self.time += 0.05 + total_energy * 0.2;

        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;

        // Log de débogage
        if bass_energy > 0.01 || mid_energy > 0.01 || high_energy > 0.01 {
            println!(
                "🌊 [CircularWave] Bass: {:.2}, Mid: {:.2}, High: {:.2}, Time: {:.1}",
                bass_energy, mid_energy, high_energy, self.time
            );
        }

        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = ((i % 128) as f32 - 64.0) / 64.0;
            let y = ((i / 128) as f32 - 64.0) / 64.0;

            let dist = (x * x + y * y).sqrt();
            let angle = y.atan2(x);

            // Vitesse des ondes modulée par l'audio
            let speed_mod = 1.0 + bass_energy * 3.0;

            // Plusieurs ondes qui pulsent avec différentes fréquences
            let wave1 = ((dist * 20.0 - self.time * 8.0 * speed_mod).sin() + 1.0) / 2.0;
            let wave2 = ((dist * 10.0 - self.time * 4.0 * speed_mod).cos() + 1.0) / 2.0;
            let wave3 = ((dist * 5.0 - self.time * 2.0 * speed_mod).sin() + 1.0) / 2.0;

            // Intensité de base même sans audio
            let base_intensity = 0.3; // Intensité minimale pour voir l'effet
            let audio_intensity =
                wave1 * bass_energy * 2.0 + wave2 * mid_energy * 1.5 + wave3 * high_energy;

            // Combiner intensité de base et audio
            let intensity = (base_intensity + audio_intensity).min(1.0);

            // Couleur arc-en-ciel animée avec modulation audio
            let hue_shift = bass_energy * 0.2;
            let hue = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)
                + self.time * 0.1
                + hue_shift;

            // Saturation élevée pour des couleurs vives
            let saturation = 0.9 + (bass_energy + mid_energy) * 0.1;

            // Créer un pattern visible même sans audio
            let wave_pattern = (wave1 * 0.4 + wave2 * 0.3 + wave3 * 0.3).min(1.0);
            let brightness = (base_intensity + intensity * wave_pattern).min(1.0);

            let (r, g, b) = hsv_to_rgb(hue % 1.0, saturation.min(1.0), brightness);

            pixel[0] = (r * 255.0) as u8;
            pixel[1] = (g * 255.0) as u8;
            pixel[2] = (b * 255.0) as u8;
        });
    }
}

// Effet 3: Système de particules
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
}

impl Effect for ParticleSystem {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Analyser le spectre par bandes de fréquences
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;
        let total_energy = (bass_energy + mid_energy + high_energy) / 3.0;

        // Log de débogage
        if total_energy > 0.01 {
            println!(
                "✨ [Particles] Bass: {:.2}, Mid: {:.2}, High: {:.2}, Particles: {}",
                bass_energy,
                mid_energy,
                high_energy,
                self.particles.len()
            );
        }

        // Toujours ajouter quelques particules pour avoir un effet visible
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
            // Position de spawn selon la fréquence ou aléatoire si pas d'audio
            let (spawn_x, spawn_y) = if i < base_particles {
                // Particules de base : spawn aléatoire
                (rand() * 128.0, 100.0 + rand() * 28.0)
            } else if i % 3 == 0 && bass_energy > 0.1 {
                // Basses : depuis le bas
                (rand() * 128.0, 120.0 + rand() * 8.0)
            } else if i % 3 == 1 && mid_energy > 0.1 {
                // Mediums : depuis les côtés
                if rand() > 0.5 {
                    (0.0 + rand() * 8.0, 64.0 + (rand() - 0.5) * 64.0)
                } else {
                    (120.0 + rand() * 8.0, 64.0 + (rand() - 0.5) * 64.0)
                }
            } else {
                // Aigus ou par défaut : aléatoire
                (rand() * 128.0, rand() * 128.0)
            };

            // Vitesse et couleur selon la fréquence ou par défaut
            let (vx, vy, hue) = if i < base_particles {
                // Particules de base : mouvement lent coloré
                ((rand() - 0.5) * 5.0, -rand() * 8.0 - 2.0, rand())
            } else if i % 3 == 0 {
                // Basses : montée rapide, rouge/orange
                (
                    (rand() - 0.5) * bass_energy * 10.0,
                    -bass_energy * 15.0 - rand() * 5.0,
                    rand() * 0.1,
                ) // Rouge
            } else if i % 3 == 1 {
                // Mediums : mouvement horizontal, vert/bleu
                (
                    (rand() - 0.5) * mid_energy * 15.0,
                    (rand() - 0.5) * mid_energy * 10.0,
                    0.3 + rand() * 0.3,
                ) // Vert-Bleu
            } else {
                // Aigus : explosion, violet/rose
                (
                    (rand() - 0.5) * high_energy * 20.0,
                    (rand() - 0.5) * high_energy * 20.0,
                    0.7 + rand() * 0.3,
                ) // Violet
            };

            self.particles.push(Particle {
                x: spawn_x,
                y: spawn_y,
                vx,
                vy,
                life: 0.5 + total_energy * 0.5, // Vie plus longue si fort
                color: hsv_to_rgb(hue, 1.0, 1.0), // Saturation et luminosité maximales
            });
        }

        // Mise à jour des particules avec physique améliorée
        self.particles.retain_mut(|p| {
            p.x += p.vx;
            p.y += p.vy;

            // Gravité variable selon l'énergie (moins de gravité si musique forte)
            p.vy += 0.3 - total_energy * 0.2;

            // Friction adaptative
            let friction = 0.97 - total_energy * 0.02;
            p.vx *= friction;
            p.vy *= friction;

            // Décroissance de vie plus lente si musique forte
            p.life -= 0.02 - total_energy * 0.01;

            p.life > 0.0 && p.x >= -5.0 && p.x < 133.0 && p.y >= -5.0 && p.y < 133.0
        });

        // Effacer
        frame.fill(0);

        // Dessiner les particules avec un effet de flou et trail
        for particle in &self.particles {
            let x = particle.x as i32;
            let y = particle.y as i32;

            // Taille variable selon la vie restante
            let size = if particle.life > 0.7 { 2 } else { 1 };

            // Dessiner la particule avec un halo
            for dy in -size..=size {
                for dx in -size..=size {
                    let px = x + dx;
                    let py = y + dy;

                    if px >= 0 && px < 128 && py >= 0 && py < 128 {
                        let idx = (py as usize * 128 + px as usize) * 3;
                        if idx + 2 < frame.len() {
                            let factor = if dx == 0 && dy == 0 { 1.0 } else { 0.5 };
                            frame[idx] = ((particle.color.0 * particle.life * 255.0 * factor)
                                as u8)
                                .saturating_add(frame[idx]);
                            frame[idx + 1] = ((particle.color.1 * particle.life * 255.0 * factor)
                                as u8)
                                .saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((particle.color.2 * particle.life * 255.0 * factor)
                                as u8)
                                .saturating_add(frame[idx + 2]);
                        }
                    }
                }
            }
        }
    }
}

// Helpers
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (r + m, g + m, b + m)
}

fn rand() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u32> = Cell::new(0x12345678);
    }

    SEED.with(|seed| {
        let mut s = seed.get();
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        seed.set(s);
        (s as f32) / (u32::MAX as f32)
    })
}
