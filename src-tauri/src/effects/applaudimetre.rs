// src-tauri/src/effects/applaudimetre.rs

use super::{Effect, get_color_config, hsv_to_rgb, rand};
use rayon::prelude::*;

/// Particule d'éclat pour les effets de pointe
#[derive(Clone)]
struct PeakSparkle {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    life: f32,
    max_life: f32,
    brightness: f32,
    color: (f32, f32, f32),
    size: f32,
}

impl PeakSparkle {
    fn new(x: f32, y: f32, color: (f32, f32, f32)) -> Self {
        let life = 0.8 + rand() * 0.7;
        Self {
            x,
            y,
            velocity_x: (rand() - 0.5) * 2.0,
            velocity_y: -0.5 - rand() * 1.5,
            life,
            max_life: life,
            brightness: 0.7 + rand() * 0.3,
            color,
            size: 1.0 + rand() * 2.0,
        }
    }

    fn update(&mut self) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;
        self.velocity_y += 0.05; // Gravité
        self.velocity_x *= 0.98; // Friction
        self.life -= 0.015;
    }

    fn is_alive(&self) -> bool {
        self.life > 0.0
    }

    fn get_alpha(&self) -> f32 {
        (self.life / self.max_life).powf(0.5)
    }
}

/// Effet d'applaudimètre avec visualisation de niveau sonore
pub struct Applaudimetre {
    // Niveaux audio
    current_level: f32,
    max_level: f32,
    max_hold_time: f32,
    smoothed_level: f32,

    // Historiques
    peak_history: Vec<f32>,
    level_history: Vec<f32>,

    // Animation
    animation_time: f32,
    background_pulse: f32,

    // Particules
    peak_sparkles: Vec<PeakSparkle>,

    // Configuration
    sensitivity: f32,
    auto_gain: f32,
    decay_rate: f32,

    // Rendu
    bar_width: usize,
    bar_left: usize,
    bar_right: usize,

    // Modes de visualisation
    show_peak_hold: bool,
    show_history_trail: bool,
    show_sparkles: bool,
    show_graduations: bool,
}

impl Applaudimetre {
    pub fn new() -> Self {
        let bar_width = 48; // Largeur de la barre
        let bar_left = (128 - bar_width) / 2; // Centré
        let bar_right = bar_left + bar_width;

        Self {
            current_level: 0.0,
            max_level: 0.0,
            max_hold_time: 0.0,
            smoothed_level: 0.0,
            peak_history: vec![0.0; 60], // 1 seconde à 60fps
            level_history: vec![0.0; bar_width],
            animation_time: 0.0,
            background_pulse: 0.0,
            peak_sparkles: Vec::new(),
            sensitivity: 2.5,
            auto_gain: 1.0,
            decay_rate: 0.006,
            bar_width,
            bar_left,
            bar_right,
            show_peak_hold: true,
            show_history_trail: true,
            show_sparkles: true,
            show_graduations: true,
        }
    }

    /// Calcule le niveau audio pondéré à partir du spectre
    fn calculate_audio_level(&mut self, spectrum: &[f32]) -> f32 {
        if spectrum.is_empty() {
            return 0.0;
        }

        // Pondération par fréquence (bass plus fort, aigus plus faibles)
        let bass_weight = 0.6;   // 0-8 (graves)
        let mid_weight = 0.3;    // 8-24 (médiums)
        let high_weight = 0.1;   // 24+ (aigus)

        let spectrum_len = spectrum.len();
        let bass_end = (spectrum_len / 8).min(spectrum_len);
        let mid_end = (spectrum_len * 3 / 8).min(spectrum_len);

        let bass_level = if bass_end > 0 {
            spectrum[..bass_end].iter().sum::<f32>() / bass_end as f32
        } else {
            0.0
        };

        let mid_level = if mid_end > bass_end {
            spectrum[bass_end..mid_end].iter().sum::<f32>() / (mid_end - bass_end) as f32
        } else {
            0.0
        };

        let high_level = if spectrum_len > mid_end {
            spectrum[mid_end..].iter().sum::<f32>() / (spectrum_len - mid_end) as f32
        } else {
            0.0
        };

        let raw_level = (bass_level * bass_weight + mid_level * mid_weight + high_level * high_weight)
            * self.sensitivity;

        // Auto-gain adaptatif
        if raw_level > 0.01 {
            let avg_recent = self.peak_history.iter().sum::<f32>() / self.peak_history.len() as f32;

            if avg_recent < 0.15 {
                self.auto_gain = (self.auto_gain + 0.008).min(3.0);
            } else if avg_recent > 0.85 {
                self.auto_gain = (self.auto_gain - 0.008).max(0.3);
            }
        }

        let final_level = (raw_level * self.auto_gain).powf(0.65);
        final_level.clamp(0.0, 1.0)
    }

    /// Obtient la couleur selon le niveau et le mode couleur
    fn get_color_for_level(&self, level: f32, is_max_indicator: bool) -> (f32, f32, f32) {
        let color_config = get_color_config();

        if is_max_indicator {
            // Couleur pour l'indicateur de pic
            match color_config.mode.as_str() {
                "rainbow" => {
                    let hue = (self.animation_time * 0.01) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    (r * 1.2, g * 1.2, b * 1.2)
                }
                "fire" => (1.0, 0.9, 0.1),
                "ocean" => (0.1, 0.9, 1.0),
                "sunset" => (1.0, 0.5, 0.0),
                "custom" => {
                    let (r, g, b) = color_config.custom_color;
                    (r * 1.5, g * 1.5, b * 1.5)
                }
                _ => (1.0, 1.0, 1.0),
            }
        } else {
            // Couleur progressive selon le niveau
            match color_config.mode.as_str() {
                "rainbow" => {
                    if level < 0.2 {
                        let t = level * 5.0;
                        (0.0, t * 0.4, 0.8 + t * 0.2)
                    } else if level < 0.4 {
                        let t = (level - 0.2) * 5.0;
                        (0.0, 0.4 + t * 0.6, 1.0 - t * 0.5)
                    } else if level < 0.7 {
                        let t = (level - 0.4) / 0.3;
                        (t * 0.8, 1.0, 0.5 - t * 0.5)
                    } else {
                        let t = (level - 0.7) / 0.3;
                        (0.8 + t * 0.2, 1.0 - t * 0.3, t * 0.2)
                    }
                }
                "fire" => {
                    if level < 0.3 {
                        let t = level / 0.3;
                        (t * 0.9, t * 0.1, 0.0)
                    } else if level < 0.6 {
                        let t = (level - 0.3) / 0.3;
                        (0.9, 0.1 + t * 0.5, 0.0)
                    } else {
                        let t = (level - 0.6) / 0.4;
                        (0.9 + t * 0.1, 0.6 + t * 0.4, t * 0.4)
                    }
                }
                "ocean" => {
                    if level < 0.4 {
                        let t = level / 0.4;
                        (0.0, t * 0.2, 0.3 + t * 0.5)
                    } else {
                        let t = (level - 0.4) / 0.6;
                        (t * 0.3, 0.2 + t * 0.6, 0.8 + t * 0.2)
                    }
                }
                "sunset" => {
                    if level < 0.5 {
                        let t = level * 2.0;
                        (0.4 + t * 0.3, 0.05 + t * 0.15, 0.6 + t * 0.2)
                    } else {
                        let t = (level - 0.5) * 2.0;
                        (0.7 + t * 0.3, 0.2 + t * 0.6, 0.8 - t * 0.6)
                    }
                }
                "custom" => {
                    let (r, g, b) = color_config.custom_color;
                    let intensity = (level * 0.8 + 0.2).min(1.0);
                    (r * intensity, g * intensity, b * intensity)
                }
                _ => {
                    let intensity = level.max(0.1);
                    (0.2 + intensity * 0.8, 0.8 * intensity, 0.6 + intensity * 0.4)
                }
            }
        }
    }

    /// Met à jour les particules d'éclat
    fn update_sparkles(&mut self) {
        // Mise à jour des particules existantes
        self.peak_sparkles.retain_mut(|sparkle| {
            sparkle.update();
            sparkle.is_alive()
        });

        // Création de nouvelles particules lors de pics
        if self.current_level > 0.25 && rand() < (self.current_level * 0.6) {
            let num_sparkles = 1 + (self.current_level * 4.0) as usize;
            let bar_center = (self.bar_left + self.bar_right) as f32 / 2.0;
            let peak_y = 127.0 - self.max_level * 127.0;

            for _ in 0..num_sparkles {
                let x = bar_center + (rand() - 0.5) * (self.bar_width as f32 * 1.5);
                let y = peak_y + (rand() - 0.5) * 20.0;
                let color = self.get_color_for_level(self.current_level, false);

                self.peak_sparkles.push(PeakSparkle::new(x, y, color));
            }
        }
    }

    /// Dessine les graduations sur les côtés
    fn draw_graduations(&self, frame: &mut [u8]) {
        if !self.show_graduations {
            return;
        }

        let grad_positions = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

        for &grad_level in &grad_positions {
            let grad_y = (127.0 - grad_level * 127.0) as usize;
            let intensity = if grad_level == 1.0 {
                0.8
            } else if grad_level % 0.2 < 0.05 {
                0.6
            } else {
                0.3
            };

            let brightness = (intensity * self.background_pulse * 200.0) as u8;

            // Graduations à gauche et à droite
            for grad_x in [self.bar_left.saturating_sub(2), self.bar_right + 1] {
                if grad_x < 128 && grad_y < 128 {
                    let idx = (grad_y * 128 + grad_x) * 3;
                    if idx + 2 < frame.len() {
                        frame[idx] = brightness;
                        frame[idx + 1] = brightness;
                        frame[idx + 2] = brightness;
                    }
                }
            }
        }
    }

    /// Dessine la traînée historique
    fn draw_history_trail(&self, frame: &mut [u8]) {
        if !self.show_history_trail {
            return;
        }

        for (i, &historical_level) in self.level_history.iter().enumerate() {
            if historical_level < 0.05 {
                continue;
            }

            let x = self.bar_left + i;
            if x >= self.bar_right {
                continue;
            }

            let bar_height = (historical_level * 127.0) as usize;

            for y in (127 - bar_height)..128 {
                let y_normalized = (127 - y) as f32 / 127.0;
                let trail_intensity = 0.15 + (historical_level / self.current_level.max(0.1)) * 0.25;
                let (r, g, b) = self.get_color_for_level(y_normalized, false);

                let idx = (y * 128 + x) * 3;
                if idx + 2 < frame.len() {
                    frame[idx] = (r * trail_intensity * 255.0) as u8;
                    frame[idx + 1] = (g * trail_intensity * 255.0) as u8;
                    frame[idx + 2] = (b * trail_intensity * 255.0) as u8;
                }
            }
        }
    }

    /// Dessine la barre principale
    fn draw_main_bar(&self, frame: &mut [u8]) {
        let bar_height = (self.current_level * 127.0) as usize;
        let bar_center = (self.bar_left + self.bar_right) as f32 / 2.0;

        frame.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = i % 128;
            let y = i / 128;

            if x >= self.bar_left && x < self.bar_right && y >= (127 - bar_height) {
                let y_normalized = (127 - y) as f32 / 127.0;

                // Effet de profondeur selon la distance du centre
                let distance_from_center = ((x as f32 - bar_center) / (self.bar_width as f32 / 2.0)).abs();
                let center_glow = (1.0 - distance_from_center * 0.7).max(0.3);

                // Pulsation selon le niveau audio
                let pulse = 1.0 + self.current_level * (self.animation_time * 0.08).sin() * 0.15;

                let brightness = (0.6 + y_normalized * 0.4) * center_glow * pulse;
                let (r, g, b) = self.get_color_for_level(y_normalized, false);

                pixel[0] = (r * brightness * 255.0).min(255.0) as u8;
                pixel[1] = (g * brightness * 255.0).min(255.0) as u8;
                pixel[2] = (b * brightness * 255.0).min(255.0) as u8;
            }
        });
    }

    /// Dessine l'indicateur de pic maximum
    fn draw_peak_indicator(&self, frame: &mut [u8]) {
        if !self.show_peak_hold || self.max_level < 0.05 {
            return;
        }

        let max_y = (127.0 - self.max_level * 127.0) as usize;
        let (r, g, b) = self.get_color_for_level(self.max_level, true);

        // Clignotement selon le temps de hold
        let blink_factor = if self.max_hold_time < 3.0 {
            0.9 + 0.1 * (self.max_hold_time * 10.0).sin()
        } else {
            0.5 + 0.5 * (self.max_hold_time * 3.0).sin().abs()
        };

        // Dessiner sur plusieurs lignes pour plus de visibilité
        for dy in -1i32..=1i32 {
            let y = (max_y as i32 + dy) as usize;
            if y >= 128 {
                continue;
            }

            let line_intensity = if dy == 0 { 1.0 } else { 0.7 };

            for x in self.bar_left..self.bar_right {
                let idx = (y * 128 + x) * 3;
                if idx + 2 < frame.len() {
                    let brightness = blink_factor * line_intensity;
                    frame[idx] = (r * brightness * 255.0) as u8;
                    frame[idx + 1] = (g * brightness * 255.0) as u8;
                    frame[idx + 2] = (b * brightness * 255.0) as u8;
                }
            }
        }
    }

    /// Dessine les particules d'éclat
    fn draw_sparkles(&self, frame: &mut [u8]) {
        if !self.show_sparkles {
            return;
        }

        for sparkle in &self.peak_sparkles {
            let x = sparkle.x as i32;
            let y = sparkle.y as i32;

            if x < 0 || x >= 128 || y < 0 || y >= 128 {
                continue;
            }

            let alpha = sparkle.get_alpha();
            let intensity = sparkle.brightness * alpha;

            // Dessiner la particule avec sa taille
            let size = (sparkle.size * alpha) as i32;
            for dx in -size..=size {
                for dy in -size..=size {
                    let px = x + dx;
                    let py = y + dy;

                    if px < 0 || px >= 128 || py < 0 || py >= 128 {
                        continue;
                    }

                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    if distance <= sparkle.size * alpha {
                        let idx = (py as usize * 128 + px as usize) * 3;
                        if idx + 2 < frame.len() {
                            let fade = (1.0 - distance / (sparkle.size * alpha)).max(0.0);
                            let final_intensity = intensity * fade;

                            let new_r = (sparkle.color.0 * final_intensity * 255.0) as u8;
                            let new_g = (sparkle.color.1 * final_intensity * 255.0) as u8;
                            let new_b = (sparkle.color.2 * final_intensity * 255.0) as u8;

                            frame[idx] = frame[idx].saturating_add(new_r);
                            frame[idx + 1] = frame[idx + 1].saturating_add(new_g);
                            frame[idx + 2] = frame[idx + 2].saturating_add(new_b);
                        }
                    }
                }
            }
        }
    }

    /// Dessine le cadre autour de la barre
    fn draw_frame(&self, frame: &mut [u8]) {
        let glow_intensity = (100.0 + self.current_level * 80.0) as u8;

        // Cadre horizontal (haut et bas)
        for x in (self.bar_left.saturating_sub(1))..=(self.bar_right) {
            if x < 128 {
                // Ligne du haut
                let idx_top = x * 3;
                if idx_top + 2 < frame.len() {
                    frame[idx_top] = glow_intensity;
                    frame[idx_top + 1] = glow_intensity;
                    frame[idx_top + 2] = glow_intensity;
                }

                // Ligne du bas
                let idx_bottom = (127 * 128 + x) * 3;
                if idx_bottom + 2 < frame.len() {
                    frame[idx_bottom] = glow_intensity;
                    frame[idx_bottom + 1] = glow_intensity;
                    frame[idx_bottom + 2] = glow_intensity;
                }
            }
        }

        // Cadre vertical (gauche et droite)
        for y in 0..128 {
            // Ligne de gauche
            if self.bar_left > 0 {
                let idx_left = (y * 128 + self.bar_left - 1) * 3;
                if idx_left + 2 < frame.len() {
                    frame[idx_left] = glow_intensity;
                    frame[idx_left + 1] = glow_intensity;
                    frame[idx_left + 2] = glow_intensity;
                }
            }

            // Ligne de droite
            if self.bar_right < 128 {
                let idx_right = (y * 128 + self.bar_right) * 3;
                if idx_right + 2 < frame.len() {
                    frame[idx_right] = glow_intensity;
                    frame[idx_right + 1] = glow_intensity;
                    frame[idx_right + 2] = glow_intensity;
                }
            }
        }
    }
}

impl Effect for Applaudimetre {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Calcul du niveau audio
        let raw_level = self.calculate_audio_level(spectrum);

        // Lissage adaptatif
        let smoothing = if raw_level > self.smoothed_level {
            0.3 // Montée rapide
        } else {
            0.88 // Descente lente
        };

        self.smoothed_level = self.smoothed_level * smoothing + raw_level * (1.0 - smoothing);
        self.current_level = self.smoothed_level;

        // Mise à jour des historiques
        self.level_history.remove(0);
        self.level_history.push(self.current_level);

        self.peak_history.remove(0);
        self.peak_history.push(self.current_level);

        // Gestion du pic maximum
        let recent_max = self.peak_history.iter().fold(0.0f32, |a, &b| a.max(b));

        if self.current_level > self.max_level {
            self.max_level = self.current_level;
            self.max_hold_time = 0.0;
        } else {
            self.max_hold_time += 1.0 / 60.0; // Assume 60 FPS

            if self.max_hold_time >= 3.0 {
                self.max_level = (self.max_level - self.decay_rate).max(recent_max);
                if self.max_level <= recent_max {
                    self.max_hold_time = 0.0;
                }
            }
        }

        // Mise à jour de l'animation
        self.animation_time += 1.0 + self.current_level * 3.0;
        self.background_pulse = (self.animation_time * 0.04).sin() * 0.2 + 0.8;

        // Mise à jour des particules
        self.update_sparkles();

        // Effacer le frame
        frame.fill(0);

        // Rendu des éléments (ordre important pour la superposition)
        self.draw_graduations(frame);
        self.draw_history_trail(frame);
        self.draw_main_bar(frame);
        self.draw_peak_indicator(frame);
        self.draw_sparkles(frame);
        self.draw_frame(frame);
    }

    fn set_color_mode(&mut self, _mode: &str) {
        // La configuration couleur est gérée globalement
        // L'effet s'adapte automatiquement via get_color_config()
    }

    fn set_custom_color(&mut self, _r: f32, _g: f32, _b: f32) {
        // La configuration couleur est gérée globalement
        // L'effet s'adapte automatiquement via get_color_config()
    }

    fn name(&self) -> &'static str {
        "Applaudimetre"
    }

    fn description(&self) -> &'static str {
        "Visualisateur de niveau sonore avec indicateur de pic, traînée historique et effets de particules"
    }

    fn reset(&mut self) {
        self.current_level = 0.0;
        self.max_level = 0.0;
        self.max_hold_time = 0.0;
        self.smoothed_level = 0.0;
        self.peak_history.fill(0.0);
        self.level_history.fill(0.0);
        self.animation_time = 0.0;
        self.background_pulse = 0.0;
        self.peak_sparkles.clear();
        self.auto_gain = 1.0;
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}
