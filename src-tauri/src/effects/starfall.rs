// src-tauri/src/effects/starfall.rs
use super::*;
use std::f32::consts::PI;

pub struct Starfall {
    shooting_stars: Vec<ShootingStar>,
    animation_time: f32,
    spawn_timer: f32,
}

struct ShootingStar {
    // Position et mouvement
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,

    // Apparence
    brightness: f32,
    size: f32,
    color: (f32, f32, f32),
    temperature: f32,  // Gardé pour la couleur réaliste

    // Traînée
    trail_points: Vec<TrailPoint>,
    max_trail_length: usize,

    // Durée de vie
    age: f32,
    max_age: f32,

    // Effets visuels
    twinkle_phase: f32,
    twinkle_speed: f32,
    pulsation: f32,

    // Physics simple
    mass: f32,
    atmospheric_friction: f32,
}

#[derive(Clone)]
struct TrailPoint {
    x: f32,
    y: f32,
    intensity: f32,
    age: f32,
    temperature: f32,
}

#[derive(Clone)]
enum SpawnSide {
    TopLeft,
    TopRight,
    Top,
    Left,
    Right,
    Radial(f32),
}

impl ShootingStar {
    fn new(spawn_side: SpawnSide, sound_intensity: f32, stellar_temp: f32) -> Self {
        let (start_x, start_y, vel_x, vel_y) = match spawn_side {
            SpawnSide::TopLeft => {
                let x = -30.0 + rand() * 20.0;
                let y = -30.0 + rand() * 80.0;
                let vx = 2.0 + rand() * 3.0 + sound_intensity * 3.0;
                let vy = 1.5 + rand() * 2.5 + sound_intensity * 2.0;
                (x, y, vx, vy)
            },
            SpawnSide::TopRight => {
                let x = 138.0 + rand() * 20.0;
                let y = -30.0 + rand() * 80.0;
                let vx = -2.0 - rand() * 3.0 - sound_intensity * 3.0;
                let vy = 1.5 + rand() * 2.5 + sound_intensity * 2.0;
                (x, y, vx, vy)
            },
            SpawnSide::Top => {
                let x = 20.0 + rand() * 88.0;
                let y = -40.0 + rand() * 30.0;
                let vx = (rand() - 0.5) * 4.0;
                let vy = 3.0 + rand() * 3.0 + sound_intensity * 3.0;
                (x, y, vx, vy)
            },
            SpawnSide::Left => {
                let x = -40.0 + rand() * 30.0;
                let y = 20.0 + rand() * 88.0;
                let vx = 3.0 + rand() * 3.0 + sound_intensity * 3.0;
                let vy = (rand() - 0.5) * 4.0;
                (x, y, vx, vy)
            },
            SpawnSide::Right => {
                let x = 138.0 + rand() * 30.0;
                let y = 20.0 + rand() * 88.0;
                let vx = -3.0 - rand() * 3.0 - sound_intensity * 3.0;
                let vy = (rand() - 0.5) * 4.0;
                (x, y, vx, vy)
            },
            SpawnSide::Radial(angle) => {
                let radius = 160.0;
                let x = 64.0 + radius * angle.cos();
                let y = 64.0 + radius * angle.sin();
                let speed = 2.0 + sound_intensity * 4.0;
                let vx = -speed * angle.cos();
                let vy = -speed * angle.sin();
                (x, y, vx, vy)
            },
        };

        // Couleurs stellaires réalistes simplifiées
        let base_color = Self::get_stellar_color(stellar_temp);
        let mass = 0.5 + stellar_temp * 1.5 + sound_intensity;

        Self {
            x: start_x,
            y: start_y,
            velocity_x: vel_x,
            velocity_y: vel_y,
            brightness: 0.4 + rand() * 0.4 + sound_intensity * 0.4,
            size: 0.8 + rand() * 1.5 + sound_intensity * 1.2,
            color: base_color,
            temperature: stellar_temp,
            trail_points: Vec::new(),
            max_trail_length: (10 + (mass * 15.0) as usize).min(35),
            age: 0.0,
            max_age: 80.0 + rand() * 60.0 + mass * 20.0,
            twinkle_phase: rand() * 2.0 * PI,
            twinkle_speed: 0.05 + rand() * 0.15,
            pulsation: rand() * 2.0 * PI,
            mass,
            atmospheric_friction: 0.995 + rand() * 0.004,
        }
    }

    fn get_stellar_color(temperature: f32) -> (f32, f32, f32) {
        match temperature {
            t if t > 0.8 => (0.6, 0.7, 1.0),      // Bleu
            t if t > 0.6 => (0.9, 0.9, 1.0),      // Blanc
            t if t > 0.4 => (1.0, 1.0, 0.7),      // Jaune
            t if t > 0.2 => (1.0, 0.7, 0.4),      // Orange
            _ => (1.0, 0.4, 0.2),                  // Rouge
        }
    }

    fn update(&mut self, time: f32) {
        self.age += 1.0;
        self.twinkle_phase += self.twinkle_speed;
        self.pulsation += 0.03;

        // Ajouter le point actuel à la traînée
        self.trail_points.push(TrailPoint {
            x: self.x,
            y: self.y,
            intensity: self.brightness,
            age: 0.0,
            temperature: self.temperature,
        });

        // Limiter la longueur de la traînée
        if self.trail_points.len() > self.max_trail_length {
            self.trail_points.remove(0);
        }

        // Vieillir les points de la traînée
        for point in &mut self.trail_points {
            point.age += 1.0;
            point.intensity *= 0.93;
            point.temperature *= 0.98;
        }

        // Mouvement avec physics basique
        self.x += self.velocity_x;
        self.y += self.velocity_y;

        // Friction atmosphérique
        self.velocity_x *= self.atmospheric_friction;
        self.velocity_y *= self.atmospheric_friction;

        // Gravité légère
        self.velocity_y += 0.02;

        // Turbulence légère
        let turbulence = 0.05 / self.mass.max(0.1);
        self.velocity_x += (time * 0.1 + self.twinkle_phase).sin() * turbulence;
        self.velocity_y += (time * 0.08 + self.twinkle_phase * 1.3).cos() * turbulence;

        // Diminution progressive de la luminosité
        if self.age > self.max_age * 0.7 {
            let fade_factor = 1.0 - (self.age - self.max_age * 0.7) / (self.max_age * 0.3);
            self.brightness *= fade_factor.max(0.0);
        }
    }

    fn is_alive(&self) -> bool {
        self.age < self.max_age &&
        self.brightness > 0.005 &&
        self.x > -60.0 && self.x < 188.0 &&
        self.y > -60.0 && self.y < 188.0
    }

    fn get_twinkle_factor(&self) -> f32 {
        0.7 + 0.3 * (self.twinkle_phase.sin() * 0.5 + 0.5)
    }

    fn get_brightness_modifier(&self) -> f32 {
        let life_progress = self.age / self.max_age;
        if life_progress < 0.1 {
            // Formation
            0.5 + life_progress * 5.0
        } else if life_progress > 0.8 {
            // Déclin
            1.0 - (life_progress - 0.8) * 2.5
        } else {
            // Stable
            1.0 + 0.2 * self.pulsation.sin()
        }
    }
}

impl Starfall {
    pub fn new() -> Self {
        Self {
            shooting_stars: Vec::new(),
            animation_time: 0.0,
            spawn_timer: 0.0,
        }
    }

    fn get_star_color(&self, base_color: (f32, f32, f32), brightness: f32, temperature: f32) -> (f32, f32, f32) {
        let color_config = get_color_config();

        match color_config.mode.as_str() {
            "rainbow" => {
                let hue = (self.animation_time * 0.003 + temperature) % 1.0;
                hsv_to_rgb(hue, 0.8, brightness)
            },
            "fire" => {
                if brightness < 0.5 {
                    (brightness * 2.0, brightness * 0.3, 0.0)
                } else {
                    (brightness, brightness * 0.8, brightness * (brightness - 0.5) * 2.0)
                }
            },
            "ocean" => {
                (brightness * 0.2, brightness * 0.7, brightness)
            },
            "sunset" => {
                let warm = temperature.powf(0.5);
                (brightness * warm, brightness * warm * 0.8, brightness * (1.0 - warm) * 0.9)
            },
            "custom" => {
                let (r, g, b) = color_config.custom_color;
                (r * brightness, g * brightness, b * brightness)
            },
            _ => {
                // Couleurs stellaires réalistes
                (base_color.0 * brightness, base_color.1 * brightness, base_color.2 * brightness)
            }
        }
    }

    fn spawn_shooting_star(&mut self, sound_intensity: f32) {
        let spawn_side = match (rand() * 6.0) as usize {
            0 => SpawnSide::TopLeft,
            1 => SpawnSide::TopRight,
            2 => SpawnSide::Top,
            3 => SpawnSide::Left,
            4 => SpawnSide::Right,
            _ => SpawnSide::Radial(rand() * 2.0 * PI),
        };

        let stellar_temp = rand();
        let star = ShootingStar::new(spawn_side, sound_intensity, stellar_temp);
        self.shooting_stars.push(star);
    }

    fn create_meteor_shower(&mut self, intensity: f32) {
        let meteor_count = (intensity * 5.0) as usize + 2;
        let center_angle = rand() * 2.0 * PI;

        for i in 0..meteor_count {
            let angle_offset = (i as f32 / meteor_count as f32) * PI * 0.5;
            let spawn_angle = center_angle + angle_offset;

            let spawn_side = SpawnSide::Radial(spawn_angle);
            let mut star = ShootingStar::new(spawn_side, intensity, 0.6 + rand() * 0.4);

            // Météores plus impressionnants
            star.max_age *= 1.3;
            star.brightness *= 1.2;
            star.max_trail_length = (star.max_trail_length as f32 * 1.5) as usize;

            self.shooting_stars.push(star);
        }
    }
}

impl Effect for Starfall {
    fn render(&mut self, spectrum: &[f32], frame: &mut [u8]) {
        // Analyser l'intensité sonore
        let bass_energy = (spectrum[..8].iter().sum::<f32>() / 8.0) * 4.0;
        let mid_energy = (spectrum[8..24].iter().sum::<f32>() / 16.0) * 3.0;
        let high_energy = (spectrum[24..].iter().sum::<f32>() / 40.0) * 2.0;

        let total_energy = (bass_energy * 0.3 + mid_energy * 0.4 + high_energy * 0.3)
            .clamp(0.0, 1.0);

        self.animation_time += 1.0 + total_energy;
        self.spawn_timer += 1.0;

        // Debug simplifié
        static mut FRAME_COUNT: u64 = 0;
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 60 == 0 && total_energy > 0.1 {
                println!("⭐ [Starfall] Energy: {:.3}, Stars: {}", total_energy, self.shooting_stars.len());
            }
        }

        // Gestion du spawn
        if total_energy > 0.1 {
            let spawn_interval = (50.0 / (1.0 + total_energy * 3.0)).max(8.0);

            if self.spawn_timer >= spawn_interval {
                self.spawn_timer = 0.0;

                if total_energy > 0.8 && rand() < 0.1 {
                    // Pluie de météores pour les pics intenses
                    self.create_meteor_shower(total_energy);
                } else if self.shooting_stars.len() < 20 {
                    // Spawn normal
                    let star_count = if total_energy > 0.6 { 2 } else { 1 };
                    for _ in 0..star_count {
                        self.spawn_shooting_star(total_energy);
                    }
                }
            }
        }

        // Mettre à jour toutes les étoiles
        for star in &mut self.shooting_stars {
            star.update(self.animation_time);
        }

        // Supprimer les étoiles mortes
        self.shooting_stars.retain(|star| star.is_alive());

        // Effacer l'écran
        frame.fill(0);

        // Dessiner toutes les étoiles filantes
        for star in &self.shooting_stars {
            let twinkle = star.get_twinkle_factor();
            let brightness_mod = star.get_brightness_modifier();

            // Dessiner la traînée
            for (i, trail_point) in star.trail_points.iter().enumerate() {
                if trail_point.intensity > 0.03 {
                    let trail_age_factor = 1.0 - (i as f32 / star.trail_points.len() as f32);
                    let trail_brightness = trail_point.intensity * trail_age_factor * 0.6;

                    let px = trail_point.x as i32;
                    let py = trail_point.y as i32;

                    if px >= 0 && px < 128 && py >= 0 && py < 128 {
                        let idx = (py as usize * 128 + px as usize) * 3;
                        if idx + 2 < frame.len() {
                            let (r, g, b) = self.get_star_color(
                                star.color,
                                trail_brightness,
                                trail_point.temperature
                            );

                            frame[idx] = ((r * 255.0) as u8).saturating_add(frame[idx]);
                            frame[idx + 1] = ((g * 255.0) as u8).saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((b * 255.0) as u8).saturating_add(frame[idx + 2]);
                        }
                    }
                }
            }

            // Dessiner l'étoile principale
            let star_brightness = star.brightness * twinkle * brightness_mod;
            let star_size = star.size * brightness_mod.sqrt();

            let center_x = star.x as i32;
            let center_y = star.y as i32;

            // Dessiner avec anti-aliasing
            let render_size = (star_size as i32 + 1).max(1);
            for dy in -render_size..=render_size {
                for dx in -render_size..=render_size {
                    let x = center_x + dx;
                    let y = center_y + dy;

                    if x >= 0 && x < 128 && y >= 0 && y < 128 {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        let mut intensity = 0.0;

                        // Cœur brillant
                        if distance <= star_size {
                            let core_falloff = (1.0 - distance / star_size.max(0.1)).max(0.0);
                            intensity = star_brightness * core_falloff;
                        }
                        // Halo externe
                        else if distance <= star_size + 1.0 {
                            let halo_falloff = (1.0 - (distance - star_size)).max(0.0);
                            intensity = star_brightness * halo_falloff * 0.3;
                        }

                        if intensity > 0.01 {
                            let idx = (y as usize * 128 + x as usize) * 3;
                            if idx + 2 < frame.len() {
                                let (r, g, b) = self.get_star_color(star.color, intensity, star.temperature);

                                frame[idx] = ((r * 255.0) as u8).saturating_add(frame[idx]);
                                frame[idx + 1] = ((g * 255.0) as u8).saturating_add(frame[idx + 1]);
                                frame[idx + 2] = ((b * 255.0) as u8).saturating_add(frame[idx + 2]);
                            }
                        }
                    }
                }
            }

            // Rayons de diffraction pour les étoiles brillantes
            if star_brightness > 0.6 {
                let ray_length = (star_size * 1.5) as i32;
                let ray_brightness = star_brightness * 0.3;

                for offset in -ray_length..=ray_length {
                    if offset == 0 { continue; }

                    let ray_intensity = ray_brightness * (1.0 - offset.abs() as f32 / ray_length as f32);

                    // Rayon horizontal
                    let hx = center_x + offset;
                    let hy = center_y;
                    if hx >= 0 && hx < 128 && hy >= 0 && hy < 128 {
                        let idx = (hy as usize * 128 + hx as usize) * 3;
                        if idx + 2 < frame.len() {
                            let (r, g, b) = self.get_star_color(star.color, ray_intensity, star.temperature);
                            frame[idx] = ((r * 180.0) as u8).saturating_add(frame[idx]);
                            frame[idx + 1] = ((g * 180.0) as u8).saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((b * 180.0) as u8).saturating_add(frame[idx + 2]);
                        }
                    }

                    // Rayon vertical
                    let vx = center_x;
                    let vy = center_y + offset;
                    if vx >= 0 && vx < 128 && vy >= 0 && vy < 128 {
                        let idx = (vy as usize * 128 + vx as usize) * 3;
                        if idx + 2 < frame.len() {
                            let (r, g, b) = self.get_star_color(star.color, ray_intensity, star.temperature);
                            frame[idx] = ((r * 180.0) as u8).saturating_add(frame[idx]);
                            frame[idx + 1] = ((g * 180.0) as u8).saturating_add(frame[idx + 1]);
                            frame[idx + 2] = ((b * 180.0) as u8).saturating_add(frame[idx + 2]);
                        }
                    }
                }
            }
        }
    }

    fn set_color_mode(&mut self, mode: &str) {
        println!("⭐ [Starfall] Color mode set to '{}'", mode);
    }

    fn set_custom_color(&mut self, r: f32, g: f32, b: f32) {
        println!("⭐ [Starfall] Custom color set to ({:.2}, {:.2}, {:.2})", r, g, b);
    }

    fn name(&self) -> &'static str {
        "Starfall"
    }

    fn description(&self) -> &'static str {
        "Beautiful shooting stars with realistic physics and stellar colors"
    }

    fn reset(&mut self) {
        self.shooting_stars.clear();
        self.animation_time = 0.0;
        self.spawn_timer = 0.0;
        println!("⭐ [Starfall] Effect reset");
    }

    fn supports_transitions(&self) -> bool {
        true
    }
}
