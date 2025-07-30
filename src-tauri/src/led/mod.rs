pub mod artnet;
pub mod controller;

// Re-export pour compatibilité
pub use controller::LedController;

use serde::{Deserialize, Serialize};

// Configuration LED Production
pub const MATRIX_WIDTH: usize = 128;
pub const MATRIX_HEIGHT: usize = 128;
pub const MATRIX_SIZE: usize = MATRIX_WIDTH * MATRIX_HEIGHT * 3;
pub const TARGET_FPS: f32 = 77.0;
pub const FRAME_TIME_MS: u64 = 13; // ~77 FPS

/// Mode de fonctionnement - Production avec fallback Simulator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedMode {
    Production, // Mode principal
    Simulator,  // Mode fallback pour compatibilité
}

impl Default for LedMode {
    fn default() -> Self {
        LedMode::Production // TOUJOURS PRODUCTION PAR DÉFAUT
    }
}

/// Patterns de test pour production
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TestPattern {
    AllRed,
    AllGreen,
    AllBlue,
    AllWhite,
    Gradient,
    Checkerboard,
    QuarterTest,
}

impl TestPattern {
    pub fn all() -> Vec<TestPattern> {
        vec![
            TestPattern::AllRed,
            TestPattern::AllGreen,
            TestPattern::AllBlue,
            TestPattern::AllWhite,
            TestPattern::Gradient,
            TestPattern::Checkerboard,
            TestPattern::QuarterTest,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            TestPattern::AllRed => "Rouge Production",
            TestPattern::AllGreen => "Vert Production",
            TestPattern::AllBlue => "Bleu Production",
            TestPattern::AllWhite => "Blanc Production",
            TestPattern::Gradient => "Dégradé Production",
            TestPattern::Checkerboard => "Damier Production",
            TestPattern::QuarterTest => "Test Quarts Production",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TestPattern::AllRed => "Toutes les LEDs physiques en rouge",
            TestPattern::AllGreen => "Toutes les LEDs physiques en vert",
            TestPattern::AllBlue => "Toutes les LEDs physiques en bleu",
            TestPattern::AllWhite => "Toutes les LEDs physiques en blanc",
            TestPattern::Gradient => "Dégradé horizontal sur écran physique",
            TestPattern::Checkerboard => "Pattern damier sur écran physique",
            TestPattern::QuarterTest => "Test par contrôleur (4 quarts)",
        }
    }
}

// Fonctions utilitaires production
pub fn validate_frame_size(frame: &[u8]) -> Result<(), String> {
    if frame.len() != MATRIX_SIZE {
        return Err(format!("PRODUCTION: Taille frame invalide: {} (attendu {})",
                          frame.len(), MATRIX_SIZE));
    }
    Ok(())
}

pub fn validate_brightness(brightness: f32) -> Result<(), String> {
    if !(0.0..=1.0).contains(&brightness) {
        return Err("PRODUCTION: Brightness doit être entre 0.0 et 1.0".to_string());
    }
    Ok(())
}

pub fn create_test_pattern(pattern: &str, width: usize, height: usize) -> Vec<u8> {
    let size = width * height * 3;
    println!("🎨 [PATTERN] PRODUCTION Pattern '{}' - {}x{} = {} bytes",
             pattern, width, height, size);

    match pattern {
        "red" => {
            let mut frame = vec![0; size];
            for i in (0..size).step_by(3) {
                frame[i] = 255; // Rouge
            }
            println!("🎨 [PATTERN] PRODUCTION Rouge créé");
            frame
        }
        "green" => {
            let mut frame = vec![0; size];
            for i in (1..size).step_by(3) {
                frame[i] = 255; // Vert
            }
            println!("🎨 [PATTERN] PRODUCTION Vert créé");
            frame
        }
        "blue" => {
            let mut frame = vec![0; size];
            for i in (2..size).step_by(3) {
                frame[i] = 255; // Bleu
            }
            println!("🎨 [PATTERN] PRODUCTION Bleu créé");
            frame
        }
        "white" => {
            println!("🎨 [PATTERN] PRODUCTION Blanc créé");
            vec![255; size]
        }
        "black" | "off" => {
            println!("🎨 [PATTERN] PRODUCTION Noir/Off créé");
            vec![0; size]
        }
        "gradient" => {
            let mut frame = vec![0; size];
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 3;
                    let intensity = (x * 255 / width.max(1)) as u8;
                    frame[idx] = intensity;           // Rouge
                    frame[idx + 1] = intensity / 2;   // Vert
                    frame[idx + 2] = 255 - intensity; // Bleu
                }
            }
            println!("🎨 [PATTERN] PRODUCTION Gradient créé");
            frame
        }
        "checkerboard" => {
            let mut frame = vec![0; size];
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 3;
                    let is_white = (x / 8 + y / 8) % 2 == 0;
                    let value = if is_white { 255 } else { 0 };
                    frame[idx] = value;
                    frame[idx + 1] = value;
                    frame[idx + 2] = value;
                }
            }
            println!("🎨 [PATTERN] PRODUCTION Damier créé");
            frame
        }
        "quarter" => {
            // Pattern spécial pour tester les 4 contrôleurs
            let mut frame = vec![0; size];
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 3;
                    let quarter = (x / 32) % 4; // 4 quarts de 32 colonnes
                    match quarter {
                        0 => { frame[idx] = 255; frame[idx + 1] = 0; frame[idx + 2] = 0; }     // Rouge
                        1 => { frame[idx] = 0; frame[idx + 1] = 255; frame[idx + 2] = 0; }     // Vert
                        2 => { frame[idx] = 0; frame[idx + 1] = 0; frame[idx + 2] = 255; }     // Bleu
                        3 => { frame[idx] = 255; frame[idx + 1] = 255; frame[idx + 2] = 255; } // Blanc
                        _ => {}
                    }
                }
            }
            println!("🎨 [PATTERN] PRODUCTION Test Quarts créé");
            frame
        }
        _ => {
            println!("⚠️ [PATTERN] PRODUCTION Pattern '{}' inconnu, utilisation gradient", pattern);
            create_test_pattern("gradient", width, height)
        }
    }
}
