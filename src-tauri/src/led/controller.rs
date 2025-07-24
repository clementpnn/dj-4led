use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::led::artnet::{ArtNetClient, utils};
use super::{ControllerConfig, LedMode, LedStats, default_production_controllers, default_simulator_controllers};

/// Dimensions de la matrice LED
pub const MATRIX_WIDTH: usize = 128;
pub const MATRIX_HEIGHT: usize = 128;
pub const MATRIX_SIZE: usize = MATRIX_WIDTH * MATRIX_HEIGHT * 3; // RGB

/// Configuration d'un mapping physique LED
#[derive(Debug, Clone)]
struct LedMapping {
    controller_ip: String,
    universe: u16,
    dmx_channel: u16,
}

/// Contrôleur principal pour l'envoi de données LED
pub struct LedController {
    mode: LedMode,
    clients: HashMap<String, ArtNetClient>,
    controllers: Vec<ControllerConfig>,
    led_mappings: HashMap<(usize, usize), LedMapping>, // (x, y) -> mapping
    stats: LedStats,
    last_frame_time: Instant,
    gamma_correction: f32,
    brightness: f32,
}

impl LedController {
    /// Crée un nouveau contrôleur LED
    pub fn new() -> Result<Self> {
        Self::new_with_mode(LedMode::default())
    }

    /// Crée un nouveau contrôleur LED avec un mode spécifique
    pub fn new_with_mode(mode: LedMode) -> Result<Self> {
        println!("🌐 [LED] Init contrôleur mode {:?}", mode);

        let controllers = match mode {
            LedMode::Simulator => default_simulator_controllers(),
            LedMode::Production => default_production_controllers(),
        };

        let mut controller = Self {
            mode,
            clients: HashMap::new(),
            controllers: controllers.clone(),
            led_mappings: HashMap::new(),
            stats: LedStats::default(),
            last_frame_time: Instant::now(),
            gamma_correction: 2.2,
            brightness: 1.0,
        };

        controller.init_clients()?;
        controller.build_led_mappings();

        println!("✅ [LED] Contrôleur init avec {} contrôleurs", controller.controllers.len());

        Ok(controller)
    }

    /// Initialise les clients Art-Net pour chaque contrôleur
    fn init_clients(&mut self) -> Result<()> {
        for controller in &self.controllers {
            let client = ArtNetClient::new(&controller.ip_address)?;
            self.clients.insert(controller.ip_address.clone(), client);
            println!("🌐 [ART-NET] Client créé pour {}", controller.ip_address);
        }

        self.stats.controllers_active = self.clients.len();
        println!("✅ [LED] {} clients Art-Net init pour {:?}", self.clients.len(), self.mode);

        Ok(())
    }

    /// Construit les mappings entre positions de pixels et contrôleurs
    fn build_led_mappings(&mut self) {
        self.led_mappings.clear();

        match self.mode {
            LedMode::Simulator => self.build_simulator_mappings(),
            LedMode::Production => self.build_production_mappings(),
        }

        self.stats.universes_active = self.led_mappings.len();
        println!("✅ [LED] {} mappings construits", self.led_mappings.len());
    }

    /// Construit les mappings pour le mode simulateur (simplifié comme l'ancien code)
    fn build_simulator_mappings(&mut self) {
        let mut universe = 0;

        for col in 0..MATRIX_WIDTH {
            for uni_in_col in 0..2 {
                let controller_ip = "127.0.0.1".to_string();

                for pixel in 0..64 {
                    let y = if col % 2 == 0 {
                        127 - (uni_in_col * 64 + pixel)
                    } else {
                        uni_in_col * 64 + pixel
                    };

                    if y < MATRIX_HEIGHT {
                        let dmx_channel = pixel * 3;

                        self.led_mappings.insert(
                            (col, y),
                            LedMapping {
                                controller_ip: controller_ip.clone(),
                                universe,
                                dmx_channel: dmx_channel as u16,
                            }
                        );
                    }
                }

                universe += 1;
            }
        }
    }

    /// Construit les mappings pour le mode production
    fn build_production_mappings(&mut self) {
        let quarters = [
            (0, 31, "192.168.1.45", 0),
            (32, 63, "192.168.1.46", 32),
            (64, 95, "192.168.1.47", 64),
            (96, 127, "192.168.1.48", 96),
        ];

        for (col_start, col_end, ip, universe_base) in quarters {
            for col in col_start..=col_end {
                let band = (col - col_start) / 2;

                for uni_in_band in 0..2 {
                    let universe = universe_base + band * 2 + uni_in_band;
                    self.map_production_band(ip, universe as u16, col, uni_in_band);
                }
            }
        }
    }

    /// Mappe une bande de LEDs pour le mode production
    fn map_production_band(&mut self, ip: &str, universe: u16, col: usize, uni_in_band: usize) {
        if uni_in_band == 0 {
            for led in 0..130 {
                if led < 128 {
                    let y = 127 - (led * 128 / 130);
                    let dmx_channel = led * 3;

                    self.led_mappings.insert(
                        (col, y),
                        LedMapping {
                            controller_ip: ip.to_string(),
                            universe,
                            dmx_channel: dmx_channel as u16,
                        }
                    );
                }
            }

            let next_col = col + 1;
            if next_col < MATRIX_WIDTH {
                for led in 0..40 {
                    let y = led * 128 / 129;
                    let dmx_channel = (130 + led) * 3;

                    if dmx_channel < 512 {
                        self.led_mappings.insert(
                            (next_col, y),
                            LedMapping {
                                controller_ip: ip.to_string(),
                                universe,
                                dmx_channel: dmx_channel as u16,
                            }
                        );
                    }
                }
            }
        } else {
            let next_col = col + 1;
            if next_col < MATRIX_WIDTH {
                for led in 40..129 {
                    let y = led * 128 / 129;
                    let dmx_channel = (led - 40) * 3;

                    if dmx_channel < 512 {
                        self.led_mappings.insert(
                            (next_col, y),
                            LedMapping {
                                controller_ip: ip.to_string(),
                                universe,
                                dmx_channel: dmx_channel as u16,
                            }
                        );
                    }
                }
            }
        }
    }

    /// Envoie une frame complète à tous les contrôleurs
    pub fn send_frame(&mut self, frame: &[u8]) -> Result<()> {
        if frame.len() != MATRIX_SIZE {
            anyhow::bail!("Invalid frame size: expected {}, got {}", MATRIX_SIZE, frame.len());
        }

        // Log périodique de l'activité
        if self.stats.frames_sent % 200 == 0 {
            let active_pixels = (0..frame.len()).step_by(3)
                .filter(|&i| frame[i] > 0 || frame[i+1] > 0 || frame[i+2] > 0)
                .count();

            if active_pixels > 0 {
                println!("🖼️ [LED] Frame #{}: {} pixels actifs", self.stats.frames_sent, active_pixels);
            }
        }

        // Préparer les données par univers
        let mut universe_data: HashMap<(String, u16), Vec<u8>> = HashMap::new();

        // Initialiser tous les univers avec des zéros
        for controller in &self.controllers {
            for universe in controller.start_universe..(controller.start_universe + controller.universe_count) {
                universe_data.insert(
                    (controller.ip_address.clone(), universe),
                    vec![0; 512]
                );
            }
        }

        // Remplir les données selon les mappings
        for y in 0..MATRIX_HEIGHT {
            for x in 0..MATRIX_WIDTH {
                if let Some(mapping) = self.led_mappings.get(&(x, y)) {
                    let pixel_idx = (y * MATRIX_WIDTH + x) * 3;

                    if pixel_idx + 2 < frame.len() {
                        let mut r = frame[pixel_idx];
                        let mut g = frame[pixel_idx + 1];
                        let mut b = frame[pixel_idx + 2];

                        // Appliquer les corrections
                        (r, g, b) = utils::apply_brightness_rgb(r, g, b, self.brightness);
                        (r, g, b) = utils::apply_gamma_rgb(r, g, b, self.gamma_correction);

                        let key = (mapping.controller_ip.clone(), mapping.universe);
                        if let Some(dmx_data) = universe_data.get_mut(&key) {
                            let ch = mapping.dmx_channel as usize;
                            if ch + 2 < dmx_data.len() {
                                dmx_data[ch] = r;
                                dmx_data[ch + 1] = g;
                                dmx_data[ch + 2] = b;
                            }
                        }
                    }
                }
            }
        }

        // Envoyer les données à tous les contrôleurs
        let mut total_bytes = 0;
        let mut packets_sent = 0;
        let mut errors = 0;

        for ((ip, universe), dmx_data) in &universe_data {
            if let Some(client) = self.clients.get_mut(ip) {
                match client.send_universe(*universe, dmx_data) {
                    Ok(bytes) => {
                        total_bytes += bytes;
                        packets_sent += 1;
                    }
                    Err(e) => {
                        errors += 1;
                        if errors % 100 == 1 {
                            eprintln!("❌ [LED] Error {}:{} - {}", ip, universe, e);
                        }
                    }
                }
            } else {
                eprintln!("❌ [LED] No client found for IP: {}", ip);
            }
        }

        // Mettre à jour les statistiques
        let frame_time = Instant::now().duration_since(self.last_frame_time);
        self.update_stats(total_bytes, packets_sent, frame_time);

        if errors > 0 && self.stats.frames_sent % 500 == 0 {
            println!("⚠️ [LED] Frame #{}: {} erreurs sur {} paquets",
                     self.stats.frames_sent, errors, packets_sent + errors);
        }

        Ok(())
    }

    /// Met à jour les statistiques de performance
    fn update_stats(&mut self, bytes_sent: usize, packets_sent: usize, frame_time: Duration) {
        self.stats.frames_sent += 1;
        self.stats.packets_sent += packets_sent as u64;
        self.stats.bytes_sent += bytes_sent as u64;

        let frame_time_ms = frame_time.as_millis() as f32;
        self.stats.avg_frame_time_ms = (self.stats.avg_frame_time_ms + frame_time_ms) / 2.0;

        let time_since_last = self.last_frame_time.elapsed();
        if time_since_last.as_millis() > 0 {
            self.stats.fps = 1000.0 / time_since_last.as_millis() as f32;
        }
        self.last_frame_time = Instant::now();

        // Log moins fréquent mais plus informatif
        if self.stats.frames_sent % 300 == 0 {
            println!("📊 [LED] Frames: {}, FPS: {:.1}, Controllers: {}, Bytes/s: {:.1}k",
                     self.stats.frames_sent,
                     self.stats.fps,
                     self.stats.controllers_active,
                     (bytes_sent as f32) / 1024.0);
        }
    }

    /// Obtient les statistiques actuelles
    pub fn get_stats(&self) -> &LedStats {
        &self.stats
    }

    /// Définit la correction gamma
    pub fn set_gamma_correction(&mut self, gamma: f32) {
        self.gamma_correction = gamma.clamp(0.1, 5.0);
        println!("🎨 [LED] Gamma: {:.2}", self.gamma_correction);
    }

    /// Définit la luminosité globale
    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness.clamp(0.0, 1.0);
        println!("💡 [LED] Brightness: {:.1}%", self.brightness * 100.0);
    }

    /// Obtient le mode actuel
    pub fn get_mode(&self) -> LedMode {
        self.mode
    }

    /// Obtient la liste des contrôleurs
    pub fn get_controllers(&self) -> &[ControllerConfig] {
        &self.controllers
    }

    /// Test de connectivité avec tous les contrôleurs
    pub fn test_connectivity(&mut self) -> Result<HashMap<String, bool>> {
        let mut results = HashMap::new();

        println!("🔍 [LED] Test connectivité {} contrôleurs", self.controllers.len());

        for controller in &self.controllers {
            let test_data = vec![0; 512];

            if let Some(client) = self.clients.get_mut(&controller.ip_address) {
                match client.send_universe(0, &test_data) {
                    Ok(_) => {
                        results.insert(controller.ip_address.clone(), true);
                        println!("✅ [LED] {} - OK", controller.ip_address);
                    }
                    Err(e) => {
                        results.insert(controller.ip_address.clone(), false);
                        println!("❌ [LED] {} - Error: {}", controller.ip_address, e);
                    }
                }
            } else {
                results.insert(controller.ip_address.clone(), false);
                println!("❌ [LED] {} - No client", controller.ip_address);
            }
        }

        let successful = results.values().filter(|&&v| v).count();
        println!("📊 [LED] Test: {}/{} OK", successful, self.controllers.len());

        Ok(results)
    }

    /// Envoie un pattern de test sur tous les contrôleurs
    pub fn send_test_pattern(&mut self, pattern: TestPattern) -> Result<()> {
        println!("🎨 [LED] Test pattern: {:?}", pattern);
        let frame = self.generate_test_frame(pattern);
        self.send_frame(&frame)
    }

    /// Génère une frame de test selon le pattern demandé
    fn generate_test_frame(&self, pattern: TestPattern) -> Vec<u8> {
        let mut frame = vec![0; MATRIX_SIZE];

        match pattern {
            TestPattern::AllRed => {
                for i in (0..MATRIX_SIZE).step_by(3) {
                    frame[i] = 255;
                }
            }
            TestPattern::AllGreen => {
                for i in (0..MATRIX_SIZE).step_by(3) {
                    frame[i + 1] = 255;
                }
            }
            TestPattern::AllBlue => {
                for i in (0..MATRIX_SIZE).step_by(3) {
                    frame[i + 2] = 255;
                }
            }
            TestPattern::AllWhite => {
                for i in (0..MATRIX_SIZE).step_by(3) {
                    frame[i] = 255;
                    frame[i + 1] = 255;
                    frame[i + 2] = 255;
                }
            }
            TestPattern::Gradient => {
                for y in 0..MATRIX_HEIGHT {
                    for x in 0..MATRIX_WIDTH {
                        let idx = (y * MATRIX_WIDTH + x) * 3;
                        let intensity = (x * 255 / MATRIX_WIDTH) as u8;
                        frame[idx] = intensity;
                        frame[idx + 1] = intensity;
                        frame[idx + 2] = intensity;
                    }
                }
            }
            TestPattern::Checkerboard => {
                for y in 0..MATRIX_HEIGHT {
                    for x in 0..MATRIX_WIDTH {
                        let idx = (y * MATRIX_WIDTH + x) * 3;
                        let is_white = (x / 8 + y / 8) % 2 == 0;
                        let value = if is_white { 255 } else { 0 };
                        frame[idx] = value;
                        frame[idx + 1] = value;
                        frame[idx + 2] = value;
                    }
                }
            }
            TestPattern::QuarterTest => {
                for y in 0..MATRIX_HEIGHT {
                    for x in 0..MATRIX_WIDTH {
                        let idx = (y * MATRIX_WIDTH + x) * 3;

                        let (r, g, b) = match (x < MATRIX_WIDTH / 2, y < MATRIX_HEIGHT / 2) {
                            (true, true) => (255, 0, 0),
                            (false, true) => (0, 255, 0),
                            (true, false) => (0, 0, 255),
                            (false, false) => (255, 255, 0),
                        };

                        frame[idx] = r;
                        frame[idx + 1] = g;
                        frame[idx + 2] = b;
                    }
                }
            }
        }

        frame
    }

    /// Éteint toutes les LEDs
    pub fn clear_all(&mut self) -> Result<()> {
        println!("🔳 [LED] Clear all");
        let black_frame = vec![0; MATRIX_SIZE];
        self.send_frame(&black_frame)
    }

    /// Redémarre la connexion avec tous les contrôleurs
    pub fn restart_connections(&mut self) -> Result<()> {
        println!("🔄 [LED] Restart connections");
        self.clients.clear();

        // Petit délai pour laisser les sockets se fermer
        std::thread::sleep(Duration::from_millis(100));

        self.init_clients()?;
        println!("✅ [LED] Connections restarted");
        Ok(())
    }
}

/// Patterns de test disponibles
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
    /// Obtient tous les patterns de test disponibles
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

    /// Obtient le nom du pattern
    pub fn name(&self) -> &'static str {
        match self {
            TestPattern::AllRed => "Tout Rouge",
            TestPattern::AllGreen => "Tout Vert",
            TestPattern::AllBlue => "Tout Bleu",
            TestPattern::AllWhite => "Tout Blanc",
            TestPattern::Gradient => "Dégradé",
            TestPattern::Checkerboard => "Damier",
            TestPattern::QuarterTest => "Test par Quarts",
        }
    }

    /// Obtient la description du pattern
    pub fn description(&self) -> &'static str {
        match self {
            TestPattern::AllRed => "Toutes les LEDs en rouge",
            TestPattern::AllGreen => "Toutes les LEDs en vert",
            TestPattern::AllBlue => "Toutes les LEDs en bleu",
            TestPattern::AllWhite => "Toutes les LEDs en blanc",
            TestPattern::Gradient => "Dégradé horizontal de gauche à droite",
            TestPattern::Checkerboard => "Pattern damier noir et blanc",
            TestPattern::QuarterTest => "Chaque quart en couleur différente",
        }
    }
}
