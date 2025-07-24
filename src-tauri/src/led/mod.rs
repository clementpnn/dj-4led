pub mod artnet;
pub mod controller;

pub use controller::{LedController, TestPattern};

use serde::{Deserialize, Serialize};

/// Mode de fonctionnement du contrôleur LED
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedMode {
    /// Mode simulateur (localhost)
    Simulator,
    /// Mode production (contrôleurs physiques)
    Production,
}

impl Default for LedMode {
    fn default() -> Self {
        LedMode::Simulator
    }
}

/// Configuration d'un contrôleur LED
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub ip_address: String,
    pub start_universe: u16,
    pub universe_count: u16,
    pub name: String,
}

/// Statistiques de performance LED
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LedStats {
    pub frames_sent: u64,
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub fps: f32,
    pub avg_frame_time_ms: f32,
    pub controllers_active: usize,
    pub universes_active: usize,
}

/// Retourne la configuration des contrôleurs pour le mode production
pub fn default_production_controllers() -> Vec<ControllerConfig> {
    vec![
        ControllerConfig {
            ip_address: "192.168.1.45".to_string(),
            start_universe: 0,
            universe_count: 32,
            name: "Controller 1".to_string(),
        },
        ControllerConfig {
            ip_address: "192.168.1.46".to_string(),
            start_universe: 32,
            universe_count: 32,
            name: "Controller 2".to_string(),
        },
        ControllerConfig {
            ip_address: "192.168.1.47".to_string(),
            start_universe: 64,
            universe_count: 32,
            name: "Controller 3".to_string(),
        },
        ControllerConfig {
            ip_address: "192.168.1.48".to_string(),
            start_universe: 96,
            universe_count: 32,
            name: "Controller 4".to_string(),
        },
    ]
}

/// Retourne la configuration des contrôleurs pour le mode simulateur
pub fn default_simulator_controllers() -> Vec<ControllerConfig> {
    vec![
        ControllerConfig {
            ip_address: "127.0.0.1".to_string(),
            start_universe: 0,
            universe_count: 256, // Tous les univers sur localhost
            name: "Simulator".to_string(),
        },
    ]
}
