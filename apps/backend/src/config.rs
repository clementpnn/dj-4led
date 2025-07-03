use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub audio: AudioConfig,
    pub led: LedConfig,
    pub effects: EffectsConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub channels: u16,
    pub device_name: Option<String>,
    pub gain: f32,
    pub noise_floor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedConfig {
    pub controllers: Vec<String>,
    pub fps: u32,
    pub brightness: f32,
    pub gamma_correction: f32,
    pub color_temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsConfig {
    pub smoothing_factor: f32,
    pub bass_boost: f32,
    pub mid_boost: f32,
    pub high_boost: f32,
    pub particle_limit: usize,
    pub wave_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub thread_pool_size: usize,
    pub frame_skip: bool,
    pub adaptive_quality: bool,
    pub max_cpu_percent: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig {
                sample_rate: 48000,
                buffer_size: 64,
                channels: 1,
                device_name: None,
                gain: 1.0,
                noise_floor: 0.01,
            },
            led: LedConfig {
                controllers: vec![
                    "192.168.1.45:6454".to_string(),
                    "192.168.1.46:6454".to_string(),
                    "192.168.1.47:6454".to_string(),
                    "192.168.1.48:6454".to_string(),
                ],
                fps: 60,
                brightness: 1.0,
                gamma_correction: 2.2,
                color_temperature: 1.0,
            },
            effects: EffectsConfig {
                smoothing_factor: 0.7,
                bass_boost: 1.5,
                mid_boost: 1.2,
                high_boost: 1.0,
                particle_limit: 2000,
                wave_speed: 1.0,
            },
            performance: PerformanceConfig {
                thread_pool_size: 4,
                frame_skip: false,
                adaptive_quality: true,
                max_cpu_percent: 80.0,
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = "config.toml";

        if Path::new(config_path).exists() {
            match fs::read_to_string(config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => {
                        println!("✅ Configuration loaded from {}", config_path);
                        return config;
                    }
                    Err(e) => {
                        eprintln!("❌ Error parsing config file: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("❌ Error reading config file: {}", e);
                }
            }
        }

        println!("📝 Using default configuration");
        let default_config = Self::default();

        // Save default config for reference
        if let Err(e) = default_config.save() {
            eprintln!("⚠️  Could not save default config: {}", e);
        }

        default_config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let toml = toml::to_string_pretty(self)?;
        fs::write("config.toml", toml)?;
        println!("💾 Configuration saved to config.toml");
        Ok(())
    }

    pub fn production() -> Self {
        Self {
            audio: AudioConfig {
                sample_rate: 48000,
                buffer_size: 128, // Slightly larger for stability
                channels: 1,
                device_name: None,
                gain: 1.2,
                noise_floor: 0.02,
            },
            led: LedConfig {
                controllers: vec![
                    "192.168.1.45:6454".to_string(),
                    "192.168.1.46:6454".to_string(),
                    "192.168.1.47:6454".to_string(),
                    "192.168.1.48:6454".to_string(),
                ],
                fps: 50, // Lower FPS for stability
                brightness: 0.9,
                gamma_correction: 2.2,
                color_temperature: 1.0,
            },
            effects: EffectsConfig {
                smoothing_factor: 0.6,
                bass_boost: 1.8,
                mid_boost: 1.3,
                high_boost: 1.1,
                particle_limit: 1500, // Less particles for performance
                wave_speed: 1.2,
            },
            performance: PerformanceConfig {
                thread_pool_size: 6,
                frame_skip: true,
                adaptive_quality: true,
                max_cpu_percent: 70.0,
            },
        }
    }

    pub fn high_performance() -> Self {
        Self {
            audio: AudioConfig {
                sample_rate: 44100,
                buffer_size: 256, // Larger buffer for less CPU usage
                channels: 1,
                device_name: None,
                gain: 1.0,
                noise_floor: 0.03,
            },
            led: LedConfig {
                controllers: vec![
                    "192.168.1.45:6454".to_string(),
                    "192.168.1.46:6454".to_string(),
                    "192.168.1.47:6454".to_string(),
                    "192.168.1.48:6454".to_string(),
                ],
                fps: 30, // Lower FPS for performance
                brightness: 0.8,
                gamma_correction: 2.0,
                color_temperature: 1.0,
            },
            effects: EffectsConfig {
                smoothing_factor: 0.5,
                bass_boost: 1.5,
                mid_boost: 1.1,
                high_boost: 1.0,
                particle_limit: 1000, // Less particles
                wave_speed: 1.0,
            },
            performance: PerformanceConfig {
                thread_pool_size: 4,
                frame_skip: true,
                adaptive_quality: true,
                max_cpu_percent: 60.0,
            },
        }
    }
}

// Helper functions for runtime adjustments
impl Config {
    pub fn apply_brightness(&self, color: &mut [u8; 3]) {
        color[0] = (color[0] as f32 * self.led.brightness) as u8;
        color[1] = (color[1] as f32 * self.led.brightness) as u8;
        color[2] = (color[2] as f32 * self.led.brightness) as u8;
    }

    pub fn apply_gamma_correction(&self, color: &mut [u8; 3]) {
        let gamma = 1.0 / self.led.gamma_correction;
        color[0] = ((color[0] as f32 / 255.0).powf(gamma) * 255.0) as u8;
        color[1] = ((color[1] as f32 / 255.0).powf(gamma) * 255.0) as u8;
        color[2] = ((color[2] as f32 / 255.0).powf(gamma) * 255.0) as u8;
    }

    pub fn apply_color_temperature(&self, color: &mut [u8; 3]) {
        if self.led.color_temperature != 1.0 {
            // Warm (< 1.0) or Cool (> 1.0) adjustment
            if self.led.color_temperature < 1.0 {
                // Warmer - reduce blue
                color[2] = (color[2] as f32 * self.led.color_temperature) as u8;
            } else {
                // Cooler - reduce red
                color[0] = (color[0] as f32 / self.led.color_temperature) as u8;
            }
        }
    }

    pub fn get_frame_delay_ms(&self) -> u64 {
        1000 / self.led.fps as u64
    }

    pub fn should_skip_frame(&self, frame_time_ms: f32) -> bool {
        self.performance.frame_skip && frame_time_ms > self.get_frame_delay_ms() as f32 * 1.5
    }
}
