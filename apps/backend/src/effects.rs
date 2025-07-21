use rayon::prelude::*;
use std::f32::consts::PI;

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
                Box::new(SpectrumBars::new()) as Box<dyn Effect>,
                Box::new(CircularWave::new()) as Box<dyn Effect>,
                Box::new(ParticleSystem::new()) as Box<dyn Effect>,
                Box::new(Heartbeat::new()) as Box<dyn Effect>,
                Box::new(Starfall::new()) as Box<dyn Effect>,
                Box::new(Rain::new()) as Box<dyn Effect>,
                Box::new(Flames::new()) as Box<dyn Effect>,
                Box::new(Applaudimetre::new()) as Box<dyn Effect>,
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

        let turbulence_x = (time * 0.1 + self.turbulence_offset).sin() * 0.3;
        let _turbulence_y = (time * 0.08 + self.turbulence_offset * 1.3).cos() * 0.2;

        self.velocity_y -= 0.15 + sound_intensity * 0.1;
        self.velocity_x += turbulence_x + wind_force;

        self.velocity_x *= 0.98;
        self.velocity_y *= 0.995;

        self.x += self.velocity_x;
        self.y += self.velocity_y;

        let cooling_rate = 0.02 + (self.age / self.max_age) * 0.08;
        self.temperature *= 1.0 - cooling_rate;

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
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        let t = temperature.clamp(0.0, 1.0);

        match color_mode.mode.as_str() {
            "fire" => {
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
                let (r, g, b) = color_mode.custom_color;
                let intensity = t.powf(0.7); // Courbe plus naturelle
                (r * intensity, g * intensity, b * intensity)
            }
            _ => {
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

        self.sound_history.remove(0);
        self.sound_history.push(sound_intensity);
        let smoothed_intensity =
            self.sound_history.iter().sum::<f32>() / self.sound_history.len() as f32;

        self.time += 1.0 + sound_intensity * 2.0;

        self.create_flame_base(smoothed_intensity);

        self.add_sparks(sound_intensity);

        let wind_force = (high_energy - 0.1).max(0.0) * 0.3 * (self.time * 0.05).sin();

        for particle in &mut self.particles {
            particle.update(self.time, wind_force, sound_intensity);
        }

        self.particles.retain(|p| p.is_alive());

        let max_particles = 300 + (sound_intensity * 200.0) as usize;
        if self.particles.len() > max_particles {
            self.particles
                .drain(0..self.particles.len() - max_particles);
        }

        for pixel in frame.iter_mut() {
            *pixel = 0;
        }

        let mut temperature_buffer = vec![0.0f32; 128 * 128];

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
}

struct Rain {
    drops: Vec<RainDrop>,
    animation_counter: f32,
    color_mode: String,
    custom_color: (f32, f32, f32),
}

struct RainDrop {
    x: f32,
    y: f32,
    length: f32,
    speed: f32,
    brightness: f32,
}

impl Rain {
    fn new() -> Self {
        let mut drops = Vec::with_capacity(100);

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
            color_mode: "ocean".to_string(),
            custom_color: (0.0, 0.5, 1.0),
        }
    }

    fn get_rain_color(&self, brightness: f32, y_pos: f32) -> (f32, f32, f32) {
        match self.color_mode.as_str() {
            "custom" => {
                let (r, g, b) = self.custom_color;
                (r * brightness, g * brightness, b * brightness)
            }
            "rainbow" => {
                let hue = (y_pos / 128.0) * 360.0;
                let (r, g, b) = hsv_to_rgb(hue, 0.7, brightness);
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

        let sensitivity = 4.0;
        let bass_energy = (bass_energy * sensitivity).min(1.0);
        let mid_energy = (mid_energy * sensitivity).min(1.0);
        let high_energy = (high_energy * sensitivity).min(1.0);

        let total_energy = (bass_energy * 0.5 + mid_energy * 0.3 + high_energy * 0.2).min(1.0);

        for pixel in frame.chunks_exact_mut(3) {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        }

        self.animation_counter += 0.1;

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

        let mut i = 0;
        while i < self.drops.len() {
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

            let drop = &self.drops[i];
            let x = drop.x as usize;
            let start_y = (drop.y - drop.length).max(0.0) as usize;
            let end_y = drop.y.min(127.0) as usize;

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

            if drop.y - drop.length > 128.0 {
                self.drops.swap_remove(i);
            } else {
                i += 1;
            }
        }

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

pub struct Applaudimetre {
    current_level: f32,
    max_level: f32,
    max_hold_time: f32,
    smoothed_level: f32,
    peak_history: Vec<f32>,
    animation_time: f32,
    level_history: Vec<f32>,
    peak_sparkles: Vec<PeakSparkle>,
    sensitivity: f32,
    auto_gain: f32,
    background_pulse: f32,
}

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
            level_history: vec![0.0; 128],
            peak_sparkles: Vec::new(),
            sensitivity: 3.0,
            auto_gain: 1.0,
            background_pulse: 0.0,
        }
    }

    fn get_color_for_level(&self, level: f32, is_max_indicator: bool) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        if is_max_indicator {
            match color_mode.mode.as_str() {
                "rainbow" => {
                    let hue = (self.animation_time * 0.02) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    (r, g, b)
                }
                "fire" => (1.0, 1.0, 0.2),
                "ocean" => (0.2, 1.0, 1.0),
                "sunset" => (1.0, 0.6, 0.1),
                "custom" => {
                    let (r, g, b) = color_mode.custom_color;
                    ((r * 1.8).min(1.0), (g * 1.8).min(1.0), (b * 1.8).min(1.0))
                }
                _ => (1.0, 1.0, 1.0),
            }
        } else {
            match color_mode.mode.as_str() {
                "rainbow" => {
                    if level < 0.25 {
                        let t = level * 4.0;
                        (0.0, t * 0.5, 1.0)
                    } else if level < 0.5 {
                        let t = (level - 0.25) * 4.0;
                        (0.0, 0.5 + t * 0.5, 1.0 - t)
                    } else if level < 0.75 {
                        let t = (level - 0.5) * 4.0;
                        (t, 1.0, 0.0)
                    } else {
                        let t = (level - 0.75) * 4.0;
                        (1.0, 1.0 - t, 0.0)
                    }
                }
                "fire" => {
                    if level < 0.3 {
                        let t = level / 0.3;
                        (t * 0.8, 0.0, 0.0)
                    } else if level < 0.6 {
                        let t = (level - 0.3) / 0.3;
                        (0.8 + t * 0.2, t * 0.4, 0.0)
                    } else {
                        let t = (level - 0.6) / 0.4;
                        (1.0, 0.4 + t * 0.6, t * 0.3)
                    }
                }
                "ocean" => {
                    if level < 0.5 {
                        let t = level * 2.0;
                        (0.0, t * 0.3, 0.4 + t * 0.4)
                    } else {
                        let t = (level - 0.5) * 2.0;
                        (t * 0.2, 0.3 + t * 0.5, 0.8 + t * 0.2)
                    }
                }
                "sunset" => {
                    if level < 0.4 {
                        let t = level / 0.4;
                        (0.3 + t * 0.4, 0.1 * t, 0.5 + t * 0.3)
                    } else {
                        let t = (level - 0.4) / 0.6;
                        (0.7 + t * 0.3, 0.1 + t * 0.7, 0.8 - t * 0.8)
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
        let bass_weight = 0.5;
        let mid_weight = 0.35;
        let high_weight = 0.15;

        let bass_level = spectrum[..8].iter().sum::<f32>() / 8.0;
        let mid_level = spectrum[8..24].iter().sum::<f32>() / 16.0;
        let high_level = spectrum[24..].iter().sum::<f32>() / 40.0;

        let raw_level =
            (bass_level * bass_weight + mid_level * mid_weight + high_level * high_weight)
                * self.sensitivity;

        if raw_level > 0.01 {
            let avg_recent = self.peak_history.iter().sum::<f32>() / self.peak_history.len() as f32;
            if avg_recent < 0.2 {
                self.auto_gain = (self.auto_gain + 0.01).min(2.0);
            } else if avg_recent > 0.8 {
                self.auto_gain = (self.auto_gain - 0.01).max(0.5);
            }
        }

        let final_level = (raw_level * self.auto_gain).powf(0.6);
        final_level.min(1.0)
    }

    fn update_sparkles(&mut self) {
        self.peak_sparkles.retain_mut(|sparkle| {
            sparkle.life -= 0.03;
            sparkle.y -= 0.5;
            sparkle.x += (rand() - 0.5) * 0.3;
            sparkle.life > 0.0
        });

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
        let raw_level = self.calculate_audio_level(spectrum);

        let smoothing = if raw_level > self.smoothed_level {
            0.4
        } else {
            0.85
        };
        self.smoothed_level = self.smoothed_level * smoothing + raw_level * (1.0 - smoothing);
        self.current_level = self.smoothed_level;

        self.level_history.remove(0);
        self.level_history.push(self.current_level);

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
                let decay_rate = 0.008;
                self.max_level = (self.max_level - decay_rate).max(recent_max);
                if self.max_level <= recent_max {
                    self.max_hold_time = 0.0;
                }
            }
        }

        self.animation_time += 1.0 + self.current_level * 2.0;
        self.background_pulse = (self.animation_time * 0.05).sin() * 0.1 + 0.9;
        self.update_sparkles();

        frame.fill(0);

        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = i % 128;
            let y = i / 128;
            let bar_left = 40;
            let bar_right = 88;
            let bar_center = (bar_left + bar_right) / 2;

            if x >= bar_left && x < bar_right {
                let y_pos = (127 - y) as f32 / 127.0;
                let bar_height = self.current_level.min(1.0);

                let history_index = ((x - bar_left) as f32 / (bar_right - bar_left) as f32
                    * self.level_history.len() as f32) as usize;
                if history_index < self.level_history.len() {
                    let historical_level = self.level_history[history_index];
                    if y_pos <= historical_level && historical_level > 0.05 {
                        let trail_intensity =
                            0.2 + (historical_level / self.current_level.max(0.1)) * 0.3;
                        let (r, g, b) = self.get_color_for_level(y_pos, false);

                        pixel[0] = (r * trail_intensity * 255.0) as u8;
                        pixel[1] = (g * trail_intensity * 255.0) as u8;
                        pixel[2] = (b * trail_intensity * 255.0) as u8;
                    }
                }

                if y_pos <= bar_height && bar_height > 0.0 {
                    let level_factor = y_pos / bar_height.max(0.01);

                    let distance_from_center =
                        ((x - bar_center) as f32 / ((bar_right - bar_left) / 2) as f32).abs();
                    let center_glow = (1.0 - distance_from_center).max(0.0);

                    let brightness = 0.7 + level_factor * 0.3 + center_glow * 0.3;
                    let (r, g, b) = self.get_color_for_level(level_factor, false);

                    let pulse = 1.0 + self.current_level * (self.animation_time * 0.1).sin() * 0.2;

                    pixel[0] = (r * brightness * pulse * 255.0).min(255.0) as u8;
                    pixel[1] = (g * brightness * pulse * 255.0).min(255.0) as u8;
                    pixel[2] = (b * brightness * pulse * 255.0).min(255.0) as u8;
                }

                let max_y = (127.0 - self.max_level * 127.0) as usize;
                if y >= max_y.saturating_sub(2)
                    && y <= max_y.saturating_add(2)
                    && self.max_level > 0.05
                {
                    let (r, g, b) = self.get_color_for_level(self.max_level, true);

                    let blink_factor = if self.max_hold_time < 5.0 {
                        let base_blink = 0.8 + 0.2 * (self.max_hold_time * 8.0).sin();
                        let pulse_blink = 1.0 + 0.3 * (self.animation_time * 0.15).sin();
                        base_blink * pulse_blink
                    } else {
                        0.6 + 0.4 * (self.max_hold_time * 2.0).sin().abs()
                    };

                    let width_factor = if (y as i32 - max_y as i32).abs() <= 1 {
                        1.0
                    } else {
                        0.7
                    };

                    pixel[0] = (r * blink_factor * width_factor * 255.0) as u8;
                    pixel[1] = (g * blink_factor * width_factor * 255.0) as u8;
                    pixel[2] = (b * blink_factor * width_factor * 255.0) as u8;
                }

                if x == bar_left + 1 || x == bar_right - 2 {
                    for grad in 0..11 {
                        let grad_y = (127.0 - (grad as f32 * 127.0 / 10.0)) as usize;
                        if y >= grad_y.saturating_sub(1) && y <= grad_y.saturating_add(1) {
                            let intensity = if grad == 10 {
                                0.9
                            } else if grad % 2 == 0 {
                                0.6
                            } else {
                                0.3
                            };
                            let background_effect = intensity * self.background_pulse;

                            pixel[0] = (150.0 * background_effect) as u8;
                            pixel[1] = (150.0 * background_effect) as u8;
                            pixel[2] = (150.0 * background_effect) as u8;
                        }
                    }
                }
            }

            let is_frame = ((x == bar_left - 1 || x == bar_right) && y < 128)
                || ((y == 0 || y == 127) && x >= bar_left - 1 && x <= bar_right);

            if is_frame {
                let glow_intensity = 80.0 + self.current_level * 50.0;
                pixel[0] = glow_intensity as u8;
                pixel[1] = glow_intensity as u8;
                pixel[2] = glow_intensity as u8;
            }
        });

        for sparkle in &self.peak_sparkles {
            let x = sparkle.x as usize;
            let y = sparkle.y as usize;

            if x < 128 && y < 128 {
                let idx = (y * 128 + x) * 3;
                if idx + 2 < frame.len() {
                    let intensity = sparkle.brightness * sparkle.life;

                    frame[idx] =
                        ((sparkle.color.0 * intensity * 255.0) as u8).saturating_add(frame[idx]);
                    frame[idx + 1] = ((sparkle.color.1 * intensity * 255.0) as u8)
                        .saturating_add(frame[idx + 1]);
                    frame[idx + 2] = ((sparkle.color.2 * intensity * 255.0) as u8)
                        .saturating_add(frame[idx + 2]);
                }
            }
        }
    }
}

pub struct Starfall {
    shooting_stars: Vec<ShootingStar>,
    animation_time: f32,
    spawn_timer: f32,
}

struct ShootingStar {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,

    brightness: f32,
    size: f32,
    color: (f32, f32, f32),

    trail_points: Vec<TrailPoint>,
    max_trail_length: usize,

    age: f32,
    max_age: f32,

    twinkle_phase: f32,
    twinkle_speed: f32,
}

#[derive(Clone)]
struct TrailPoint {
    x: f32,
    y: f32,
    intensity: f32,
    age: f32,
}

impl ShootingStar {
    fn new(spawn_side: SpawnSide, sound_intensity: f32) -> Self {
        let (start_x, start_y, vel_x, vel_y) = match spawn_side {
            SpawnSide::TopLeft => {
                let x = -30.0 + rand() * 20.0;
                let y = -30.0 + rand() * 80.0;
                let vx = 2.0 + rand() * 3.0 + sound_intensity * 3.0;
                let vy = 1.5 + rand() * 2.5 + sound_intensity * 2.0;
                (x, y, vx, vy)
            }
            SpawnSide::TopRight => {
                let x = 138.0 + rand() * 20.0;
                let y = -30.0 + rand() * 80.0;
                let vx = -2.0 - rand() * 3.0 - sound_intensity * 3.0;
                let vy = 1.5 + rand() * 2.5 + sound_intensity * 2.0;
                (x, y, vx, vy)
            }
            SpawnSide::Top => {
                let x = 20.0 + rand() * 88.0;
                let y = -40.0 + rand() * 30.0;
                let vx = (rand() - 0.5) * 4.0;
                let vy = 3.0 + rand() * 3.0 + sound_intensity * 3.0;
                (x, y, vx, vy)
            }
        };

        let star_temp = rand();
        let base_color = if star_temp < 0.1 {
            (0.7, 0.8, 1.0)
        } else if star_temp < 0.3 {
            (1.0, 1.0, 1.0)
        } else if star_temp < 0.6 {
            (1.0, 1.0, 0.8)
        } else if star_temp < 0.8 {
            (1.0, 0.8, 0.6)
        } else {
            (1.0, 0.6, 0.4)
        };

        Self {
            x: start_x,
            y: start_y,
            velocity_x: vel_x,
            velocity_y: vel_y,
            brightness: 0.6 + rand() * 0.4 + sound_intensity * 0.3,
            size: 1.0 + rand() * 2.0 + sound_intensity * 1.5,
            color: base_color,
            trail_points: Vec::new(),
            max_trail_length: (15 + (sound_intensity * 25.0) as usize).min(40),
            age: 0.0,
            max_age: 120.0 + rand() * 60.0,
            twinkle_phase: rand() * 2.0 * PI,
            twinkle_speed: 0.1 + rand() * 0.2,
        }
    }

    fn update(&mut self, time: f32) {
        self.age += 1.0;
        self.twinkle_phase += self.twinkle_speed;

        self.trail_points.push(TrailPoint {
            x: self.x,
            y: self.y,
            intensity: self.brightness,
            age: 0.0,
        });

        if self.trail_points.len() > self.max_trail_length {
            self.trail_points.remove(0);
        }

        for point in &mut self.trail_points {
            point.age += 1.0;
            point.intensity *= 0.95;
        }

        self.x += self.velocity_x;
        self.y += self.velocity_y;

        self.velocity_y += 0.05;

        self.velocity_x += (time * 0.1 + self.twinkle_phase).sin() * 0.1;
        self.velocity_y += (time * 0.08 + self.twinkle_phase * 1.3).cos() * 0.05;

        if self.age > self.max_age * 0.7 {
            let fade_factor = 1.0 - (self.age - self.max_age * 0.7) / (self.max_age * 0.3);
            self.brightness *= fade_factor.max(0.0);
        }
    }

    fn is_alive(&self) -> bool {
        self.age < self.max_age
            && self.brightness > 0.01
            && self.x > -50.0
            && self.x < 178.0
            && self.y > -50.0
            && self.y < 178.0
    }

    fn get_twinkle_factor(&self) -> f32 {
        0.7 + 0.3 * (self.twinkle_phase.sin() * 0.5 + 0.5)
    }
}

#[derive(Clone)]
enum SpawnSide {
    TopLeft,
    TopRight,
    Top,
}

impl Starfall {
    pub fn new() -> Self {
        Self {
            shooting_stars: Vec::new(),
            animation_time: 0.0,
            spawn_timer: 0.0,
        }
    }

    fn get_star_color(&self, base_color: (f32, f32, f32), brightness: f32) -> (f32, f32, f32) {
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        match color_mode.mode.as_str() {
            "rainbow" => {
                let hue = (self.animation_time * 0.005 + brightness) % 1.0;
                hsv_to_rgb(hue, 0.7, brightness)
            }
            "fire" => {
                // Étoiles filantes de feu
                if brightness < 0.3 {
                    (brightness * 3.0, brightness * 0.5, 0.0)
                } else if brightness < 0.7 {
                    (1.0, (brightness - 0.3) * 2.5, 0.0)
                } else {
                    (1.0, 1.0, (brightness - 0.7) * 3.0)
                }
            }
            "ocean" => (brightness * 0.3, brightness * 0.8, brightness),
            "sunset" => {
                let warm_factor = brightness.powf(0.8);
                (warm_factor, warm_factor * 0.7, warm_factor * 0.9)
            }
            "custom" => {
                let (r, g, b) = color_mode.custom_color;
                (r * brightness, g * brightness, b * brightness)
            }
            _ => (
                base_color.0 * brightness,
                base_color.1 * brightness,
                base_color.2 * brightness,
            ),
        }
    }

    fn spawn_shooting_star(&mut self, sound_intensity: f32) {
        let spawn_side = match (rand() * 3.0) as usize {
            0 => SpawnSide::TopLeft,
            1 => SpawnSide::TopRight,
            _ => SpawnSide::Top,
        };

        let star = ShootingStar::new(spawn_side, sound_intensity);
        self.shooting_stars.push(star);
    }

    fn create_meteor_shower(&mut self, intensity: f32) {
        let meteor_count = (intensity * 8.0) as usize;
        for _ in 0..meteor_count {
            let spawn_side = SpawnSide::TopLeft;
            let mut star = ShootingStar::new(spawn_side, intensity);

            star.velocity_x *= 1.5;
            star.velocity_y *= 1.5;
            star.brightness *= 1.3;
            star.max_trail_length *= 2;

            self.shooting_stars.push(star);
        }
    }
}

impl Effect for Starfall {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        let bass_energy = (spectrum[..8].iter().sum::<f32>() / 8.0) * 4.0;
        let mid_energy = (spectrum[8..24].iter().sum::<f32>() / 16.0) * 3.0;
        let high_energy = (spectrum[24..].iter().sum::<f32>() / 40.0) * 2.0;

        let total_energy =
            (bass_energy * 0.3 + mid_energy * 0.4 + high_energy * 0.3).clamp(0.0, 1.0);

        self.animation_time += 1.0;
        self.spawn_timer += 1.0;

        let base_spawn_interval = 45.0;
        let min_spawn_interval = 8.0;

        let spawn_interval =
            base_spawn_interval - (total_energy * (base_spawn_interval - min_spawn_interval));

        if self.spawn_timer >= spawn_interval {
            self.spawn_timer = 0.0;

            let star_count = if total_energy > 0.7 {
                2 + (total_energy * 3.0) as usize
            } else if total_energy > 0.3 {
                1 + (total_energy * 2.0) as usize
            } else {
                1
            };

            for _ in 0..star_count {
                if self.shooting_stars.len() < 25 {
                    self.spawn_shooting_star(total_energy);
                }
            }
        }

        if total_energy > 0.85 && rand() < 0.1 {
            self.create_meteor_shower(total_energy);
        }

        for star in &mut self.shooting_stars {
            star.update(self.animation_time);
        }

        self.shooting_stars.retain(|star| star.is_alive());

        frame.fill(0);

        for star in &self.shooting_stars {
            let twinkle = star.get_twinkle_factor();

            for (i, trail_point) in star.trail_points.iter().enumerate() {
                if trail_point.intensity > 0.05 {
                    let trail_age_factor = 1.0 - (i as f32 / star.trail_points.len() as f32);
                    let trail_brightness = trail_point.intensity * trail_age_factor * 0.6;

                    let px = trail_point.x as i32;
                    let py = trail_point.y as i32;

                    if px >= 0 && px < 128 && py >= 0 && py < 128 {
                        let idx = (py as usize * 128 + px as usize) * 3;
                        if idx + 2 < frame.len() {
                            let (r, g, b) = self.get_star_color(star.color, trail_brightness);

                            frame[idx] = ((r * 255.0) as u8).saturating_add(frame[idx]);
                            frame[idx + 1] = ((g * 255.0) as u8).saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((b * 255.0) as u8).saturating_add(frame[idx + 2]);
                        }
                    }
                }
            }

            let star_brightness = star.brightness * twinkle;
            let star_size = star.size;

            let center_x = star.x as i32;
            let center_y = star.y as i32;

            for dy in -(star_size as i32 + 1)..=(star_size as i32 + 1) {
                for dx in -(star_size as i32 + 1)..=(star_size as i32 + 1) {
                    let x = center_x + dx;
                    let y = center_y + dy;

                    if x >= 0 && x < 128 && y >= 0 && y < 128 {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();

                        let mut intensity = 0.0;

                        if distance <= star_size {
                            let core_falloff = (1.0 - distance / star_size).max(0.0);
                            intensity = star_brightness * core_falloff;
                        } else if distance <= star_size + 1.0 {
                            let halo_falloff = (1.0 - (distance - star_size)).max(0.0);
                            intensity = star_brightness * halo_falloff * 0.3;
                        }

                        if intensity > 0.01 {
                            let idx = (y as usize * 128 + x as usize) * 3;
                            if idx + 2 < frame.len() {
                                let (r, g, b) = self.get_star_color(star.color, intensity);

                                frame[idx] = ((r * 255.0) as u8).saturating_add(frame[idx]);
                                frame[idx + 1] = ((g * 255.0) as u8).saturating_add(frame[idx + 1]);
                                frame[idx + 2] = ((b * 255.0) as u8).saturating_add(frame[idx + 2]);
                            }
                        }
                    }
                }
            }

            if star_brightness > 0.7 {
                let ray_length = (star_size * 1.5) as i32;
                let ray_brightness = star_brightness * 0.4;

                for offset in -ray_length..=ray_length {
                    let hx = center_x + offset;
                    let hy = center_y;
                    if hx >= 0 && hx < 128 && hy >= 0 && hy < 128 {
                        let idx = (hy as usize * 128 + hx as usize) * 3;
                        if idx + 2 < frame.len() {
                            let ray_intensity =
                                ray_brightness * (1.0 - offset.abs() as f32 / ray_length as f32);
                            let (r, g, b) = self.get_star_color(star.color, ray_intensity);

                            frame[idx] = ((r * 128.0) as u8).saturating_add(frame[idx]);
                            frame[idx + 1] = ((g * 128.0) as u8).saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((b * 128.0) as u8).saturating_add(frame[idx + 2]);
                        }
                    }

                    let vx = center_x;
                    let vy = center_y + offset;
                    if vx >= 0 && vx < 128 && vy >= 0 && vy < 128 {
                        let idx = (vy as usize * 128 + vx as usize) * 3;
                        if idx + 2 < frame.len() {
                            let ray_intensity =
                                ray_brightness * (1.0 - offset.abs() as f32 / ray_length as f32);
                            let (r, g, b) = self.get_star_color(star.color, ray_intensity);

                            frame[idx] = ((r * 128.0) as u8).saturating_add(frame[idx]);
                            frame[idx + 1] = ((g * 128.0) as u8).saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((b * 128.0) as u8).saturating_add(frame[idx + 2]);
                        }
                    }
                }
            }
        }
    }
}

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
        let color_mode = unsafe { &GLOBAL_COLOR_CONFIG };

        match color_mode.mode.as_str() {
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
                let (r, g, b) = color_mode.custom_color;
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

        self.beat_frequency = 40.0 + total_energy * 100.0;
        let beat_interval = 60.0 / self.beat_frequency;

        self.animation_time += 1.0 / 60.0;

        if self.animation_time - self.last_beat_time >= beat_interval {
            self.last_beat_time = self.animation_time;
            self.beat_phase = 0.0;

            self.pulse_rings.push(PulseRing {
                radius: 0.0,
                life: 1.0,
                intensity: 0.3 + total_energy * 0.7,
                color: self.get_heart_color(1.0),
            });
        }

        self.beat_phase += (self.beat_frequency / 60.0) * 0.15;

        let double_beat = if self.beat_phase % 1.0 < 0.3 {
            ((self.beat_phase % 1.0) * 10.0).sin().max(0.0)
        } else if self.beat_phase % 1.0 < 0.5 {
            (((self.beat_phase % 1.0) - 0.3) * 15.0).sin().max(0.0) * 0.6
        } else {
            0.2
        };

        self.beat_intensity = 0.4 + double_beat * (0.3 + total_energy * 0.3);

        let base_size = 15.0 + total_energy * 25.0;
        self.heart_size = base_size * (0.8 + self.beat_intensity * 0.4);

        self.pulse_rings.retain_mut(|ring| {
            ring.radius += 2.0 + total_energy * 3.0;
            ring.life -= 0.02;
            ring.intensity *= 0.98;
            ring.life > 0.0 && ring.radius < 100.0
        });

        frame.fill(0);

        let center_x = 64.0;
        let center_y = 64.0;

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

        for y in 0..128 {
            for x in 0..128 {
                let heart_intensity =
                    self.is_inside_heart(x as f32, y as f32, center_x, center_y, self.heart_size);

                if heart_intensity > 0.0 {
                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        let final_intensity = heart_intensity * self.beat_intensity;

                        let distance_from_center =
                            ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                        let center_glow =
                            (1.0 - (distance_from_center / self.heart_size).min(1.0)) * 0.3 + 0.7;

                        let (r, g, b) = self.get_heart_color(final_intensity * center_glow);

                        let pulse_glow = 1.0 + (self.beat_phase * 12.56).sin() * total_energy * 0.3;

                        frame[idx] = (r * pulse_glow * 255.0).min(255.0) as u8;
                        frame[idx + 1] = (g * pulse_glow * 255.0).min(255.0) as u8;
                        frame[idx + 2] = (b * pulse_glow * 255.0).min(255.0) as u8;
                    }
                }
            }
        }

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

        if self.beat_intensity > 0.9 && total_energy > 0.7 {
            let flash_intensity = (self.beat_intensity - 0.9) * 10.0 * total_energy;
            let (flash_r, flash_g, flash_b) = self.get_heart_color(1.0);

            for y in (center_y as usize).saturating_sub(50)..((center_y as usize + 50).min(128)) {
                for x in (center_x as usize).saturating_sub(50)..((center_x as usize + 50).min(128))
                {
                    let idx = (y * 128 + x) * 3;
                    if idx + 2 < frame.len() {
                        let distance =
                            ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                        let flash_falloff = (1.0 - distance / 50.0).max(0.0);
                        let flash_add = flash_intensity * flash_falloff * 50.0;

                        frame[idx] = ((flash_r * flash_add) as u8).saturating_add(frame[idx]);
                        frame[idx + 1] =
                            ((flash_g * flash_add) as u8).saturating_add(frame[idx + 1]);
                        frame[idx + 2] =
                            ((flash_b * flash_add) as u8).saturating_add(frame[idx + 2]);
                    }
                }
            }
        }
    }
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
