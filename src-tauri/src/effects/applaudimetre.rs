// src-tauri/src/effects/applaudimetre.rs

use super::*;

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
    frame_count: u64,
    decay_timer: f32,
    noise_gate: f32,
}

struct PeakSparkle {
    x: f32,
    y: f32,
    life: f32,
    max_life: f32,
    brightness: f32,
    color: (f32, f32, f32),
    velocity_x: f32,
    velocity_y: f32,
    size: f32,
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
            level_history: vec![0.0; 48], // Réduit pour optimisation
            peak_sparkles: Vec::with_capacity(50), // Pré-allocation
            sensitivity: 2.8,
            auto_gain: 1.0,
            background_pulse: 0.0,
            frame_count: 0,
            decay_timer: 0.0,
            noise_gate: 0.02, // Seuil de bruit
        }
    }

    fn get_color_for_level(&self, level: f32, is_max_indicator: bool) -> (f32, f32, f32) {
        let color_config = get_color_config();

        if is_max_indicator {
            match color_config.mode.as_str() {
                "rainbow" => {
                    let hue = (self.animation_time * 0.015) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 0.9, 1.0);
                    (r * 1.2, g * 1.2, b * 1.2)
                }
                "fire" => (1.0, 0.9, 0.1),
                "ocean" => (0.1, 0.9, 1.0),
                "sunset" => (1.0, 0.5, 0.0),
                "custom" => {
                    let (r, g, b) = color_config.custom_color;
                    (
                        (r * 1.5).min(1.0),
                        (g * 1.5).min(1.0),
                        (b * 1.5).min(1.0),
                    )
                }
                _ => (1.0, 1.0, 1.0),
            }
        } else {
            let normalized_level = level.clamp(0.0, 1.0);
            match color_config.mode.as_str() {
                "rainbow" => {
                    // Amélioration du dégradé arc-en-ciel
                    let hue = normalized_level * 0.8; // 0.0 à 0.8 pour éviter le magenta
                    hsv_to_rgb(hue, 0.95, normalized_level.max(0.2))
                }
                "fire" => {
                    if normalized_level < 0.3 {
                        let t = normalized_level / 0.3;
                        (t * 0.9, t * 0.1, 0.0)
                    } else if normalized_level < 0.7 {
                        let t = (normalized_level - 0.3) / 0.4;
                        (0.9 + t * 0.1, 0.1 + t * 0.5, t * 0.1)
                    } else {
                        let t = (normalized_level - 0.7) / 0.3;
                        (1.0, 0.6 + t * 0.4, 0.1 + t * 0.4)
                    }
                }
                "ocean" => {
                    if normalized_level < 0.4 {
                        let t = normalized_level / 0.4;
                        (0.0, t * 0.4, 0.3 + t * 0.5)
                    } else {
                        let t = (normalized_level - 0.4) / 0.6;
                        (t * 0.3, 0.4 + t * 0.5, 0.8 + t * 0.2)
                    }
                }
                "sunset" => {
                    if normalized_level < 0.5 {
                        let t = normalized_level / 0.5;
                        (0.4 + t * 0.4, 0.1 + t * 0.2, 0.6 + t * 0.2)
                    } else {
                        let t = (normalized_level - 0.5) / 0.5;
                        (0.8 + t * 0.2, 0.3 + t * 0.5, 0.8 - t * 0.6)
                    }
                }
                "custom" => {
                    let (r, g, b) = color_config.custom_color;
                    let intensity = normalized_level.max(0.1);
                    (r * intensity, g * intensity, b * intensity)
                }
                _ => hsv_to_rgb(0.3, 1.0, normalized_level.max(0.3)),
            }
        }
    }

    fn calculate_audio_level(&mut self, spectrum: &[f32]) -> f32 {
        if spectrum.is_empty() {
            return 0.0;
        }

        // Amélioration de l'analyse spectrale avec pondération logarithmique
        let spectrum_len = spectrum.len().min(64);
        let bass_end = (spectrum_len * 12 / 64).max(1);
        let mid_end = (spectrum_len * 32 / 64).max(bass_end + 1);

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
            spectrum[mid_end..spectrum_len].iter().sum::<f32>() / (spectrum_len - mid_end) as f32
        } else {
            0.0
        };

        // Pondération améliorée pour l'applaudissement
        let raw_level = (bass_level * 0.6 + mid_level * 0.3 + high_level * 0.1) * self.sensitivity;

        // Gate de bruit adaptatif
        if raw_level < self.noise_gate {
            return 0.0;
        }

        // Auto-gain avec hystérésis
        if raw_level > 0.01 {
            let avg_recent = self.peak_history.iter().sum::<f32>() / self.peak_history.len() as f32;
            let gain_adjustment = 0.005; // Plus lent pour plus de stabilité

            if avg_recent < 0.15 && self.auto_gain < 2.5 {
                self.auto_gain += gain_adjustment;
            } else if avg_recent > 0.85 && self.auto_gain > 0.3 {
                self.auto_gain -= gain_adjustment;
            }

            // Ajustement du seuil de bruit basé sur l'activité
            if avg_recent > 0.1 {
                self.noise_gate = (self.noise_gate * 0.99 + avg_recent * 0.05 * 0.01).min(0.05);
            }
        }

        // Courbe de réponse logarithmique pour les applaudissements
        let processed_level = (raw_level * self.auto_gain).powf(0.65);
        processed_level.min(1.0)
    }

    fn update_sparkles(&mut self) {
        // Mise à jour des étincelles existantes avec physique améliorée
        self.peak_sparkles.retain_mut(|sparkle| {
            sparkle.life -= 0.025;
            sparkle.x += sparkle.velocity_x;
            sparkle.y += sparkle.velocity_y;
            sparkle.velocity_x *= 0.98; // Friction
            sparkle.velocity_y -= 0.1; // Gravité
            sparkle.size *= 0.995; // Rétrécissement progressif

            // Effet de scintillement
            sparkle.brightness = (sparkle.life / sparkle.max_life) *
                (0.7 + 0.3 * (self.animation_time * 0.3 + sparkle.x * 0.1).sin());

            sparkle.life > 0.0 && sparkle.x >= 0.0 && sparkle.x < 128.0
        });

        // Génération de nouvelles étincelles avec contrôle de densité
        if self.current_level > 0.25 && rand() < (0.3 + self.current_level * 0.4) {
            let sparkle_count = (1.0 + self.current_level * 4.0) as usize;
            let max_y = 127.0 - self.max_level * 127.0;

            for _ in 0..sparkle_count.min(8) {
                if self.peak_sparkles.len() < 50 { // Limite pour les performances
                    let angle = rand() * std::f32::consts::PI * 2.0;
                    let speed = 0.5 + rand() * 1.5;

                    self.peak_sparkles.push(PeakSparkle {
                        x: 64.0 + (rand() - 0.5) * 30.0,
                        y: max_y + (rand() - 0.5) * 8.0,
                        life: 0.6 + rand() * 0.8,
                        max_life: 1.0,
                        brightness: 0.8 + rand() * 0.2,
                        color: self.get_color_for_level(self.current_level, false),
                        velocity_x: angle.cos() * speed * (rand() - 0.5),
                        velocity_y: -speed * (0.5 + rand() * 0.5),
                        size: 0.8 + rand() * 0.4,
                    });
                }
            }
        }
    }

    fn render_bar(&self, frame: &mut [u8]) {
        let bar_left = 38;
        let bar_right = 90;
        let bar_center = (bar_left + bar_right) / 2;

        // FIX: Utilisation de chunks_mut au lieu de par_chunks_mut
        frame.chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = i % 128;
            let y = i / 128;

            if x >= bar_left && x < bar_right {
                let y_norm = (127 - y) as f32 / 127.0;
                let bar_height = self.current_level.min(1.0);

                // Effet de traînée optimisé
                if y_norm <= bar_height && bar_height > 0.02 {
                    let level_intensity = (y_norm / bar_height.max(0.01)).powf(0.8);

                    // Effet de brillance au centre
                    let center_dist = ((x - bar_center) as f32 / ((bar_right - bar_left) / 2) as f32).abs();
                    let center_glow = (1.0 - center_dist).max(0.0).powf(1.5);

                    let brightness = 0.6 + level_intensity * 0.3 + center_glow * 0.4;
                    let (r, g, b) = self.get_color_for_level(level_intensity, false);

                    // Pulsation dynamique
                    let pulse = 1.0 + self.current_level * 0.15 * (self.animation_time * 0.08).sin();

                    pixel[0] = (r * brightness * pulse * 255.0).min(255.0) as u8;
                    pixel[1] = (g * brightness * pulse * 255.0).min(255.0) as u8;
                    pixel[2] = (b * brightness * pulse * 255.0).min(255.0) as u8;
                }

                // Indicateur de maximum amélioré
                let max_y = (127.0 - self.max_level * 127.0) as usize;
                if y >= max_y.saturating_sub(1) && y <= max_y.saturating_add(1) && self.max_level > 0.03 {
                    let (r, g, b) = self.get_color_for_level(self.max_level, true);

                    let blink_factor = if self.max_hold_time < 5.0 {
                        0.85 + 0.15 * (self.animation_time * 0.2).sin()
                    } else {
                        0.5 + 0.3 * (self.max_hold_time * 1.5).sin().abs()
                    };

                    pixel[0] = (r * blink_factor * 255.0) as u8;
                    pixel[1] = (g * blink_factor * 255.0) as u8;
                    pixel[2] = (b * blink_factor * 255.0) as u8;
                }
            }

            // Cadre avec lueur adaptative
            let is_frame = ((x == bar_left - 1 || x == bar_right) && y < 128) ||
                          ((y == 0 || y == 127) && x >= bar_left - 1 && x <= bar_right);

            if is_frame {
                let glow_intensity = (60.0 + self.current_level * 40.0) * self.background_pulse;
                pixel[0] = glow_intensity as u8;
                pixel[1] = glow_intensity as u8;
                pixel[2] = glow_intensity as u8;
            }
        });
    }

    fn render_sparkles(&self, frame: &mut [u8]) {
        for sparkle in &self.peak_sparkles {
            let base_x = sparkle.x as i32;
            let base_y = sparkle.y as i32;
            let size = sparkle.size as i32;

            // Rendu avec anti-aliasing simple pour les étincelles
            for dy in -size..=size {
                for dx in -size..=size {
                    let x = base_x + dx;
                    let y = base_y + dy;

                    if x >= 0 && x < 128 && y >= 0 && y < 128 {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        if distance <= sparkle.size {
                            let idx = ((y * 128 + x) * 3) as usize;
                            if idx + 2 < frame.len() {
                                let falloff = (1.0 - distance / sparkle.size).max(0.0);
                                let intensity = sparkle.brightness * falloff * 255.0;

                                frame[idx] = frame[idx].saturating_add((sparkle.color.0 * intensity) as u8);
                                frame[idx + 1] = frame[idx + 1].saturating_add((sparkle.color.1 * intensity) as u8);
                                frame[idx + 2] = frame[idx + 2].saturating_add((sparkle.color.2 * intensity) as u8);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Effect for Applaudimetre {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Calcul du niveau audio
        let level = self.calculate_audio_level(spectrum);

        // Lissage adaptatif optimisé
        let smoothing = if level > self.smoothed_level { 0.3 } else { 0.88 };
        self.smoothed_level = self.smoothed_level * smoothing + level * (1.0 - smoothing);
        self.current_level = self.smoothed_level;

        // Mise à jour de l'historique (rotation efficace)
        if !self.level_history.is_empty() {
            self.level_history.rotate_left(1);
            let len = self.level_history.len();
            if len > 0 {
                self.level_history[len - 1] = self.current_level;
            }
        }

        // Gestion du pic avec decay amélioré
        if !self.peak_history.is_empty() {
            self.peak_history.rotate_left(1);
            let len = self.peak_history.len();
            if len > 0 {
                self.peak_history[len - 1] = self.current_level;
            }
        }

        let recent_max = self.peak_history.iter().fold(0.0f32, |acc, &x| acc.max(x));

        if self.current_level > self.max_level {
            self.max_level = self.current_level;
            self.max_hold_time = 0.0;
            self.decay_timer = 0.0;
        } else {
            self.max_hold_time += 1.0 / 60.0;
            if self.max_hold_time >= 3.0 {
                self.decay_timer += 1.0 / 60.0;
                let decay_rate = 0.006 + self.decay_timer * 0.001; // Décroissance progressive
                self.max_level = (self.max_level - decay_rate).max(recent_max);
                if self.max_level <= recent_max {
                    self.max_hold_time = 0.0;
                    self.decay_timer = 0.0;
                }
            }
        }

        // Mise à jour des animations
        self.animation_time += 0.8 + self.current_level * 1.5;
        self.background_pulse = 0.85 + 0.15 * (self.animation_time * 0.04).sin();
        self.update_sparkles();

        // Statistiques de débogage (optimisées)
        self.frame_count += 1;
        if self.frame_count % 60 == 0 {
            println!(
                "👏 [Applaudimètre] L:{:.3} M:{:.3} H:{:.1}s G:{:.2} S:{} N:{:.3}",
                self.current_level, self.max_level, self.max_hold_time,
                self.auto_gain, self.peak_sparkles.len(), self.noise_gate
            );
        }

        // Rendu
        frame.fill(0);
        self.render_bar(frame);
        self.render_sparkles(frame);
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

    fn name(&self) -> &'static str {
        "Applaudimetre"
    }

    fn description(&self) -> &'static str {
        "Mesureur d'applaudissements avec indicateur de pic et effets visuels"
    }

    fn reset(&mut self) {
        self.current_level = 0.0;
        self.max_level = 0.0;
        self.max_hold_time = 0.0;
        self.smoothed_level = 0.0;
        self.peak_history.fill(0.0);
        self.level_history.fill(0.0);
        self.peak_sparkles.clear();
        self.animation_time = 0.0;
        self.auto_gain = 1.0;
        self.background_pulse = 0.0;
        self.frame_count = 0;
        self.decay_timer = 0.0;
        self.noise_gate = 0.02;
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}
