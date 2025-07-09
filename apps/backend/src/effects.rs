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
                Box::new(SpectrumBars::new()),
                Box::new(CircularWave::new()),
                Box::new(ParticleSystem::new()),
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
