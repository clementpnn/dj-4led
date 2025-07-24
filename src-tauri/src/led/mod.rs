pub mod artnet;
pub mod controller;

// Types et structs partagés définis ici
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LedMode {
    Simulator,
    Production,
}

impl Default for LedMode {
    fn default() -> Self {
        LedMode::Simulator
    }
}

#[derive(Debug, Clone)]
pub struct ControllerConfig {
    pub name: String,
    pub ip_address: String,
    pub start_universe: u16,
    pub universe_count: u16,
}

#[derive(Debug, Clone, Default)]
pub struct LedStats {
    pub frames_sent: u64,
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub fps: f32,
    pub avg_frame_time_ms: f32,
    pub controllers_active: usize,
    pub universes_active: usize,
}

/// Configuration par défaut pour le mode simulateur
pub fn default_simulator_controllers() -> Vec<ControllerConfig> {
    vec![
        ControllerConfig {
            name: "Simulator 1".to_string(),
            ip_address: "127.0.0.1".to_string(),
            start_universe: 0,
            universe_count: 64,
        },
        ControllerConfig {
            name: "Simulator 2".to_string(),
            ip_address: "127.0.0.1".to_string(),
            start_universe: 64,
            universe_count: 64,
        },
        ControllerConfig {
            name: "Simulator 3".to_string(),
            ip_address: "127.0.0.1".to_string(),
            start_universe: 128,
            universe_count: 64,
        },
        ControllerConfig {
            name: "Simulator 4".to_string(),
            ip_address: "127.0.0.1".to_string(),
            start_universe: 192,
            universe_count: 64,
        },
    ]
}

/// Configuration par défaut pour le mode production
pub fn default_production_controllers() -> Vec<ControllerConfig> {
    vec![
        ControllerConfig {
            name: "LED Quarter 1".to_string(),
            ip_address: "192.168.1.45".to_string(),
            start_universe: 0,
            universe_count: 32,
        },
        ControllerConfig {
            name: "LED Quarter 2".to_string(),
            ip_address: "192.168.1.46".to_string(),
            start_universe: 32,
            universe_count: 32,
        },
        ControllerConfig {
            name: "LED Quarter 3".to_string(),
            ip_address: "192.168.1.47".to_string(),
            start_universe: 64,
            universe_count: 32,
        },
        ControllerConfig {
            name: "LED Quarter 4".to_string(),
            ip_address: "192.168.1.48".to_string(),
            start_universe: 96,
            universe_count: 32,
        },
    ]
}

// Re-exports pour faciliter l'utilisation depuis l'extérieur
pub use controller::{LedController, TestPattern, MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_SIZE};
pub use artnet::ArtNetClient;
