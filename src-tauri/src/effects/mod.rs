// src-tauri/src/effects/mod.rs

use std::sync::RwLock;

// Import des effets individuels
mod spectrum_bars;
mod circular_wave;
mod particle_system;
mod heartbeat;
mod starfall;
mod rain;
mod flames;
mod applaudimetre;

pub use spectrum_bars::*;
pub use circular_wave::*;
pub use particle_system::*;
pub use heartbeat::*;
pub use starfall::*;
pub use rain::*;
pub use flames::*;
pub use applaudimetre::*;

/// Trait principal pour tous les effets
pub trait Effect: Send + Sync {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]);
    fn set_color_mode(&mut self, mode: &str);
    fn set_custom_color(&mut self, r: f32, g: f32, b: f32);

    /// Nom de l'effet (optionnel)
    fn name(&self) -> &'static str {
        "Unknown Effect"
    }

    /// Description de l'effet (optionnel)
    fn description(&self) -> &'static str {
        "No description available"
    }

    /// Réinitialise l'effet à son état initial (optionnel)
    fn reset(&mut self) {}

    /// Indique si l'effet supporte les transitions (optionnel)
    fn supports_transitions(&self) -> bool {
        false
    }
}

/// Configuration globale des couleurs partagée entre tous les effets
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ColorConfig {
    pub mode: String,
    pub custom_color: (f32, f32, f32),
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            mode: "rainbow".to_string(),
            custom_color: (1.0, 0.0, 0.5),
        }
    }
}

/// Configuration globale thread-safe pour les couleurs
static GLOBAL_COLOR_CONFIG: RwLock<ColorConfig> = RwLock::new(ColorConfig {
    mode: String::new(),
    custom_color: (1.0, 0.0, 0.5),
});

/// Obtient la configuration couleur actuelle de manière thread-safe
pub fn get_color_config() -> ColorConfig {
    GLOBAL_COLOR_CONFIG.read().unwrap().clone()
}

/// Met à jour la configuration couleur de manière thread-safe
pub fn set_color_config(config: ColorConfig) {
    *GLOBAL_COLOR_CONFIG.write().unwrap() = config;
}

/// Moteur d'effets principal
pub struct EffectEngine {
    effects: Vec<Box<dyn Effect>>,
    current: usize,
    transition_progress: f32,
    transitioning: bool,
    target_effect: Option<usize>,
    color_config: ColorConfig,
    frame_buffer: Vec<u8>,
    last_spectrum: Vec<f32>,
}

impl EffectEngine {
    pub fn new() -> Self {
        // Initialiser la configuration globale
        set_color_config(ColorConfig::default());

        let mut engine = Self {
            effects: Vec::new(),
            current: 0,
            transition_progress: 0.0,
            transitioning: false,
            target_effect: None,
            color_config: ColorConfig::default(),
            frame_buffer: vec![0u8; 128 * 128 * 3],
            last_spectrum: vec![0.0; 64],
        };

        // Créer tous les effets
        engine.effects = vec![
            Box::new(SpectrumBars::new()) as Box<dyn Effect>,
            Box::new(CircularWave::new()) as Box<dyn Effect>,
            Box::new(ParticleSystem::new()) as Box<dyn Effect>,
            Box::new(Heartbeat::new()) as Box<dyn Effect>,
            Box::new(Starfall::new()) as Box<dyn Effect>,
            Box::new(Rain::new()) as Box<dyn Effect>,
            Box::new(Flames::new()) as Box<dyn Effect>,
            Box::new(Applaudimetre::new()) as Box<dyn Effect>,
        ];

        println!("✅ EffectEngine initialized with {} effects", engine.effects.len());
        engine
    }

    /// Rendu principal avec gestion des transitions
    pub fn render(&mut self, spectrum: &[f32]) -> Vec<u8> {
        // Sauvegarder le spectrum pour les transitions
        self.last_spectrum = spectrum.to_vec();

        // Réinitialiser le buffer
        self.frame_buffer.fill(0);

        if self.transitioning {
            self.render_with_transition(spectrum)
        } else {
            self.render_single_effect(spectrum)
        }
    }

    /// Rendu d'un seul effet
    fn render_single_effect(&mut self, spectrum: &[f32]) -> Vec<u8> {
        if let Some(effect) = self.effects.get_mut(self.current) {
            effect.render(spectrum, &mut self.frame_buffer);
        }
        self.frame_buffer.clone()
    }

    /// Rendu avec transition entre deux effets
    fn render_with_transition(&mut self, spectrum: &[f32]) -> Vec<u8> {
        if let Some(target) = self.target_effect {
            // Créer deux buffers pour les effets
            let mut current_frame = vec![0u8; 128 * 128 * 3];
            let mut target_frame = vec![0u8; 128 * 128 * 3];

            // Rendu de l'effet actuel
            if let Some(current_effect) = self.effects.get_mut(self.current) {
                current_effect.render(spectrum, &mut current_frame);
            }

            // Rendu de l'effet cible
            if let Some(target_effect) = self.effects.get_mut(target) {
                target_effect.render(spectrum, &mut target_frame);
            }

            // Mélanger les deux frames
            self.blend_frames(&current_frame, &target_frame, self.transition_progress);

            // Progresser la transition
            self.transition_progress += 0.02; // 2% par frame = ~50 frames = ~0.83 seconde à 60fps

            // Vérifier si la transition est terminée
            if self.transition_progress >= 1.0 {
                self.current = target;
                self.transitioning = false;
                self.target_effect = None;
                self.transition_progress = 0.0;
                println!("🔄 Transition completed to effect {}", self.current);
            }

            self.frame_buffer.clone()
        } else {
            self.render_single_effect(spectrum)
        }
    }

    /// Mélange deux frames selon un ratio
    fn blend_frames(&mut self, frame_a: &[u8], frame_b: &[u8], blend_ratio: f32) {
        let ratio = blend_ratio.clamp(0.0, 1.0);
        let inv_ratio = 1.0 - ratio;

        // FIX: Utilisation de chunks_mut au lieu de par_chunks_mut
        self.frame_buffer
            .chunks_mut(3)
            .enumerate()
            .for_each(|(i, pixel)| {
                let idx = i * 3;
                if idx + 2 < frame_a.len() && idx + 2 < frame_b.len() {
                    // FIX: Suppression des parenthèses inutiles
                    pixel[0] = (frame_a[idx] as f32 * inv_ratio + frame_b[idx] as f32 * ratio) as u8;
                    pixel[1] = (frame_a[idx + 1] as f32 * inv_ratio + frame_b[idx + 1] as f32 * ratio) as u8;
                    pixel[2] = (frame_a[idx + 2] as f32 * inv_ratio + frame_b[idx + 2] as f32 * ratio) as u8;
                }
            });
    }

    /// Change d'effet avec ou sans transition
    pub fn set_effect(&mut self, index: usize) -> Result<(), String> {
        if index >= self.effects.len() {
            return Err(format!("Effect index {} out of range (0-{})", index, self.effects.len() - 1));
        }

        if index == self.current {
            return Ok(()); // Déjà sur cet effet
        }

        // Vérifier si l'effet actuel supporte les transitions
        if let Some(current_effect) = self.effects.get(self.current) {
            if current_effect.supports_transitions() && !self.transitioning {
                // Démarrer une transition
                self.target_effect = Some(index);
                self.transitioning = true;
                self.transition_progress = 0.0;
                println!("🔄 Starting transition from effect {} to {}", self.current, index);
            } else {
                // Changement immédiat
                self.current = index;
                self.transitioning = false;
                self.target_effect = None;
                println!("⚡ Immediate switch to effect {}", index);
            }
        } else {
            self.current = index;
        }

        Ok(())
    }

    /// Change d'effet par nom
    pub fn set_effect_by_name(&mut self, name: &str) -> Result<(), String> {
        let index = self.effects
            .iter()
            .position(|effect| effect.name().to_lowercase() == name.to_lowercase())
            .ok_or_else(|| format!("Effect '{}' not found", name))?;

        self.set_effect(index)
    }

    /// Obtient l'index de l'effet actuel
    pub fn get_current_effect(&self) -> usize {
        self.current
    }

    /// Obtient le nom de l'effet actuel
    pub fn get_current_effect_name(&self) -> String {
        if let Some(effect) = self.effects.get(self.current) {
            effect.name().to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Obtient la liste des noms d'effets disponibles
    pub fn get_available_effects() -> Vec<&'static str> {
        vec![
            "SpectrumBars",
            "CircularWave",
            "ParticleSystem",
            "Heartbeat",
            "Starfall",
            "Rain",
            "Flames",
            "Applaudimetre",
        ]
    }

    /// Obtient la liste des noms d'effets
    pub fn get_effect_names(&self) -> Vec<String> {
        self.effects
            .iter()
            .map(|effect| effect.name().to_string())
            .collect()
    }

    /// Obtient les descriptions des effets
    pub fn get_effect_descriptions(&self) -> Vec<(String, String)> {
        self.effects
            .iter()
            .map(|effect| (effect.name().to_string(), effect.description().to_string()))
            .collect()
    }

    /// Change le mode couleur pour tous les effets
    pub fn set_color_mode(&mut self, mode: &str) {
        self.color_config.mode = mode.to_string();

        // Mettre à jour la configuration globale
        let mut global_config = self.color_config.clone();
        global_config.mode = mode.to_string();
        set_color_config(global_config);

        // Appliquer à tous les effets
        for effect in &mut self.effects {
            effect.set_color_mode(mode);
        }

        println!("🎨 Color mode changed to: {}", mode);
    }

    /// Obtient le mode couleur actuel
    pub fn get_color_mode(&self) -> String {
        self.color_config.mode.clone()
    }

    /// Change la couleur personnalisée pour tous les effets
    pub fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        self.color_config.custom_color = (r, g, b);

        // Mettre à jour la configuration globale
        let mut global_config = self.color_config.clone();
        global_config.custom_color = (r, g, b);
        set_color_config(global_config);

        // Appliquer à tous les effets
        for effect in &mut self.effects {
            effect.set_custom_color(r, g, b);
        }

        println!("🎨 Custom color changed to: ({:.2}, {:.2}, {:.2})", r, g, b);
    }

    /// Obtient la couleur personnalisée actuelle
    pub fn get_custom_color(&self) -> (f32, f32, f32) {
        self.color_config.custom_color
    }

    /// Réinitialise tous les effets
    pub fn reset_all_effects(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
        println!("🔄 Reset all effects");
    }

    /// Obtient des statistiques sur l'engine
    pub fn get_stats(&self) -> EffectEngineStats {
        EffectEngineStats {
            current_effect: self.current,
            current_effect_name: self.get_current_effect_name(),
            total_effects: self.effects.len(),
            transitioning: self.transitioning,
            transition_progress: self.transition_progress,
            color_mode: self.color_config.mode.clone(),
            custom_color: self.color_config.custom_color,
        }
    }

    /// Obtient la configuration couleur actuelle
    pub fn get_color_config(&self) -> &ColorConfig {
        &self.color_config
    }

}

/// Statistiques de l'engine d'effets
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EffectEngineStats {
    pub current_effect: usize,
    pub current_effect_name: String,
    pub total_effects: usize,
    pub transitioning: bool,
    pub transition_progress: f32,
    pub color_mode: String,
    pub custom_color: (f32, f32, f32),
}

/// Fonction utilitaire pour la conversion HSV vers RGB
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
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

/// Générateur de nombres pseudo-aléatoires thread-safe
pub fn rand() -> f32 {
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

/// Modes couleur disponibles
pub const COLOR_MODES: &[&str] = &[
    "rainbow",
    "fire",
    "ocean",
    "sunset",
    "custom",
];

/// Vérifie si un mode couleur est valide
pub fn is_valid_color_mode(mode: &str) -> bool {
    COLOR_MODES.contains(&mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_engine_creation() {
        let engine = EffectEngine::new();
        assert_eq!(engine.effects.len(), 8);
        assert_eq!(engine.current, 0);
        assert!(!engine.transitioning);
    }

    #[test]
    fn test_effect_switching() {
        let mut engine = EffectEngine::new();

        assert!(engine.set_effect(2).is_ok());
        assert_eq!(engine.get_current_effect(), 2);

        assert!(engine.set_effect(10).is_err()); // Index invalide
        assert_eq!(engine.get_current_effect(), 2); // Pas changé
    }

    #[test]
    fn test_color_config() {
        let mut engine = EffectEngine::new();

        engine.set_color_mode("fire");
        assert_eq!(engine.color_config.mode, "fire");

        engine.set_custom_color(0.5, 0.7, 0.9);
        assert_eq!(engine.color_config.custom_color, (0.5, 0.7, 0.9));
    }

    #[test]
    fn test_hsv_to_rgb() {
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0); // Rouge pur
        assert!((r - 1.0).abs() < 0.01);
        assert!(g < 0.01);
        assert!(b < 0.01);

        let (r, g, b) = hsv_to_rgb(0.333, 1.0, 1.0); // Vert pur
        assert!(r < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!(b < 0.01);
    }

    #[test]
    fn test_color_modes() {
        assert!(is_valid_color_mode("rainbow"));
        assert!(is_valid_color_mode("fire"));
        assert!(!is_valid_color_mode("invalid_mode"));
    }

    #[test]
    fn test_global_color_config() {
        let config = ColorConfig {
            mode: "test_mode".to_string(),
            custom_color: (0.1, 0.2, 0.3),
        };

        set_color_config(config.clone());
        let retrieved = get_color_config();

        assert_eq!(retrieved.mode, "test_mode");
        assert_eq!(retrieved.custom_color, (0.1, 0.2, 0.3));
    }
}
