use rayon::prelude::*;

pub trait Effect: Send + Sync {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]);
    fn set_color_mode(&mut self, mode: &str);
    fn set_custom_color(&mut self, r: f32, g: f32, b: f32);
}

#[derive(Clone)]
pub struct ColorConfig {
    pub mode: String,
    pub custom_color: (f32, f32, f32),
}

// Global color config that will be used by all effects
static mut GLOBAL_COLOR_CONFIG: ColorConfig = ColorConfig {
    mode: String::new(),
    custom_color: (1.0, 0.0, 0.5),
};

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            mode: "rainbow".to_string(),
            custom_color: (1.0, 0.0, 0.5),
        }
    }
}

pub struct EffectEngine {
    effects: Vec<Box<dyn Effect>>,
    current: usize,
    transition: f32,
    color_config: ColorConfig,
}

impl EffectEngine {
    pub fn new() -> Self {
        // Initialize global color config
        unsafe {
            GLOBAL_COLOR_CONFIG = ColorConfig::default();
        }

        Self {
            effects: vec![
                Box::new(SpectrumBars::new()) as Box<dyn Effect>,
                Box::new(CircularWave::new()) as Box<dyn Effect>,
                Box::new(ParticleSystem::new()) as Box<dyn Effect>,
                Box::new(Applaudimetre::new()) as Box<dyn Effect>,
                Box::new(Flames::new()) as Box<dyn Effect>,
                Box::new(Rain::new()) as Box<dyn Effect>,
                Box::new(Starfall::new()) as Box<dyn Effect>,
                Box::new(Heartbeat::new()) as Box<dyn Effect>,
            ],
            current: 0,
            transition: 0.0,
            color_config: ColorConfig::default(),
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

    pub fn set_color_mode(&mut self, mode: &str) {
        println!("🎨 EffectEngine: Setting color mode to '{}'", mode);
        self.color_config.mode = mode.to_string();

        // Update global config
        unsafe {
            GLOBAL_COLOR_CONFIG.mode = mode.to_string();
        }

        for (i, effect) in self.effects.iter_mut().enumerate() {
            println!("   Setting color mode for effect {}", i);
            effect.set_color_mode(mode);
        }
    }

    pub fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "🎨 EffectEngine: Setting custom color to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
        self.color_config.custom_color = (r, g, b);

        // Update global config
        unsafe {
            GLOBAL_COLOR_CONFIG.custom_color = (r, g, b);
        }

        for (i, effect) in self.effects.iter_mut().enumerate() {
            println!("   Setting custom color for effect {}", i);
            effect.set_custom_color(r, g, b);
        }
    }
}

// Effet 1: Barres de spectre
pub struct SpectrumBars {
    smoothed: Vec<f32>,
    peak_hold: Vec<f32>,
    peak_decay: Vec<f32>,
}

impl SpectrumBars {
    pub fn new() -> Self {
        Self {
            smoothed: vec![0.0; 64],
            peak_hold: vec![0.0; 64],
            peak_decay: vec![0.0; 64],
        }
    }

    fn get_color_for_bar(&self, bar: usize, brightness: f32) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };
        match color_mode.mode.as_str() {
            "rainbow" => {
                let hue = (bar as f32 / 64.0) * 360.0;
                let saturation = 0.8
                    + if bar < self.smoothed.len() {
                        self.smoothed[bar] * 0.2
                    } else {
                        0.0
                    };
                hsv_to_rgb(hue / 360.0, saturation.min(1.0), brightness)
            }
            "fire" => {
                let hue = (bar as f32 / 64.0) * 60.0; // Rouge à jaune
                let saturation = 1.0;
                hsv_to_rgb(hue / 360.0, saturation, brightness)
            }
            "ocean" => {
                let hue = 180.0 + (bar as f32 / 64.0) * 60.0; // Cyan à bleu
                let saturation = 0.8
                    + if bar < self.smoothed.len() {
                        self.smoothed[bar] * 0.2
                    } else {
                        0.0
                    };
                hsv_to_rgb(hue / 360.0, saturation.min(1.0), brightness)
            }
            "sunset" => {
                let hue = if bar < 32 {
                    300.0 + (bar as f32 / 32.0) * 60.0 // Violet à rouge
                } else {
                    (bar as f32 - 32.0) / 32.0 * 60.0 // Rouge à jaune
                };
                hsv_to_rgb(hue / 360.0, 1.0, brightness)
            }
            "custom" => {
                let (r, g, b) = color_mode.custom_color;
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
        // Lissage adaptatif : plus réactif pour les montées, plus lent pour les descentes
        for i in 0..64 {
            let target = spectrum[i];
            let current = self.smoothed[i];

            if target > current {
                // Montée rapide
                self.smoothed[i] = current * 0.4 + target * 0.6;
            } else {
                // Descente lente pour un effet plus fluide
                self.smoothed[i] = current * 0.85 + target * 0.15;
            }

            // Gestion des peaks avec décroissance
            if self.smoothed[i] > self.peak_hold[i] {
                self.peak_hold[i] = self.smoothed[i];
                self.peak_decay[i] = 0.0;
            } else {
                self.peak_decay[i] += 0.02;
                self.peak_hold[i] = (self.peak_hold[i] - self.peak_decay[i]).max(0.0);
            }
        }

        // Log de débogage moins fréquent
        static mut DEBUG_COUNTER: u32 = 0;
        unsafe {
            DEBUG_COUNTER += 1;
            if DEBUG_COUNTER % 50 == 0 {
                let max_level = self.smoothed.iter().cloned().fold(0.0f32, f32::max);
                if max_level > 0.01 {
                    println!(
                        "🎵 [SpectrumBars] Audio level: {:.2}, spectrum[0]: {:.2}",
                        max_level, self.smoothed[0]
                    );
                }
            }
        }

        // Log de débogage pour vérifier le mode de couleur
        static mut FRAME_COUNT: u64 = 0;
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 60 == 0 {
                println!(
                    "🎨 [SpectrumBars] Current color mode: '{}', custom color: ({:.2}, {:.2}, {:.2})",
                    GLOBAL_COLOR_CONFIG.mode,
                    GLOBAL_COLOR_CONFIG.custom_color.0,
                    GLOBAL_COLOR_CONFIG.custom_color.1,
                    GLOBAL_COLOR_CONFIG.custom_color.2
                );
            }
        }

        // Effacer l'écran
        frame.fill(0);

        // Rendu parallèle
        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i % 128) as f32;
            let y = (i / 128) as f32;

            // Effet miroir : afficher le spectre deux fois (32 barres + 32 barres en miroir)
            let bar = if x < 64.0 {
                // Partie gauche : barres 0-31
                (x * 32.0 / 64.0) as usize
            } else {
                // Partie droite : barres 31-0 (miroir)
                (31.0 - (x - 64.0) * 32.0 / 64.0) as usize
            };

            if bar < 32 {
                // Hauteur avec courbe exponentielle pour un meilleur rendu
                // Utiliser les 32 premières barres du spectre pour les deux côtés
                let value = self.smoothed[bar.min(31)];
                // Appliquer une courbe pour rendre les barres plus dynamiques
                let curved_value = if value > 0.0 {
                    value.powf(0.6) // Rend les valeurs moyennes plus visibles
                } else {
                    0.0
                };

                let height = curved_value * 120.0; // Légèrement moins que 128 pour éviter saturation
                let peak_height = self.peak_hold[bar] * 120.0;

                // Gradient vertical pour chaque barre
                let bar_bottom = 128.0 - height;
                let distance_from_bottom = (y - bar_bottom).max(0.0);
                let gradient_factor = if y >= bar_bottom && y < 128.0 {
                    1.0 - (distance_from_bottom / height).min(1.0) * 0.3
                } else {
                    0.0
                };

                // Dessiner la barre principale avec gradient
                if y >= bar_bottom && y < 128.0 {
                    let brightness = gradient_factor;
                    let (r, g, b) = self.get_color_for_bar(bar, brightness);
                    pixel[0] = (r * 255.0) as u8;
                    pixel[1] = (g * 255.0) as u8;
                    pixel[2] = (b * 255.0) as u8;
                }

                // Dessiner le peak (ligne fine en haut)
                let peak_y = 128.0 - peak_height;
                if (y - peak_y).abs() < 1.0 && peak_height > 5.0 {
                    let (r, g, b) = self.get_color_for_bar(bar, 0.8);
                    pixel[0] = (r * 255.0 * 0.8) as u8;
                    pixel[1] = (g * 255.0 * 0.8) as u8;
                    pixel[2] = (b * 255.0 * 0.8) as u8;
                }

                // Ajouter une ligne de séparation subtile entre les barres
                let bar_pos = if x < 64.0 {
                    x * 32.0 / 64.0
                } else {
                    31.0 - (x - 64.0) * 32.0 / 64.0
                };
                let bar_boundary = (bar_pos - bar as f32).abs() * 64.0 / 32.0;
                if bar_boundary > 1.8 && y >= bar_bottom && y < 128.0 {
                    // Atténuer légèrement les bords pour créer une séparation visuelle
                    pixel[0] = (pixel[0] as f32 * 0.7) as u8;
                    pixel[1] = (pixel[1] as f32 * 0.7) as u8;
                    pixel[2] = (pixel[2] as f32 * 0.7) as u8;
                }

                // Ajouter une ligne centrale brillante pour séparer les deux moitiés
                if (x - 64.0).abs() < 0.5 && y >= bar_bottom && y < 128.0 {
                    let (r, g, b) = self.get_color_for_bar(bar, 0.3);
                    pixel[0] = (r * 255.0) as u8;
                    pixel[1] = (g * 255.0) as u8;
                    pixel[2] = (b * 255.0) as u8;
                }
            }
        });
    }

    fn set_color_mode(&mut self, mode: &str) {
        println!("   SpectrumBars: color mode set to '{}'", mode);
        // Color mode is now set globally
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "   SpectrumBars: custom color set to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
        // Custom color is now set globally
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

    fn get_color_for_wave(
        &self,
        angle: f32,
        dist: f32,
        brightness: f32,
        bass_energy: f32,
        mid_energy: f32,
    ) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };
        match color_mode.mode.as_str() {
            "rainbow" => {
                let hue_shift = bass_energy * 0.2;
                let hue = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)
                    + self.time * 0.1
                    + hue_shift;
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
                let progress = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                let hue = if progress < 0.5 {
                    300.0 / 360.0 + progress * 120.0 / 360.0
                } else {
                    60.0 / 360.0 * (1.0 - (progress - 0.5) * 2.0)
                };
                hsv_to_rgb(hue, 1.0, brightness)
            }
            "custom" => {
                let (r, g, b) = color_mode.custom_color;
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

        // Log de débogage pour vérifier le mode de couleur
        static mut WAVE_FRAME_COUNT: u64 = 0;
        unsafe {
            WAVE_FRAME_COUNT += 1;
            if WAVE_FRAME_COUNT % 60 == 0 {
                println!(
                    "🎨 [CircularWave] Current color mode: '{}', custom color: ({:.2}, {:.2}, {:.2})",
                    GLOBAL_COLOR_CONFIG.mode,
                    GLOBAL_COLOR_CONFIG.custom_color.0,
                    GLOBAL_COLOR_CONFIG.custom_color.1,
                    GLOBAL_COLOR_CONFIG.custom_color.2
                );
            }
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

            // Créer un pattern visible même sans audio
            let wave_pattern = (wave1 * 0.4 + wave2 * 0.3 + wave3 * 0.3).min(1.0);
            let brightness = (base_intensity + intensity * wave_pattern).min(1.0);

            let (r, g, b) =
                self.get_color_for_wave(angle, dist, brightness, bass_energy, mid_energy);

            pixel[0] = (r * 255.0) as u8;
            pixel[1] = (g * 255.0) as u8;
            pixel[2] = (b * 255.0) as u8;
        });
    }

    fn set_color_mode(&mut self, mode: &str) {
        println!("   CircularWave: color mode set to '{}'", mode);
        // Color mode is now set globally
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "   CircularWave: custom color set to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
        // Custom color is now set globally
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

    fn get_particle_color(
        &self,
        particle_index: usize,
        base_particles: usize,
        _bass_energy: f32,
        _mid_energy: f32,
        _high_energy: f32,
    ) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };
        match color_mode.mode.as_str() {
            "rainbow" => {
                let hue = if particle_index < base_particles {
                    rand()
                } else if particle_index % 3 == 0 {
                    rand() * 0.1 // Rouge
                } else if particle_index % 3 == 1 {
                    0.3 + rand() * 0.3 // Vert-Bleu
                } else {
                    0.7 + rand() * 0.3 // Violet
                };
                hsv_to_rgb(hue, 1.0, 1.0)
            }
            "fire" => {
                let hue = rand() * 0.15; // Rouge à jaune
                let saturation = 0.8 + rand() * 0.2;
                let brightness = 0.7 + rand() * 0.3;
                hsv_to_rgb(hue, saturation, brightness)
            }
            "ocean" => {
                let hue = 0.5 + rand() * 0.17; // Cyan à bleu
                let saturation = 0.6 + rand() * 0.4;
                let brightness = 0.6 + rand() * 0.4;
                hsv_to_rgb(hue, saturation, brightness)
            }
            "sunset" => {
                let hue = if rand() > 0.5 {
                    0.833 + rand() * 0.167 // Violet à rouge
                } else {
                    rand() * 0.167 // Rouge à jaune
                };
                hsv_to_rgb(hue, 1.0, 1.0)
            }
            "custom" => {
                let (r, g, b) = color_mode.custom_color;
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

        // Log de débogage pour vérifier le mode de couleur
        static mut PARTICLE_FRAME_COUNT: u64 = 0;
        unsafe {
            PARTICLE_FRAME_COUNT += 1;
            if PARTICLE_FRAME_COUNT % 60 == 0 {
                println!(
                    "🎨 [Particles] Current color mode: '{}', custom color: ({:.2}, {:.2}, {:.2})",
                    GLOBAL_COLOR_CONFIG.mode,
                    GLOBAL_COLOR_CONFIG.custom_color.0,
                    GLOBAL_COLOR_CONFIG.custom_color.1,
                    GLOBAL_COLOR_CONFIG.custom_color.2
                );
            }
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

            // Vitesse selon la fréquence
            let (vx, vy) = if i < base_particles {
                // Particules de base : mouvement lent
                ((rand() - 0.5) * 5.0, -rand() * 8.0 - 2.0)
            } else if i % 3 == 0 {
                // Basses : montée rapide
                (
                    (rand() - 0.5) * bass_energy * 10.0,
                    -bass_energy * 15.0 - rand() * 5.0,
                )
            } else if i % 3 == 1 {
                // Mediums : mouvement horizontal
                (
                    (rand() - 0.5) * mid_energy * 15.0,
                    (rand() - 0.5) * mid_energy * 10.0,
                )
            } else {
                // Aigus : explosion
                (
                    (rand() - 0.5) * high_energy * 20.0,
                    (rand() - 0.5) * high_energy * 20.0,
                )
            };

            // Couleur selon le mode
            let color =
                self.get_particle_color(i, base_particles, bass_energy, mid_energy, high_energy);

            self.particles.push(Particle {
                x: spawn_x,
                y: spawn_y,
                vx,
                vy,
                life: 0.5 + total_energy * 0.5, // Vie plus longue si fort
                color,
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

    fn set_color_mode(&mut self, mode: &str) {
        println!("   ParticleSystem: color mode set to '{}'", mode);
        // Color mode is now set globally
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "   ParticleSystem: custom color set to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
        // Custom color is now set globally
    }
}

// Effet 4: Flammes
pub struct Flames {
    // Tableau de hauteurs pour les flammes
    heights: Vec<f32>,
    // Chaleur à la base des flammes
    heat_sources: Vec<f32>,
    // Facteur de refroidissement
    cooling_map: Vec<f32>,
    // Compteur d'animation
    animation_counter: f32,
}

impl Flames {
    pub fn new() -> Self {
        // Initialiser les hauteurs des flammes à zéro
        let mut heights = vec![0.0; 128];

        // Initialiser les sources de chaleur
        let mut heat_sources = vec![0.0; 128];

        // Créer une carte de refroidissement avec des valeurs aléatoires
        let mut cooling_map = vec![0.0; 128 * 128];
        for i in 0..cooling_map.len() {
            cooling_map[i] = rand() * 0.2;
        }

        Self {
            heights,
            heat_sources,
            cooling_map,
            animation_counter: 0.0,
        }
    }

    fn get_flame_color(&self, temperature: f32, x: usize, y: usize) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        // Normaliser la température entre 0.0 et 1.0
        let t = temperature.min(1.0).max(0.0);

        match color_mode.mode.as_str() {
            // Mode flammes classique (rouge, orange, jaune)
            "fire" => {
                if t < 0.2 {
                    // Noir/rouge très sombre
                    (t * 5.0 * 0.5, 0.0, 0.0)
                } else if t < 0.5 {
                    // Rouge vers orange
                    (0.5 + (t - 0.2) * 0.5 / 0.3, (t - 0.2) * 0.4 / 0.3, 0.0)
                } else {
                    // Orange vers jaune
                    (1.0, 0.4 + (t - 0.5) * 0.6 / 0.5, (t - 0.5) * 0.2 / 0.5)
                }
            },
            // Mode flammes bleues
            "ocean" => {
                if t < 0.2 {
                    // Noir/bleu très sombre
                    (0.0, 0.0, t * 5.0 * 0.5)
                } else if t < 0.5 {
                    // Bleu vers cyan
                    (0.0, (t - 0.2) * 0.4 / 0.3, 0.5 + (t - 0.2) * 0.5 / 0.3)
                } else {
                    // Cyan vers blanc-bleu
                    ((t - 0.5) * 0.5 / 0.5, 0.4 + (t - 0.5) * 0.6 / 0.5, 1.0)
                }
            },
            // Mode arc-en-ciel
            "rainbow" => {
                let hue = (self.animation_counter * 0.01 + (x as f32 / 128.0) * 0.5) % 1.0;
                let saturation = 1.0 - t * 0.5; // Moins saturé en haut
                let value = t;
                hsv_to_rgb(hue, saturation, value)
            },
            // Mode coucher de soleil
            "sunset" => {
                if t < 0.33 {
                    // Violet foncé vers violet
                    (0.2 + t * 0.3 / 0.33, 0.0, 0.3 + t * 0.3 / 0.33)
                } else if t < 0.66 {
                    // Violet vers rouge
                    (0.5 + (t - 0.33) * 0.5 / 0.33, 0.0, 0.6 - (t - 0.33) * 0.6 / 0.33)
                } else {
                    // Rouge vers orange-jaune
                    (1.0, (t - 0.66) * 0.8 / 0.34, 0.0)
                }
            },
            // Mode couleur personnalisée
            "custom" => {
                let (r, g, b) = color_mode.custom_color;
                (r * t, g * t, b * t)
            },
            // Par défaut, flammes classiques
            _ => {
                if t < 0.2 {
                    (t * 5.0 * 0.5, 0.0, 0.0)
                } else if t < 0.5 {
                    (0.5 + (t - 0.2) * 0.5 / 0.3, (t - 0.2) * 0.4 / 0.3, 0.0)
                } else {
                    (1.0, 0.4 + (t - 0.5) * 0.6 / 0.5, (t - 0.5) * 0.2 / 0.5)
                }
            }
        }
    }
}

impl Effect for Flames {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Analyser le spectre audio pour influencer les flammes
        // Augmenter drastiquement la sensibilité en multipliant les valeurs du spectre
        let sensitivity = 5.0; // Facteur d'amplification du signal audio beaucoup plus élevé

        // Appliquer la sensibilité aux différentes bandes de fréquence
        let bass_energy = (spectrum[..8].iter().sum::<f32>() / 8.0) * sensitivity;
        let mid_energy = (spectrum[8..24].iter().sum::<f32>() / 16.0) * sensitivity;
        let high_energy = (spectrum[24..].iter().sum::<f32>() / 40.0) * sensitivity;

        // Limiter les valeurs entre 0.0 et 1.0
        let bass_energy = bass_energy.min(1.0);
        let mid_energy = mid_energy.min(1.0);
        let high_energy = high_energy.min(1.0);

        // Donner beaucoup plus d'importance aux basses fréquences pour les flammes
        // Utiliser une fonction non linéaire pour amplifier les petits signaux
        let bass_contribution = bass_energy.powf(0.5) * 0.7; // Racine carrée pour amplifier les petits signaux
        let mid_contribution = mid_energy.powf(0.6) * 0.25;
        let high_contribution = high_energy.powf(0.7) * 0.05;

        // Calculer l'énergie totale avec une amplification supplémentaire
        let raw_energy = bass_contribution + mid_contribution + high_contribution;

        // Appliquer une courbe de réponse non linéaire pour amplifier même les petits signaux
        // Cette formule transforme les petites valeurs (0.1-0.3) en valeurs moyennes (0.4-0.6)
        let total_energy = (raw_energy * 1.5).min(1.0);

        // Log pour déboguer l'intensité sonore
        if total_energy > 0.01 {
            println!(
                "🔥 [Flames] Bass: {:.2}, Mid: {:.2}, High: {:.2}, Total: {:.2}",
                bass_energy,
                mid_energy,
                high_energy,
                total_energy
            );
        }

        // Incrémenter le compteur d'animation
        self.animation_counter += 0.5 + bass_energy * 2.0;

        // Créer une matrice de température pour simuler les flammes
        let mut temperature = vec![0.0; 128 * 128];

        // Calculer la hauteur maximale des flammes en fonction de l'intensité sonore
        // Plus le son est fort, plus les flammes sont hautes
        // Augmenter drastiquement la réactivité en rendant les flammes plus réactives
        let min_height = 20.0; // Hauteur minimale des flammes réduite
        let max_height = 127.0; // Hauteur maximale possible

        // Utiliser une fonction exponentielle pour la hauteur des flammes
        // Cela rend les flammes beaucoup plus réactives aux changements d'intensité sonore
        let height_factor = total_energy.powf(0.7); // Exposant < 1 pour amplifier les petits signaux
        let max_flame_height = min_height + height_factor * (max_height - min_height); // Entre 20 et 127 pixels

        // Mettre à jour les sources de chaleur à la base en fonction du spectre audio
        for x in 0..128 {
            // Ajouter de l'aléatoire pour un effet plus naturel
            let random_factor = 0.7 + rand() * 0.6;

            // Calculer l'énergie en fonction de la position (plus d'énergie au centre)
            let position_factor = 1.0 - ((x as f32 - 64.0).abs() / 64.0) * 0.5;

            // Combiner l'audio et l'aléatoire
            let energy = (bass_energy * 1.5 + mid_energy * 0.8 + high_energy * 0.5) * position_factor * random_factor;

            // Mettre à jour la source de chaleur
            self.heat_sources[x] = (self.heat_sources[x] * 0.8 + energy * 0.2).min(1.0);

            // Appliquer la source de chaleur à la base de la matrice de température
            let idx = (127 * 128 + x) as usize;
            temperature[idx] = self.heat_sources[x];

            // Ajouter beaucoup plus d'étincelles quand le son est détecté
            let base_chance = 0.25; // Chance de base plus élevée
            let spark_chance = base_chance + total_energy * 0.7; // Entre 0.25 et 0.95 - extrêmement réactif

            // Générer plusieurs étincelles par position
            let spark_count = (1.0 + total_energy * 3.0) as usize; // Entre 1 et 4 étincelles par position

            if rand() < spark_chance {
                for _ in 0..spark_count {
                    // Générer beaucoup plus d'étincelles avec une distribution plus large quand le son est fort
                    let spread = 15.0 + total_energy * 25.0; // Entre 15 et 40 pixels de dispersion
                    let spark_x = (x as f32 + (rand() - 0.5) * spread).max(0.0).min(127.0) as usize;

                    // Les étincelles montent beaucoup plus haut quand le son est fort
                    let height_factor = 0.4 + total_energy * 0.5; // Entre 0.4 et 0.9
                    let spark_y = (127.0 - rand() * max_flame_height * height_factor) as usize;

                    // Étincelles plus intenses avec le son
                    let intensity = 0.8 + total_energy * 0.2; // Entre 0.8 et 1.0

                    let spark_idx = spark_y * 128 + spark_x;
                    if spark_idx < temperature.len() {
                        temperature[spark_idx] = intensity;
                    }
                }
            }
        }

        // Simuler la propagation de la chaleur de bas en haut
        for y in (0..127).rev() {
            // Calculer un facteur de propagation qui diminue avec la hauteur
            // et qui est fortement influencé par l'intensité sonore
            let height_factor = 1.0 - (127.0 - y as f32) / max_flame_height;

            // Propagation plus agressive quand le son est fort
            let propagation_factor = if height_factor > 0.0 {
                // Entre 0.88 et 0.98 selon l'intensité sonore - plus réactif
                0.88 + total_energy * 0.1
            } else {
                // Propagation plus forte même au-delà de la hauteur maximale si le son est fort
                0.5 + total_energy * 0.2 // Entre 0.5 et 0.7
            };

            // Ajouter des pulsations basées sur l'intensité des basses
            let bass_pulse = (self.animation_counter * 0.05).sin() * bass_energy * 0.05;

            for x in 0..128 {
                let idx = y * 128 + x;
                let idx_below = (y + 1) * 128 + x;

                // Propager la chaleur vers le haut avec diffusion
                // Ajouter l'effet de pulsation des basses
                let mut new_temp = temperature[idx_below] * (propagation_factor + bass_pulse);

                // Ajouter de la diffusion latérale
                if x > 0 {
                    new_temp += temperature[idx_below - 1] * 0.05;
                }
                if x < 127 {
                    new_temp += temperature[idx_below + 1] * 0.05;
                }

                // Ajouter un peu de mouvement aléatoire
                // Plus d'agitation quand le son est fort
                let wind_strength = 0.02 + total_energy * 0.05;
                let wind = (self.animation_counter * 0.01).sin() * wind_strength + rand() * wind_strength;
                let wind_x = (x as i32 + wind.signum() as i32).max(0).min(127) as usize;
                let wind_idx = (y + 1) * 128 + wind_x;
                if wind_idx < temperature.len() {
                    new_temp += temperature[wind_idx] * wind.abs() * 5.0;
                }

                // Appliquer le refroidissement (beaucoup moins de refroidissement quand le son est détecté)
                // Utiliser une fonction exponentielle pour réduire drastiquement le refroidissement même avec un son faible
                let min_cooling_factor = 0.2; // Facteur minimal de refroidissement (20%)
                let cooling_factor = min_cooling_factor + (1.0 - min_cooling_factor) * (1.0 - total_energy).powf(2.0);
                let cooling = self.cooling_map[idx] * cooling_factor;

                // Appliquer un plancher minimal de température pour maintenir des flammes visibles même avec un son très faible
                let min_temp = if y > 100 { 0.1 } else { 0.0 }; // Maintenir un minimum de chaleur à la base
                temperature[idx] = (new_temp - cooling).max(min_temp);
            }
        }

        // Dessiner les flammes
        for y in 0..128 {
            for x in 0..128 {
                let idx = y * 128 + x;
                let temp = temperature[idx];

                if temp > 0.01 {
                    let (r, g, b) = self.get_flame_color(temp, x, y);
                    let frame_idx = idx * 3;

                    frame[frame_idx] = (r * 255.0) as u8;
                    frame[frame_idx + 1] = (g * 255.0) as u8;
                    frame[frame_idx + 2] = (b * 255.0) as u8;
                } else {
                    // Pixel noir (fond)
                    let frame_idx = idx * 3;
                    frame[frame_idx] = 0;
                    frame[frame_idx + 1] = 0;
                    frame[frame_idx + 2] = 0;
                }
            }
        }
    }

    fn set_color_mode(&mut self, mode: &str) {
        println!("🔥 [Flames] Setting color mode to '{}'", mode);
        // Le mode de couleur est géré via GLOBAL_COLOR_CONFIG
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "🔥 [Flames] Setting custom color to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
        // La couleur personnalisée est gérée via GLOBAL_COLOR_CONFIG
    }
}


// Effet 5: Pluie
struct Rain {
    drops: Vec<RainDrop>,
    animation_counter: f32,
    color_mode: String,
    custom_color: (f32, f32, f32),
}

struct RainDrop {
    x: f32,      // Position horizontale
    y: f32,      // Position verticale
    length: f32, // Longueur de la goutte
    speed: f32,  // Vitesse de chute
    brightness: f32, // Luminosité de la goutte
}

impl Rain {
    fn new() -> Self {
        let mut drops = Vec::with_capacity(100);

        // Initialiser quelques gouttes de pluie
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
            color_mode: "ocean".to_string(), // Mode couleur par défaut pour la pluie
            custom_color: (0.0, 0.5, 1.0),  // Bleu clair par défaut
        }
    }

    fn get_rain_color(&self, brightness: f32, y_pos: f32) -> (f32, f32, f32) {
        match self.color_mode.as_str() {
            "custom" => {
                let (r, g, b) = self.custom_color;
                (r * brightness, g * brightness, b * brightness)
            }
            "rainbow" => {
                // Couleur arc-en-ciel basée sur la position verticale
                let hue = (y_pos / 128.0) * 360.0;
                let (r, g, b) = hsv_to_rgb(hue, 0.7, brightness);
                (r, g, b)
            }
            "fire" => {
                // Couleur de feu (rouge-orange)
                let r = brightness;
                let g = brightness * 0.5;
                let b = brightness * 0.1;
                (r, g, b)
            }
            "ocean" => {
                // Dégradé de bleus
                let r = brightness * 0.1;
                let g = brightness * 0.5;
                let b = brightness;
                (r, g, b)
            }
            "sunset" => {
                // Dégradé coucher de soleil
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
                // Bleu par défaut
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
        // Analyser le spectre audio pour influencer la pluie
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;

        // Amplifier la sensibilité
        let sensitivity = 4.0;
        let bass_energy = (bass_energy * sensitivity).min(1.0);
        let mid_energy = (mid_energy * sensitivity).min(1.0);
        let high_energy = (high_energy * sensitivity).min(1.0);

        // L'intensité sonore totale influence la quantité et la vitesse des gouttes
        let total_energy = (bass_energy * 0.5 + mid_energy * 0.3 + high_energy * 0.2).min(1.0);

        // Effacer le frame
        for pixel in frame.chunks_exact_mut(3) {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        }

        // Incrémenter le compteur d'animation
        self.animation_counter += 0.1;

        // Ajouter de nouvelles gouttes en fonction de l'intensité sonore
        let drop_chance = 0.2 + total_energy * 0.5; // Entre 0.2 et 0.7
        if rand() < drop_chance {
            let num_new_drops = (1.0 + total_energy * 5.0) as usize; // Entre 1 et 6 nouvelles gouttes

            for _ in 0..num_new_drops {
                if self.drops.len() < 200 { // Limiter le nombre total de gouttes
                    self.drops.push(RainDrop {
                        x: rand() * 128.0,
                        y: -10.0 - rand() * 10.0, // Commencer au-dessus de l'écran
                        length: 3.0 + rand() * 12.0 + total_energy * 10.0, // Longueur influencée par le son
                        speed: 1.0 + rand() * 2.0 + total_energy * 3.0,   // Vitesse influencée par le son
                        brightness: 0.3 + rand() * 0.5 + total_energy * 0.2, // Luminosité influencée par le son
                    });
                }
            }
        }

        // Mettre à jour et dessiner les gouttes de pluie
        let mut i = 0;
        while i < self.drops.len() {
            // CORRECTION : Séparer l'accès mutable et immutable
            {
                let drop = &mut self.drops[i];
                drop.y += drop.speed;

                let wind_effect = (self.animation_counter * 0.05).sin() * mid_energy * 0.5;
                drop.x += wind_effect;

                if drop.x < 0.0 {
                    drop.x = 0.0;
                } else if drop.x >= 128.0 {
                    drop.x = 127.9;
                }
            }

            // CORRECTION : Accès immutable séparé
            let drop = &self.drops[i];
            let x = drop.x as usize;
            let start_y = (drop.y - drop.length).max(0.0) as usize;
            let end_y = drop.y.min(127.0) as usize;

            // Dessiner la goutte avec un dégradé de luminosité du haut vers le bas
            for y in start_y..=end_y {
                if y < 128 {
                    let relative_pos = (y as f32 - start_y as f32) / (end_y as f32 - start_y as f32 + 1.0);
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

        // Ajouter des éclaboussures au sol quand les gouttes touchent le bas
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

    fn set_color_mode(&mut self, mode: &str) {
        self.color_mode = mode.to_string();
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        self.custom_color = (r, g, b);
        self.color_mode = "custom".to_string();
    }
}

// Effet 6: Applaudimètre
pub struct Applaudimetre {
    current_level: f32,
    max_level: f32,
    max_hold_time: f32,
    smoothed_level: f32,
    peak_history: Vec<f32>,
    // Nouveaux champs pour les améliorations
    animation_time: f32,
    level_history: Vec<f32>,          // Historique pour les effets de traînée
    peak_sparkles: Vec<PeakSparkle>,  // Étincelles autour du pic
    sensitivity: f32,                 // Sensibilité ajustable
    auto_gain: f32,                   // Gain automatique adaptatif
    background_pulse: f32,            // Pulsation de fond
}

// Structure pour les étincelles autour du pic
struct PeakSparkle {
    x: f32,
    y: f32,
    life: f32,
    brightness: f32,
    color: (f32, f32, f32),
}

impl Applaudimetre {
    pub fn new() -> Self {
        Self {
            current_level: 0.0,
            max_level: 0.0,
            max_hold_time: 0.0,
            smoothed_level: 0.0,
            peak_history: vec![0.0; 30],
            animation_time: 0.0,
            level_history: vec![0.0; 128],  // Historique de 128 frames
            peak_sparkles: Vec::new(),
            sensitivity: 3.0,              // Sensibilité de base augmentée
            auto_gain: 1.0,                // Gain adaptatif initial
            background_pulse: 0.0,
        }
    }

    fn get_color_for_level(&self, level: f32, is_max_indicator: bool) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        if is_max_indicator {
            // Couleurs plus brillantes et contrastées pour l'indicateur de maximum
            match color_mode.mode.as_str() {
                "rainbow" => {
                    // Arc-en-ciel animé pour le maximum
                    let hue = (self.animation_time * 0.02) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    (r, g, b)
                }
                "fire" => (1.0, 1.0, 0.2),    // Jaune-blanc brillant
                "ocean" => (0.2, 1.0, 1.0),   // Cyan-blanc brillant
                "sunset" => (1.0, 0.6, 0.1),  // Orange brillant
                "custom" => {
                    let (r, g, b) = color_mode.custom_color;
                    ((r * 1.8).min(1.0), (g * 1.8).min(1.0), (b * 1.8).min(1.0))
                }
                _ => (1.0, 1.0, 1.0),
            }
        } else {
            // Couleurs améliorées pour la barre principale avec plus de contraste
            match color_mode.mode.as_str() {
                "rainbow" => {
                    // Dégradé plus dynamique : Bleu -> Vert -> Jaune -> Rouge
                    if level < 0.25 {
                        // Bleu vers cyan
                        let t = level * 4.0;
                        (0.0, t * 0.5, 1.0)
                    } else if level < 0.5 {
                        // Cyan vers vert
                        let t = (level - 0.25) * 4.0;
                        (0.0, 0.5 + t * 0.5, 1.0 - t)
                    } else if level < 0.75 {
                        // Vert vers jaune
                        let t = (level - 0.5) * 4.0;
                        (t, 1.0, 0.0)
                    } else {
                        // Jaune vers rouge
                        let t = (level - 0.75) * 4.0;
                        (1.0, 1.0 - t, 0.0)
                    }
                }
                "fire" => {
                    // Flammes plus réalistes
                    if level < 0.3 {
                        let t = level / 0.3;
                        (t * 0.8, 0.0, 0.0)  // Rouge sombre
                    } else if level < 0.6 {
                        let t = (level - 0.3) / 0.3;
                        (0.8 + t * 0.2, t * 0.4, 0.0)  // Rouge vers orange
                    } else {
                        let t = (level - 0.6) / 0.4;
                        (1.0, 0.4 + t * 0.6, t * 0.3)  // Orange vers jaune
                    }
                }
                "ocean" => {
                    // Océan plus profond
                    if level < 0.5 {
                        let t = level * 2.0;
                        (0.0, t * 0.3, 0.4 + t * 0.4)  // Bleu profond vers bleu
                    } else {
                        let t = (level - 0.5) * 2.0;
                        (t * 0.2, 0.3 + t * 0.5, 0.8 + t * 0.2)  // Bleu vers cyan
                    }
                }
                "sunset" => {
                    // Coucher de soleil plus dramatique
                    if level < 0.4 {
                        let t = level / 0.4;
                        (0.3 + t * 0.4, 0.1 * t, 0.5 + t * 0.3)  // Violet vers rouge
                    } else {
                        let t = (level - 0.4) / 0.6;
                        (0.7 + t * 0.3, 0.1 + t * 0.7, 0.8 - t * 0.8)  // Rouge vers orange
                    }
                }
                "custom" => {
                    let (r, g, b) = color_mode.custom_color;
                    let intensity = level.max(0.1);
                    (r * intensity, g * intensity, b * intensity)
                }
                _ => hsv_to_rgb(0.3, 1.0, level.max(0.3)),
            }
        }
    }

    fn calculate_audio_level(&mut self, spectrum: &[f32]) -> f32 {
        // Amélioration de la sensibilité avec adaptation automatique
        let bass_weight = 0.5;  // Plus d'importance aux basses
        let mid_weight = 0.35;
        let high_weight = 0.15;

        let bass_level = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_level = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_level = spectrum[24..].iter().sum::<f32>() / 40.0;

        // Appliquer la sensibilité
        let raw_level = (bass_level * bass_weight + mid_level * mid_weight + high_level * high_weight) * self.sensitivity;

        // Gain adaptatif automatique
        if raw_level > 0.01 {
            // Ajuster le gain automatique selon le niveau moyen
            let avg_recent = self.peak_history.iter().sum::<f32>() / self.peak_history.len() as f32;
            if avg_recent < 0.2 {
                self.auto_gain = (self.auto_gain + 0.01).min(2.0);  // Augmenter le gain si trop faible
            } else if avg_recent > 0.8 {
                self.auto_gain = (self.auto_gain - 0.01).max(0.5);  // Diminuer le gain si trop fort
            }
        }

        // Appliquer le gain adaptatif et une courbe de réponse
        let final_level = (raw_level * self.auto_gain).powf(0.6);  // Courbe pour amplifier les petits signaux
        final_level.min(1.0)
    }

    fn update_sparkles(&mut self) {
        // Mettre à jour les étincelles existantes
        self.peak_sparkles.retain_mut(|sparkle| {
            sparkle.life -= 0.03;
            sparkle.y -= 0.5;  // Les étincelles montent
            sparkle.x += (rand() - 0.5) * 0.3;  // Légère dérive horizontale
            sparkle.life > 0.0
        });

        // Ajouter de nouvelles étincelles si le niveau est élevé
        if self.current_level > 0.3 && rand() < 0.4 {
            let bar_center = 64.0;
            let max_y = 127.0 - self.max_level * 127.0;

            for _ in 0..(1 + (self.current_level * 3.0) as usize) {
                self.peak_sparkles.push(PeakSparkle {
                    x: bar_center + (rand() - 0.5) * 50.0,
                    y: max_y + (rand() - 0.5) * 10.0,
                    life: 0.5 + rand() * 0.5,
                    brightness: 0.6 + rand() * 0.4,
                    color: self.get_color_for_level(self.current_level, false),
                });
            }
        }
    }
}

impl Effect for Applaudimetre {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Calculer le niveau audio avec les améliorations
        let raw_level = self.calculate_audio_level(spectrum);

        // Lissage adaptatif plus sophistiqué
        let smoothing = if raw_level > self.smoothed_level { 0.4 } else { 0.85 };
        self.smoothed_level = self.smoothed_level * smoothing + raw_level * (1.0 - smoothing);
        self.current_level = self.smoothed_level;

        // Mettre à jour l'historique des niveaux pour les effets de traînée
        self.level_history.remove(0);
        self.level_history.push(self.current_level);

        // Gestion du pic améliorée
        if self.peak_history.len() > 0 {
            self.peak_history.remove(0);
            self.peak_history.push(self.current_level);
        }

        let recent_max = self.peak_history.iter().cloned().fold(0.0f32, f32::max);

        if self.current_level > self.max_level {
            self.max_level = self.current_level;
            self.max_hold_time = 0.0;
        } else {
            self.max_hold_time += 1.0 / 60.0;
            if self.max_hold_time >= 5.0 {
                let decay_rate = 0.008;  // Décroissance légèrement plus rapide
                self.max_level = (self.max_level - decay_rate).max(recent_max);
                if self.max_level <= recent_max {
                    self.max_hold_time = 0.0;
                }
            }
        }

        // Mettre à jour les animations
        self.animation_time += 1.0 + self.current_level * 2.0;
        self.background_pulse = (self.animation_time * 0.05).sin() * 0.1 + 0.9;
        self.update_sparkles();

        // Logs de débogage améliorés
        static mut APPLAUD_FRAME_COUNT: u64 = 0;
        unsafe {
            APPLAUD_FRAME_COUNT += 1;
            if APPLAUD_FRAME_COUNT % 30 == 0 {
                println!(
                    "👏 [Applaudimètre] Level: {:.3}, Max: {:.3}, Hold: {:.1}s, Gain: {:.2}, Sparkles: {}",
                    self.current_level, self.max_level, self.max_hold_time, self.auto_gain, self.peak_sparkles.len()
                );
            }
        }

        frame.fill(0);

        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = i % 128;
            let y = i / 128;
            let bar_left = 40;    // Barre légèrement plus large
            let bar_right = 88;
            let bar_center = (bar_left + bar_right) / 2;

            if x >= bar_left && x < bar_right {
                let y_pos = (127 - y) as f32 / 127.0;
                let bar_height = self.current_level.min(1.0);

                // Effet de traînée - afficher l'historique des niveaux avec transparence
                let history_index = ((x - bar_left) as f32 / (bar_right - bar_left) as f32 * self.level_history.len() as f32) as usize;
                if history_index < self.level_history.len() {
                    let historical_level = self.level_history[history_index];
                    if y_pos <= historical_level && historical_level > 0.05 {
                        let trail_intensity = 0.2 + (historical_level / self.current_level.max(0.1)) * 0.3;
                        let (r, g, b) = self.get_color_for_level(y_pos, false);

                        pixel[0] = (r * trail_intensity * 255.0) as u8;
                        pixel[1] = (g * trail_intensity * 255.0) as u8;
                        pixel[2] = (b * trail_intensity * 255.0) as u8;
                    }
                }

                // Barre principale avec effet de brillance
                if y_pos <= bar_height && bar_height > 0.0 {
                    let level_factor = y_pos / bar_height.max(0.01);

                    // Effet de brillance au centre de la barre
                    let distance_from_center = ((x - bar_center) as f32 / ((bar_right - bar_left) / 2) as f32).abs();
                    let center_glow = (1.0 - distance_from_center).max(0.0);

                    let brightness = 0.7 + level_factor * 0.3 + center_glow * 0.3;
                    let (r, g, b) = self.get_color_for_level(level_factor, false);

                    // Ajouter une pulsation basée sur l'intensité
                    let pulse = 1.0 + self.current_level * (self.animation_time * 0.1).sin() * 0.2;

                    pixel[0] = (r * brightness * pulse * 255.0).min(255.0) as u8;
                    pixel[1] = (g * brightness * pulse * 255.0).min(255.0) as u8;
                    pixel[2] = (b * brightness * pulse * 255.0).min(255.0) as u8;
                }

                // Indicateur de maximum avec animation améliorée
                let max_y = (127.0 - self.max_level * 127.0) as usize;
                if y >= max_y.saturating_sub(2) && y <= max_y.saturating_add(2) && self.max_level > 0.05 {
                    let (r, g, b) = self.get_color_for_level(self.max_level, true);

                    // Animation de clignotement plus sophistiquée
                    let blink_factor = if self.max_hold_time < 5.0 {
                        // Clignotement rapide avec variation
                        let base_blink = 0.8 + 0.2 * (self.max_hold_time * 8.0).sin();
                        let pulse_blink = 1.0 + 0.3 * (self.animation_time * 0.15).sin();
                        base_blink * pulse_blink
                    } else {
                        // Clignotement de décroissance
                        0.6 + 0.4 * (self.max_hold_time * 2.0).sin().abs()
                    };

                    // Effet de largeur variable
                    let width_factor = if (y as i32 - max_y as i32).abs() <= 1 { 1.0 } else { 0.7 };

                    pixel[0] = (r * blink_factor * width_factor * 255.0) as u8;
                    pixel[1] = (g * blink_factor * width_factor * 255.0) as u8;
                    pixel[2] = (b * blink_factor * width_factor * 255.0) as u8;
                }

                // Graduations améliorées avec effet de profondeur
                if x == bar_left + 1 || x == bar_right - 2 {
                    for grad in 0..11 {  // Plus de graduations
                        let grad_y = (127.0 - (grad as f32 * 127.0 / 10.0)) as usize;
                        if y >= grad_y.saturating_sub(1) && y <= grad_y.saturating_add(1) {
                            let intensity = if grad == 10 { 0.9 } else if grad % 2 == 0 { 0.6 } else { 0.3 };
                            let background_effect = intensity * self.background_pulse;

                            pixel[0] = (150.0 * background_effect) as u8;
                            pixel[1] = (150.0 * background_effect) as u8;
                            pixel[2] = (150.0 * background_effect) as u8;
                        }
                    }
                }
            }

            // Cadre avec effet de lueur
            let is_frame = ((x == bar_left - 1 || x == bar_right) && y < 128) ||
                          ((y == 0 || y == 127) && x >= bar_left - 1 && x <= bar_right);

            if is_frame {
                let glow_intensity = 80.0 + self.current_level * 50.0;
                pixel[0] = glow_intensity as u8;
                pixel[1] = glow_intensity as u8;
                pixel[2] = glow_intensity as u8;
            }
        });

        // Dessiner les étincelles par-dessus
        for sparkle in &self.peak_sparkles {
            let x = sparkle.x as usize;
            let y = sparkle.y as usize;

            if x < 128 && y < 128 {
                let idx = (y * 128 + x) * 3;
                if idx + 2 < frame.len() {
                    let intensity = sparkle.brightness * sparkle.life;

                    // Additive blending pour les étincelles
                    frame[idx] = ((sparkle.color.0 * intensity * 255.0) as u8).saturating_add(frame[idx]);
                    frame[idx + 1] = ((sparkle.color.1 * intensity * 255.0) as u8).saturating_add(frame[idx + 1]);
                    frame[idx + 2] = ((sparkle.color.2 * intensity * 255.0) as u8).saturating_add(frame[idx + 2]);
                }
            }
        }
    }

    fn set_color_mode(&mut self, mode: &str) {
        println!("   Applaudimètre: color mode set to '{}'", mode);
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "   Applaudimètre: custom color set to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
    }
}

// Effet 7: Pluie d'Étoiles (Starfall)
pub struct Starfall {
    stars: Vec<FallingStar>,
    animation_time: f32,
}

struct FallingStar {
    x: f32,
    y: f32,
    speed: f32,
    brightness: f32,
    size: f32,
    color: (f32, f32, f32),
    trail_length: f32,
}

impl Starfall {
    pub fn new() -> Self {
        let mut stars = Vec::new();

        // Créer quelques étoiles initiales
        for _ in 0..20 {
            stars.push(FallingStar {
                x: rand() * 150.0 - 20.0,  // Commencer un peu à gauche
                y: rand() * 150.0 - 20.0,  // Commencer un peu en haut
                speed: 1.0 + rand() * 2.0,
                brightness: 0.3 + rand() * 0.7,
                size: 1.0 + rand() * 2.0,
                color: (0.8 + rand() * 0.2, 0.8 + rand() * 0.2, 1.0), // Blanc-bleu
                trail_length: 3.0 + rand() * 5.0,
            });
        }

        Self {
            stars,
            animation_time: 0.0,
        }
    }

    fn get_star_color(&self, base_color: (f32, f32, f32), brightness: f32) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        match color_mode.mode.as_str() {
            "rainbow" => {
                // Couleurs arc-en-ciel qui changent avec le temps
                let hue = (self.animation_time * 0.01 + brightness) % 1.0;
                hsv_to_rgb(hue, 0.8, brightness)
            }
            "fire" => {
                // Étoiles de feu (rouge-orange-jaune)
                let intensity = brightness;
                if intensity < 0.5 {
                    (intensity * 2.0, intensity * 0.5, 0.0)
                } else {
                    (1.0, 0.5 + (intensity - 0.5), (intensity - 0.5) * 0.5)
                }
            }
            "ocean" => {
                // Étoiles océan (bleu-cyan)
                (brightness * 0.2, brightness * 0.7, brightness)
            }
            "sunset" => {
                // Étoiles coucher de soleil (orange-rose)
                (brightness, brightness * 0.6, brightness * 0.8)
            }
            "custom" => {
                let (r, g, b) = color_mode.custom_color;
                (r * brightness, g * brightness, b * brightness)
            }
            _ => {
                // Blanc-bleu par défaut
                (base_color.0 * brightness, base_color.1 * brightness, base_color.2 * brightness)
            }
        }
    }
}

impl Effect for Starfall {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Analyser l'intensité sonore
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;

        // Amplifier la sensibilité
        let sensitivity = 4.0;
        let bass = (bass_energy * sensitivity).min(1.0);
        let mid = (mid_energy * sensitivity).min(1.0);
        let high = (high_energy * sensitivity).min(1.0);

        // Intensité totale pour contrôler le nombre d'étoiles
        let total_energy = (bass * 0.4 + mid * 0.4 + high * 0.2).min(1.0);

        self.animation_time += 0.5;

        // Effacer l'écran
        frame.fill(0);

        // Mettre à jour les étoiles existantes
        let mut i = 0;
        while i < self.stars.len() {
            let star = &mut self.stars[i];

            // Mouvement diagonal (haut-gauche vers bas-droit)
            star.x += star.speed * 0.7;  // Mouvement horizontal
            star.y += star.speed;        // Mouvement vertical (plus rapide)

            // Ajouter un peu de variation basée sur l'audio
            if total_energy > 0.1 {
                star.x += (rand() - 0.5) * total_energy * 0.5;
                star.y += total_energy * 0.3;
            }

            // Supprimer les étoiles qui sortent de l'écran
            if star.x > 150.0 || star.y > 150.0 {
                self.stars.swap_remove(i);
            } else {
                i += 1;
            }
        }

        // Ajouter de nouvelles étoiles selon l'intensité sonore
        let base_spawn_chance = 0.1;
        let audio_spawn_boost = total_energy * 0.8;  // Jusqu'à 80% de chance supplémentaire
        let spawn_chance = base_spawn_chance + audio_spawn_boost;

        if rand() < spawn_chance {
            // Nombre d'étoiles à créer (plus de son = plus d'étoiles)
            let star_count = (1.0 + total_energy * 5.0) as usize;  // 1 à 6 étoiles

            for _ in 0..star_count {
                if self.stars.len() < 150 {  // Limite pour éviter la surcharge
                    // Spawn en haut à gauche avec un peu de variation
                    let spawn_x = -20.0 + rand() * 40.0;  // Entre -20 et 20
                    let spawn_y = -20.0 + rand() * 40.0;  // Entre -20 et 20

                    // Vitesse influencée par l'audio
                    let base_speed = 1.0 + rand() * 2.0;
                    let audio_speed_boost = total_energy * 2.0;
                    let final_speed = base_speed + audio_speed_boost;

                    // Taille et luminosité influencées par l'audio
                    let size = 1.0 + rand() * 1.5 + total_energy * 2.0;
                    let brightness = 0.4 + rand() * 0.4 + total_energy * 0.2;

                    // Couleur de base (blanc-bleu étoilé)
                    let base_color = (
                        0.8 + rand() * 0.2,
                        0.8 + rand() * 0.2,
                        1.0
                    );

                    // Traînée plus longue avec plus de son
                    let trail = 3.0 + rand() * 4.0 + total_energy * 5.0;

                    self.stars.push(FallingStar {
                        x: spawn_x,
                        y: spawn_y,
                        speed: final_speed,
                        brightness,
                        size,
                        color: base_color,
                        trail_length: trail,
                    });
                }
            }
        }

        // Log de débogage
        static mut STARFALL_FRAME_COUNT: u64 = 0;
        unsafe {
            STARFALL_FRAME_COUNT += 1;
            if STARFALL_FRAME_COUNT % 60 == 0 && total_energy > 0.01 {
                println!(
                    "⭐ [Starfall] Energy: {:.2}, Stars: {}, Spawn: {:.0}%",
                    total_energy,
                    self.stars.len(),
                    spawn_chance * 100.0
                );
            }
        }

        // Dessiner les étoiles avec leurs traînées
        for star in &self.stars {
            // Dessiner la traînée (points derrière l'étoile)
            let trail_points = (star.trail_length * star.brightness) as usize;
            for i in 0..trail_points {
                let trail_factor = i as f32 / trail_points as f32;

                // Position de la traînée (inverse du mouvement)
                let trail_x = star.x - star.speed * 0.7 * trail_factor * 1.5;
                let trail_y = star.y - star.speed * trail_factor * 1.5;

                let x = trail_x as i32;
                let y = trail_y as i32;

                if x >= 0 && x < 128 && y >= 0 && y < 128 {
                    let idx = (y as usize * 128 + x as usize) * 3;
                    if idx + 2 < frame.len() {
                        // Intensité de la traînée diminue vers l'arrière
                        let trail_intensity = star.brightness * (1.0 - trail_factor) * 0.4;
                        let (r, g, b) = self.get_star_color(star.color, trail_intensity);

                        frame[idx] = (r * 255.0) as u8;
                        frame[idx + 1] = (g * 255.0) as u8;
                        frame[idx + 2] = (b * 255.0) as u8;
                    }
                }
            }

            // Dessiner l'étoile principale (plus brillante)
            let star_size = star.size as i32;
            for dy in -star_size..=star_size {
                for dx in -star_size..=star_size {
                    let x = star.x as i32 + dx;
                    let y = star.y as i32 + dy;

                    if x >= 0 && x < 128 && y >= 0 && y < 128 {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();

                        // Forme d'étoile simple (plus brillant au centre)
                        if distance <= star.size {
                            let idx = (y as usize * 128 + x as usize) * 3;
                            if idx + 2 < frame.len() {
                                // Intensité diminue du centre vers les bords
                                let center_falloff = (1.0 - distance / star.size).max(0.0);
                                let final_brightness = star.brightness * center_falloff;

                                let (r, g, b) = self.get_star_color(star.color, final_brightness);

                                // Additive blending pour un effet lumineux
                                frame[idx] = ((r * 255.0) as u8).saturating_add(frame[idx]);
                                frame[idx + 1] = ((g * 255.0) as u8).saturating_add(frame[idx + 1]);
                                frame[idx + 2] = ((b * 255.0) as u8).saturating_add(frame[idx + 2]);
                            }
                        }

                        // Ajouter des "rayons" d'étoile (croix)
                        if (dx == 0 && dy.abs() <= star_size + 1) || (dy == 0 && dx.abs() <= star_size + 1) {
                            let idx = (y as usize * 128 + x as usize) * 3;
                            if idx + 2 < frame.len() {
                                let ray_brightness = star.brightness * 0.6;
                                let (r, g, b) = self.get_star_color(star.color, ray_brightness);

                                frame[idx] = ((r * 150.0) as u8).saturating_add(frame[idx]);
                                frame[idx + 1] = ((g * 150.0) as u8).saturating_add(frame[idx + 1]);
                                frame[idx + 2] = ((b * 150.0) as u8).saturating_add(frame[idx + 2]);
                            }
                        }
                    }
                }
            }
        }

        // Effet d'explosion d'étoiles lors de pics sonores très forts
        if total_energy > 0.8 && rand() < 0.2 {
            // Créer une "supernova" temporaire
            for _ in 0..8 {
                let explosion_x = 20.0 + rand() * 88.0;  // Position aléatoire sur l'écran
                let explosion_y = 20.0 + rand() * 88.0;

                // Dessiner une explosion étoilée
                for ray in 0..8 {
                    let angle = (ray as f32 / 8.0) * 6.28;
                    let length = 5.0 + rand() * 10.0;

                    for step in 0..(length as i32) {
                        let x = (explosion_x + angle.cos() * step as f32) as i32;
                        let y = (explosion_y + angle.sin() * step as f32) as i32;

                        if x >= 0 && x < 128 && y >= 0 && y < 128 {
                            let idx = (y as usize * 128 + x as usize) * 3;
                            if idx + 2 < frame.len() {
                                let explosion_brightness = (1.0 - step as f32 / length) * total_energy;
                                let (r, g, b) = self.get_star_color((1.0, 1.0, 1.0), explosion_brightness);

                                frame[idx] = ((r * 200.0) as u8).saturating_add(frame[idx]);
                                frame[idx + 1] = ((g * 200.0) as u8).saturating_add(frame[idx + 1]);
                                frame[idx + 2] = ((b * 200.0) as u8).saturating_add(frame[idx + 2]);
                            }
                        }
                    }
                }
            }
        }
    }

    fn set_color_mode(&mut self, mode: &str) {
        println!("⭐ [Starfall] Color mode set to '{}'", mode);
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "⭐ [Starfall] Custom color set to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
    }
}


// Effet 8: Coeur Battant
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
            beat_frequency: 60.0,  // 60 BPM par defaut
        }
    }

    fn get_heart_color(&self, intensity: f32) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        match color_mode.mode.as_str() {
            "rainbow" => {
                // Arc-en-ciel qui pulse
                let hue = (self.animation_time * 0.01) % 1.0;
                hsv_to_rgb(hue, 0.8, intensity)
            }
            "fire" => {
                // Coeur de feu (rouge intense vers jaune)
                if intensity < 0.5 {
                    (intensity * 2.0, 0.0, 0.0)  // Rouge profond
                } else {
                    (1.0, (intensity - 0.5) * 2.0, 0.0)  // Rouge vers orange
                }
            }
            "ocean" => {
                // Coeur d'ocean (bleu vers cyan)
                (intensity * 0.2, intensity * 0.6, intensity)
            }
            "sunset" => {
                // Coeur coucher de soleil (rose vers orange)
                (intensity, intensity * 0.4, intensity * 0.6)
            }
            "custom" => {
                let (r, g, b) = color_mode.custom_color;
                (r * intensity, g * intensity, b * intensity)
            }
            _ => {
                // Rouge classique pour un coeur
                (intensity, intensity * 0.2, intensity * 0.2)
            }
        }
    }

    // Fonction pour dessiner un coeur mathematique
    fn is_inside_heart(&self, x: f32, y: f32, center_x: f32, center_y: f32, scale: f32) -> f32 {
        // Normaliser les coordonnees par rapport au centre et a l'echelle
        let nx = (x - center_x) / scale;
        let ny = (y - center_y) / scale;

        // Equation mathematique d'un coeur
        // (x^2 + y^2 - 1)^3 - x^2 * y^3 <= 0
        let x2 = nx * nx;
        let y2 = ny * ny;
        let heart_eq = (x2 + y2 - 1.0).powi(3) - x2 * ny.powi(3);

        if heart_eq <= 0.0 {
            // A l'interieur du coeur, calculer l'intensite basee sur la distance du bord
            let distance_factor = (-heart_eq).min(0.5) / 0.5;
            return distance_factor.max(0.3);  // Minimum d'intensite pour la visibilite
        }

        0.0  // A l'exterieur du coeur
    }
}

impl Effect for Heartbeat {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Analyser l'intensite sonore
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;

        // Amplifier la sensibilite
        let sensitivity = 5.0;
        let bass = (bass_energy * sensitivity).min(1.0);
        let mid = (mid_energy * sensitivity).min(1.0);
        let high = (high_energy * sensitivity).min(1.0);

        // Intensite totale (privilegier les basses pour le rythme)
        let total_energy = (bass * 0.6 + mid * 0.3 + high * 0.1).min(1.0);

        // Calculer la frequence des battements (40 BPM a 140 BPM)
        self.beat_frequency = 40.0 + total_energy * 100.0;
        let beat_interval = 60.0 / self.beat_frequency;  // Intervalle en secondes

        self.animation_time += 1.0 / 60.0;  // En supposant 60 FPS

        // Declencher un battement
        if self.animation_time - self.last_beat_time >= beat_interval {
            self.last_beat_time = self.animation_time;
            self.beat_phase = 0.0;

            // Creer un anneau de pulsation
            self.pulse_rings.push(PulseRing {
                radius: 0.0,
                life: 1.0,
                intensity: 0.3 + total_energy * 0.7,
                color: self.get_heart_color(1.0),
            });
        }

        // Mise a jour de la phase de battement (systole et diastole)
        self.beat_phase += (self.beat_frequency / 60.0) * 0.15;  // Vitesse de l'animation

        // Calcul de l'intensite du battement (double pic pour imiter un vrai coeur)
        let double_beat = if self.beat_phase % 1.0 < 0.3 {
            // Premier battement (systole)
            ((self.beat_phase % 1.0) * 10.0).sin().max(0.0)
        } else if self.beat_phase % 1.0 < 0.5 {
            // Deuxieme battement (plus faible)
            (((self.beat_phase % 1.0) - 0.3) * 15.0).sin().max(0.0) * 0.6
        } else {
            // Repos (diastole)
            0.2
        };

        self.beat_intensity = 0.4 + double_beat * (0.3 + total_energy * 0.3);

        // Taille du coeur basee sur l'intensite et le battement
        let base_size = 15.0 + total_energy * 25.0;  // 15 a 40 pixels
        self.heart_size = base_size * (0.8 + self.beat_intensity * 0.4);

        // Mise a jour des anneaux de pulsation
        self.pulse_rings.retain_mut(|ring| {
            ring.radius += 2.0 + total_energy * 3.0;
            ring.life -= 0.02;
            ring.intensity *= 0.98;
            ring.life > 0.0 && ring.radius < 100.0
        });

        // Log de debogage
        static mut HEART_FRAME_COUNT: u64 = 0;
        unsafe {
            HEART_FRAME_COUNT += 1;
            if HEART_FRAME_COUNT % 60 == 0 && total_energy > 0.01 {
                println!(
                    "💖 [Heartbeat] BPM: {:.0}, Size: {:.1}, Intensity: {:.2}, Rings: {}",
                    self.beat_frequency, self.heart_size, self.beat_intensity, self.pulse_rings.len()
                );
            }
        }

        // Effacer l'ecran avec un fond tres sombre
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

                        frame[idx] = ((ring.color.0 * ring_intensity * 255.0) as u8).saturating_add(frame[idx]);
                        frame[idx + 1] = ((ring.color.1 * ring_intensity * 255.0) as u8).saturating_add(frame[idx + 1]);
                        frame[idx + 2] = ((ring.color.2 * ring_intensity * 255.0) as u8).saturating_add(frame[idx + 2]);
                    }
                }
            }
        }

        // Dessiner le coeur principal
        for y in 0..128 {
            for x in 0..128 {
                let heart_intensity = self.is_inside_heart(
                    x as f32,
                    y as f32,
                    center_x,
                    center_y,
                    self.heart_size
                );

                if heart_intensity > 0.0 {
                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        // Intensite finale combinant la forme du coeur et le battement
                        let final_intensity = heart_intensity * self.beat_intensity;

                        // Effet de gradient du centre vers l'exterieur
                        let distance_from_center = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                        let center_glow = (1.0 - (distance_from_center / self.heart_size).min(1.0)) * 0.3 + 0.7;

                        let (r, g, b) = self.get_heart_color(final_intensity * center_glow);

                        // Effet de pulsation lumineuse
                        let pulse_glow = 1.0 + (self.beat_phase * 12.56).sin() * total_energy * 0.3;

                        frame[idx] = (r * pulse_glow * 255.0).min(255.0) as u8;
                        frame[idx + 1] = (g * pulse_glow * 255.0).min(255.0) as u8;
                        frame[idx + 2] = (b * pulse_glow * 255.0).min(255.0) as u8;
                    }
                }
            }
        }

        // Ajouter des etincelles autour du coeur lors de battements forts
        if self.beat_intensity > 0.8 && total_energy > 0.5 {
            let sparkle_count = (total_energy * 15.0) as usize;

            for _ in 0..sparkle_count {
                // Position aleatoire autour du coeur
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

        // Effet de flash lors de battements tres intenses
        if self.beat_intensity > 0.9 && total_energy > 0.7 {
            let flash_intensity = (self.beat_intensity - 0.9) * 10.0 * total_energy;
            let (flash_r, flash_g, flash_b) = self.get_heart_color(1.0);

            // Flash global autour du coeur
            for y in (center_y as usize).saturating_sub(50)..((center_y as usize + 50).min(128)) {
                for x in (center_x as usize).saturating_sub(50)..((center_x as usize + 50).min(128)) {
                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
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

    fn set_color_mode(&mut self, mode: &str) {
        println!("💖 [Heartbeat] Color mode set to '{}'", mode);
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "💖 [Heartbeat] Custom color set to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
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
