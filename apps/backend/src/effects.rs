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
        }

        frame
    }

    pub fn set_effect(&mut self, index: usize) {
        if index < self.effects.len() {
            self.current = index;
        } else {
        }
    }

    pub fn set_color_mode(&mut self, mode: &str) {
        self.color_config.mode = mode.to_string();

        unsafe {
            GLOBAL_COLOR_CONFIG.mode = mode.to_string();
        }

        for (i, effect) in self.effects.iter_mut().enumerate() {
            effect.set_color_mode(mode);
        }
    }

    pub fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        self.color_config.custom_color = (r, g, b);

        unsafe {
            GLOBAL_COLOR_CONFIG.custom_color = (r, g, b);
        }

        for (i, effect) in self.effects.iter_mut().enumerate() {
            effect.set_custom_color(r, g, b);
        }
    }
}

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
                let hue = (bar as f32 / 64.0) * 60.0;
                let saturation = 1.0;
                hsv_to_rgb(hue / 360.0, saturation, brightness)
            }
            "ocean" => {
                let hue = 180.0 + (bar as f32 / 64.0) * 60.0;
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
                    300.0 + (bar as f32 / 32.0) * 60.0
                } else {
                    (bar as f32 - 32.0) / 32.0 * 60.0
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

        static mut DEBUG_COUNTER: u32 = 0;
        unsafe {
            DEBUG_COUNTER += 1;
            if DEBUG_COUNTER % 50 == 0 {
                let max_level = self.smoothed.iter().cloned().fold(0.0f32, f32::max);
            }
        }

        frame.fill(0);

        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i % 128) as f32;
            let y = (i / 128) as f32;

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

                if y >= bar_bottom && y < 128.0 {
                    let brightness = gradient_factor;
                    let (r, g, b) = self.get_color_for_bar(bar, brightness);
                    pixel[0] = (r * 255.0) as u8;
                    pixel[1] = (g * 255.0) as u8;
                    pixel[2] = (b * 255.0) as u8;
                }

                let peak_y = 128.0 - peak_height;
                if (y - peak_y).abs() < 1.0 && peak_height > 5.0 {
                    let (r, g, b) = self.get_color_for_bar(bar, 0.8);
                    pixel[0] = (r * 255.0 * 0.8) as u8;
                    pixel[1] = (g * 255.0 * 0.8) as u8;
                    pixel[2] = (b * 255.0 * 0.8) as u8;
                }

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

                if (x - 64.0).abs() < 0.5 && y >= bar_bottom && y < 128.0 {
                    let (r, g, b) = self.get_color_for_bar(bar, 0.3);
                    pixel[0] = (r * 255.0) as u8;
                    pixel[1] = (g * 255.0) as u8;
                    pixel[2] = (b * 255.0) as u8;
                }
            }
        });
    }

    fn set_color_mode(&mut self, mode: &str) {}

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {}
}

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

            let wave1 = ((dist * 20.0 - self.time * 8.0 * speed_mod).sin() + 1.0) / 2.0;
            let wave2 = ((dist * 10.0 - self.time * 4.0 * speed_mod).cos() + 1.0) / 2.0;
            let wave3 = ((dist * 5.0 - self.time * 2.0 * speed_mod).sin() + 1.0) / 2.0;

            let base_intensity = 0.3;
            let audio_intensity =
                wave1 * bass_energy * 2.0 + wave2 * mid_energy * 1.5 + wave3 * high_energy;

            let intensity = (base_intensity + audio_intensity).min(1.0);

            let wave_pattern = (wave1 * 0.4 + wave2 * 0.3 + wave3 * 0.3).min(1.0);
            let brightness = (base_intensity + intensity * wave_pattern).min(1.0);

            let (r, g, b) =
                self.get_color_for_wave(angle, dist, brightness, bass_energy, mid_energy);

            pixel[0] = (r * 255.0) as u8;
            pixel[1] = (g * 255.0) as u8;
            pixel[2] = (b * 255.0) as u8;
        });
    }

    fn set_color_mode(&mut self, mode: &str) {}

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {}
}

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
                    0.3 + rand() * 0.3
                } else {
                    0.7 + rand() * 0.3
                };
                hsv_to_rgb(hue, 1.0, 1.0)
            }
            "fire" => {
                let hue = rand() * 0.15;
                let saturation = 0.8 + rand() * 0.2;
                let brightness = 0.7 + rand() * 0.3;
                hsv_to_rgb(hue, saturation, brightness)
            }
            "ocean" => {
                let hue = 0.5 + rand() * 0.17;
                let saturation = 0.6 + rand() * 0.4;
                let brightness = 0.6 + rand() * 0.4;
                hsv_to_rgb(hue, saturation, brightness)
            }
            "sunset" => {
                let hue = if rand() > 0.5 {
                    0.833 + rand() * 0.167
                } else {
                    rand() * 0.167
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
        let bass_energy = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_energy = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_energy = spectrum[24..].iter().sum::<f32>() / 40.0;
        let total_energy = (bass_energy + mid_energy + high_energy) / 3.0;

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
                (rand() * 128.0, 100.0 + rand() * 28.0)
            } else if i % 3 == 0 && bass_energy > 0.1 {
                (rand() * 128.0, 120.0 + rand() * 8.0)
            } else if i % 3 == 1 && mid_energy > 0.1 {
                if rand() > 0.5 {
                    (0.0 + rand() * 8.0, 64.0 + (rand() - 0.5) * 64.0)
                } else {
                    (120.0 + rand() * 8.0, 64.0 + (rand() - 0.5) * 64.0)
                }
            } else {
                (rand() * 128.0, rand() * 128.0)
            };

            let (vx, vy) = if i < base_particles {
                ((rand() - 0.5) * 5.0, -rand() * 8.0 - 2.0)
            } else if i % 3 == 0 {
                (
                    (rand() - 0.5) * bass_energy * 10.0,
                    -bass_energy * 15.0 - rand() * 5.0,
                )
            } else if i % 3 == 1 {
                (
                    (rand() - 0.5) * mid_energy * 15.0,
                    (rand() - 0.5) * mid_energy * 10.0,
                )
            } else {
                (
                    (rand() - 0.5) * high_energy * 20.0,
                    (rand() - 0.5) * high_energy * 20.0,
                )
            };

            let color =
                self.get_particle_color(i, base_particles, bass_energy, mid_energy, high_energy);

            self.particles.push(Particle {
                x: spawn_x,
                y: spawn_y,
                vx,
                vy,
                life: 0.5 + total_energy * 0.5,
                color,
            });
        }

        self.particles.retain_mut(|p| {
            p.x += p.vx;
            p.y += p.vy;

            p.vy += 0.3 - total_energy * 0.2;

            let friction = 0.97 - total_energy * 0.02;
            p.vx *= friction;
            p.vy *= friction;

            p.life -= 0.02 - total_energy * 0.01;

            p.life > 0.0 && p.x >= -5.0 && p.x < 133.0 && p.y >= -5.0 && p.y < 133.0
        });

        frame.fill(0);

        for particle in &self.particles {
            let x = particle.x as i32;
            let y = particle.y as i32;

            let size = if particle.life > 0.7 { 2 } else { 1 };

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

    fn set_color_mode(&mut self, mode: &str) {}

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {}
}

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

// Effet 4: Applaudimètre
pub struct Applaudimetre {
    current_level: f32,
    max_level: f32,
    max_hold_time: f32,
    smoothed_level: f32,
    peak_history: Vec<f32>,
}

impl Applaudimetre {
    pub fn new() -> Self {
        Self {
            current_level: 0.0,
            max_level: 0.0,
            max_hold_time: 0.0,
            smoothed_level: 0.0,
            peak_history: vec![0.0; 30], // Historique pour moyenner les pics
        }
    }

    fn get_color_for_level(&self, level: f32, is_max_indicator: bool) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        if is_max_indicator {
            // Couleur spéciale pour l'indicateur de maximum
            match color_mode.mode.as_str() {
                "rainbow" => (1.0, 1.0, 1.0), // Blanc pour se démarquer
                "fire" => (1.0, 1.0, 0.0),    // Jaune brillant
                "ocean" => (0.0, 1.0, 1.0),   // Cyan brillant
                "sunset" => (1.0, 0.5, 0.0),  // Orange brillant
                "custom" => {
                    let (r, g, b) = color_mode.custom_color;
                    (r * 1.5, g * 1.5, b * 1.5) // Version plus brillante
                }
                _ => (1.0, 1.0, 1.0),
            }
        } else {
            // Couleur dégradée selon le niveau
            match color_mode.mode.as_str() {
                "rainbow" => {
                    // Vert -> Jaune -> Rouge selon le niveau
                    if level < 0.5 {
                        let factor = level * 2.0;
                        (factor, 1.0, 0.0) // Vert vers jaune
                    } else {
                        let factor = (level - 0.5) * 2.0;
                        (1.0, 1.0 - factor, 0.0) // Jaune vers rouge
                    }
                }
                "fire" => {
                    let hue = (1.0 - level) * 0.15; // Rouge à jaune selon le niveau
                    hsv_to_rgb(hue, 1.0, level.max(0.3))
                }
                "ocean" => {
                    let hue = 0.5 + level * 0.17; // Cyan vers bleu selon le niveau
                    hsv_to_rgb(hue, 0.8, level.max(0.3))
                }
                "sunset" => {
                    let hue = 0.1 - level * 0.1; // Orange vers rouge selon le niveau
                    hsv_to_rgb(hue, 1.0, level.max(0.3))
                }
                "custom" => {
                    let (r, g, b) = color_mode.custom_color;
                    (r * level.max(0.2), g * level.max(0.2), b * level.max(0.2))
                }
                _ => hsv_to_rgb(0.3, 1.0, level.max(0.3)),
            }
        }
    }

    fn calculate_audio_level(&self, spectrum: &[f32]) -> f32 {
        // Calculer le niveau sonore global avec pondération
        let bass_weight = 0.4;
        let mid_weight = 0.4;
        let high_weight = 0.2;

        let bass_level = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_level = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_level = spectrum[24..].iter().sum::<f32>() / 40.0;

        let weighted_level = bass_level * bass_weight + mid_level * mid_weight + high_level * high_weight;

        // Appliquer une courbe pour rendre plus sensible aux variations
        weighted_level.powf(0.7)
    }
}

impl Effect for Applaudimetre {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Calculer le niveau audio actuel
        let raw_level = self.calculate_audio_level(spectrum);

        // Lissage du niveau pour éviter les fluctuations trop rapides
        self.smoothed_level = self.smoothed_level * 0.7 + raw_level * 0.3;
        self.current_level = self.smoothed_level;

        // Mettre à jour l'historique des pics
        self.peak_history.remove(0);
        self.peak_history.push(self.current_level);

        // Calculer le niveau maximum récent
        let recent_max = self.peak_history.iter().cloned().fold(0.0f32, f32::max);

        // Gestion du maximum avec délai de 5 secondes
        if self.current_level > self.max_level {
            // Nouveau maximum atteint
            self.max_level = self.current_level;
            self.max_hold_time = 0.0;
        } else {
            // Incrémenter le temps de maintien (approximation à ~60 FPS)
            self.max_hold_time += 1.0 / 60.0;

            // Après 5 secondes, permettre au maximum de redescendre
            if self.max_hold_time >= 5.0 {
                // Décroissance lente du maximum vers le niveau récent
                let decay_rate = 0.005;
                self.max_level = (self.max_level - decay_rate).max(recent_max);

                // Réinitialiser le timer si on atteint le niveau récent
                if self.max_level <= recent_max {
                    self.max_hold_time = 0.0;
                }
            }
        }

        // Log de débogage
        static mut APPLAUD_FRAME_COUNT: u64 = 0;
        unsafe {
            APPLAUD_FRAME_COUNT += 1;
            if APPLAUD_FRAME_COUNT % 30 == 0 && self.current_level > 0.01 {
                println!(
                    "👏 [Applaudimètre] Level: {:.3}, Max: {:.3}, Hold: {:.1}s",
                    self.current_level, self.max_level, self.max_hold_time
                );
            }
        }

        // Effacer l'écran
        frame.fill(0);

        // Rendu parallèle de la barre
        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = i % 128;
            let y = i / 128;

            // Définir la zone de la barre (centrée, largeur de 40 pixels)
            let bar_left = 44;
            let bar_right = 84;
            let bar_width = bar_right - bar_left;

            if x >= bar_left && x < bar_right {
                // Position verticale (0.0 = bas, 1.0 = haut)
                let y_pos = (127 - y) as f32 / 127.0;

                // Hauteur de la barre actuelle
                let bar_height = self.current_level.min(1.0);

                // Dessiner la barre principale
                if y_pos <= bar_height {
                    let level_factor = y_pos / bar_height;
                    let brightness = 0.8 + level_factor * 0.2;
                    let (r, g, b) = self.get_color_for_level(level_factor, false);

                    pixel[0] = (r * brightness * 255.0) as u8;
                    pixel[1] = (g * brightness * 255.0) as u8;
                    pixel[2] = (b * brightness * 255.0) as u8;
                }

                // Dessiner l'indicateur de maximum
                let max_y = (127.0 - self.max_level * 127.0) as usize;
                if y >= max_y.saturating_sub(1) && y <= max_y.saturating_add(1) && self.max_level > 0.05 {
                    let (r, g, b) = self.get_color_for_level(self.max_level, true);

                    // Effet de clignotement si le maximum est maintenu
                    let blink_factor = if self.max_hold_time < 5.0 {
                        // Clignotement rapide pendant le maintien
                        0.7 + 0.3 * (self.max_hold_time * 10.0).sin().abs()
                    } else {
                        // Clignotement lent pendant la décroissance
                        0.5 + 0.5 * (self.max_hold_time * 3.0).sin().abs()
                    };

                    pixel[0] = (r * blink_factor * 255.0) as u8;
                    pixel[1] = (g * blink_factor * 255.0) as u8;
                    pixel[2] = (b * blink_factor * 255.0) as u8;
                }

                // Dessiner les graduations sur les côtés
                if x == bar_left || x == bar_right - 1 {
                    // Graduations tous les 12.7 pixels (10 graduations)
                    for grad in 0..10 {
                        let grad_y = (127.0 - (grad as f32 * 127.0 / 9.0)) as usize;
                        if y >= grad_y.saturating_sub(1) && y <= grad_y.saturating_add(1) {
                            let intensity = if grad == 9 { 0.8 } else { 0.4 }; // Graduation du haut plus visible
                            pixel[0] = (128.0 * intensity) as u8;
                            pixel[1] = (128.0 * intensity) as u8;
                            pixel[2] = (128.0 * intensity) as u8;
                        }
                    }
                }
            }

            // Dessiner le cadre autour de la barre
            if ((x == bar_left - 1 || x == bar_right) && y >= 0 && y < 128) ||
               ((y == 0 || y == 127) && x >= bar_left - 1 && x <= bar_right) {
                pixel[0] = 64;
                pixel[1] = 64;
                pixel[2] = 64;
            }

            // Afficher le texte "MAX" près de l'indicateur de maximum
            if self.max_level > 0.1 {
                let max_y = (127.0 - self.max_level * 127.0) as usize;
                let text_x = bar_right + 5;

                // Dessiner "MAX" de façon simple (pattern 3x5)
                if x >= text_x && x < text_x + 15 && y >= max_y.saturating_sub(2) && y <= max_y.saturating_add(2) {
                    let local_x = x - text_x;
                    let local_y = y - max_y.saturating_sub(2);

                    // Pattern simple pour "MAX"
                    let max_pattern = [
                        [1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1],
                        [1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0],
                        [1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1],
                        [1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0],
                        [1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1],
                    ];

                    if local_y < 5 && local_x < 15 && max_pattern[local_y][local_x] == 1 {
                        let (r, g, b) = self.get_color_for_level(self.max_level, true);
                        pixel[0] = (r * 0.8 * 255.0) as u8;
                        pixel[1] = (g * 0.8 * 255.0) as u8;
                        pixel[2] = (b * 0.8 * 255.0) as u8;
                    }
                }
            }
        });
    }

    fn set_color_mode(&mut self, mode: &str) {
        println!("   Applaudimètre: color mode set to '{}'", mode);
        // Color mode is now set globally
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!(
            "   Applaudimètre: custom color set to ({:.2}, {:.2}, {:.2})",
            r, g, b
        );
        // Custom color is now set globally
    }
}
