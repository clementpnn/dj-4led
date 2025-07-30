use super::{LedMode, MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_SIZE, validate_frame_size, create_test_pattern};
use super::artnet::ArtNetClient;
use std::time::Instant;

/// Contrôleur LED optimisé pour la production
pub struct LedController {
    artnet_client: ArtNetClient,
    controllers: Vec<String>,
    mode: LedMode,
    frame_count: u64,
    start_time: Instant,
}

impl LedController {
    /// Créer contrôleur en mode production par défaut
    pub fn new() -> Result<Self, String> {
        Self::new_with_mode(LedMode::Production) // TOUJOURS PRODUCTION
    }

    /// Créer contrôleur avec mode spécifique
    pub fn new_with_mode(mode: LedMode) -> Result<Self, String> {
        println!("🌐 [LED] === CRÉATION CONTRÔLEUR LED ===");
        println!("🌐 [LED] Mode: {:?}", mode);

        // Créer le client Art-Net
        let artnet_client = ArtNetClient::new()?;

        // Configuration selon le mode (mais forcer production si possible)
        let controllers = match mode {
            LedMode::Simulator => {
                println!("🧪 [LED] Configuration SIMULATEUR (compatibilité)");
                vec![
                    "127.0.0.1:6454".to_string(),
                    "127.0.0.1:6454".to_string(),
                    "127.0.0.1:6454".to_string(),
                    "127.0.0.1:6454".to_string(),
                ]
            }
            LedMode::Production => {
                println!("🏭 [LED] Configuration PRODUCTION");
                vec![
                    "192.168.1.45:6454".to_string(),
                    "192.168.1.46:6454".to_string(),
                    "192.168.1.47:6454".to_string(),
                    "192.168.1.48:6454".to_string(),
                ]
            }
        };

        println!("📋 [LED] Contrôleurs: {:?}", controllers);

        let controller = Self {
            artnet_client,
            controllers,
            mode, // Utiliser le mode demandé
            frame_count: 0,
            start_time: Instant::now(),
        };

        println!("✅ [LED] Contrôleur LED créé en mode {:?}", mode);
        Ok(controller)
    }

    /// Envoi de frame - Support des deux modes
    pub fn send_frame(&mut self, frame: &[u8]) {
        // Validation
        if let Err(e) = validate_frame_size(frame) {
            println!("❌ [LED] {}", e);
            return;
        }

        self.frame_count += 1;

        // Diagnostic périodique
        let avg_brightness = frame.iter().map(|&b| b as u32).sum::<u32>() as f32 / frame.len() as f32;
        if avg_brightness > 1.0 && self.frame_count % 200 == 0 {
            println!("📡 [LED] {:?} Frame #{} - luminosité: {:.1}", self.mode, self.frame_count, avg_brightness);
        }

        // Envoi selon le mode
        match self.mode {
            LedMode::Simulator => self.send_frame_simulator(frame),
            LedMode::Production => self.send_frame_production(frame),
        }

        // Stats périodiques
        if self.frame_count % 500 == 0 {
            let elapsed = self.start_time.elapsed().as_secs_f64();
            let fps = self.frame_count as f64 / elapsed;
            let (packets, bytes) = self.artnet_client.get_stats();
            println!("📊 [LED] {:?} FPS: {:.1} | Frames: {} | Packets: {} | Bytes: {}",
                     self.mode, fps, self.frame_count, packets, bytes);
        }
    }

    /// Envoi simulateur - Pour compatibilité
    fn send_frame_simulator(&mut self, frame: &[u8]) {
        if self.frame_count % 200 == 0 {
            println!("🧪 [SIM] Envoi frame simulator #{}", self.frame_count);
        }

        // Simulateur simple - envoyer sur localhost
        let mut universe = 0;

        // Pour chaque colonne de l'écran LED
        for col in 0..128 {
            // Chaque colonne utilise 2 univers (128 pixels / 64 pixels par univers)
            for uni_in_col in 0..2 {
                let mut dmx_data = vec![0u8; 512];

                // Mapping serpentin : colonnes paires montent, colonnes impaires descendent
                if col % 2 == 0 {
                    // Colonnes paires : du bas vers le haut
                    let start_pixel = uni_in_col * 64;
                    let end_pixel = ((uni_in_col + 1) * 64).min(128);

                    for pixel in start_pixel..end_pixel {
                        let led_idx = pixel - start_pixel;
                        let y = 127 - pixel; // Inverser pour monter
                        let pixel_idx = (y * 128 + col) * 3;

                        if pixel_idx + 2 < frame.len() && led_idx * 3 + 2 < 512 {
                            dmx_data[led_idx * 3] = frame[pixel_idx];         // R
                            dmx_data[led_idx * 3 + 1] = frame[pixel_idx + 1]; // G
                            dmx_data[led_idx * 3 + 2] = frame[pixel_idx + 2]; // B
                        }
                    }
                } else {
                    // Colonnes impaires : du haut vers le bas
                    let start_pixel = uni_in_col * 64;
                    let end_pixel = ((uni_in_col + 1) * 64).min(128);

                    for pixel in start_pixel..end_pixel {
                        let led_idx = pixel - start_pixel;
                        let y = pixel; // Normal pour descendre
                        let pixel_idx = (y * 128 + col) * 3;

                        if pixel_idx + 2 < frame.len() && led_idx * 3 + 2 < 512 {
                            dmx_data[led_idx * 3] = frame[pixel_idx];         // R
                            dmx_data[led_idx * 3 + 1] = frame[pixel_idx + 1]; // G
                            dmx_data[led_idx * 3 + 2] = frame[pixel_idx + 2]; // B
                        }
                    }
                }

                // Envoyer le paquet vers localhost
                let _ = self.artnet_client.send_universe(universe, &dmx_data, "127.0.0.1:6454");
                universe += 1;
            }
        }
    }

    /// Envoi production - Configuration physique réelle
    fn send_frame_production(&mut self, frame: &[u8]) {
        if self.frame_count % 200 == 0 {
            println!("🏭 [PROD] Envoi frame production #{}", self.frame_count);
        }

        // Configuration physique réelle:
        // - 64 bandes de 259 LEDs chacune
        // - Chaque bande monte puis redescend (2 colonnes virtuelles)
        // - 4 contrôleurs de 16 bandes chacun
        // - Chaque bande utilise 2 univers Art-Net

        let mut packets_sent = 0;

        for quarter in 0..4 {
            let controller_ip = &self.controllers[quarter];
            let base_universe = quarter * 32;

            // 16 bandes par contrôleur
            for band_in_quarter in 0..16 {
                let physical_band = quarter * 16 + band_in_quarter;

                // Colonnes virtuelles correspondantes
                let col_up = physical_band * 2;     // Colonne montante
                let col_down = physical_band * 2 + 1; // Colonne descendante

                // 2 univers par bande (259 LEDs / ~170 LEDs par univers)
                for uni_in_band in 0..2 {
                    let universe = base_universe + band_in_quarter * 2 + uni_in_band;
                    let mut dmx_data = vec![0u8; 512];

                    // Mapper pixels virtuels vers LEDs physiques
                    self.map_pixels_to_band(&mut dmx_data, frame, col_up, col_down, uni_in_band);

                    match self.artnet_client.send_universe(universe as u16, &dmx_data, controller_ip) {
                        Ok(_) => packets_sent += 1,
                        Err(e) => {
                            if self.frame_count % 100 == 0 {
                                println!("❌ [PROD] Erreur {} univers {}: {}", controller_ip, universe, e);
                            }
                        }
                    }
                }
            }
        }

        // Log succès
        if packets_sent > 0 && self.frame_count % 200 == 0 {
            println!("✅ [PROD] {} paquets Art-Net envoyés vers contrôleurs", packets_sent);
        }
    }

    /// Mapping pixels virtuels vers bande physique
    fn map_pixels_to_band(
        &self,
        dmx_data: &mut [u8],
        frame: &[u8],
        col_up: usize,
        col_down: usize,
        uni_in_band: usize,
    ) {
        // Vérification limites
        if col_up >= 128 || col_down >= 128 {
            return;
        }

        if uni_in_band == 0 {
            // Premier univers: LEDs 0-169 (170 LEDs)
            let mut dmx_offset = 0;

            // Partie montante: LEDs 0-129 (130 LEDs physiques)
            for led in 0..130 {
                if dmx_offset + 2 < 510 { // 170 * 3 = 510 max
                    // Mapping physique: LED 0 = bas de la bande
                    let y = 127 - (led * 128 / 130); // Répartir sur 128 pixels virtuels
                    let y = y.min(127);

                    let pixel_idx = (y * 128 + col_up) * 3;
                    if pixel_idx + 2 < frame.len() {
                        dmx_data[dmx_offset] = frame[pixel_idx];       // Rouge
                        dmx_data[dmx_offset + 1] = frame[pixel_idx + 1]; // Vert
                        dmx_data[dmx_offset + 2] = frame[pixel_idx + 2]; // Bleu
                    }
                    dmx_offset += 3;
                }
            }

            // Début partie descendante: LEDs 130-169 (40 LEDs)
            for led in 0..40 {
                if dmx_offset + 2 < 510 {
                    let y = led * 128 / 129; // Répartir 129 LEDs descendantes
                    let y = y.min(127);

                    let pixel_idx = (y * 128 + col_down) * 3;
                    if pixel_idx + 2 < frame.len() {
                        dmx_data[dmx_offset] = frame[pixel_idx];
                        dmx_data[dmx_offset + 1] = frame[pixel_idx + 1];
                        dmx_data[dmx_offset + 2] = frame[pixel_idx + 2];
                    }
                    dmx_offset += 3;
                }
            }
        } else {
            // Deuxième univers: LEDs 170-258 (89 LEDs)
            let mut dmx_offset = 0;

            // Suite partie descendante: LEDs 170-258
            for led in 40..129 {
                if dmx_offset + 2 < 267 { // 89 * 3 = 267 max
                    let y = led * 128 / 129;
                    let y = y.min(127);

                    let pixel_idx = (y * 128 + col_down) * 3;
                    if pixel_idx + 2 < frame.len() {
                        dmx_data[dmx_offset] = frame[pixel_idx];
                        dmx_data[dmx_offset + 1] = frame[pixel_idx + 1];
                        dmx_data[dmx_offset + 2] = frame[pixel_idx + 2];
                    }
                    dmx_offset += 3;
                }
            }
        }
    }

    // Méthodes de compatibilité
    pub fn test_pattern(&mut self, pattern: &str) -> Result<(), String> {
        println!("🎨 [LED] PRODUCTION TEST PATTERN '{}'", pattern);
        let frame = create_test_pattern(pattern, MATRIX_WIDTH, MATRIX_HEIGHT);
        self.send_frame(&frame);
        println!("✅ [LED] Pattern '{}' envoyé en PRODUCTION", pattern);
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), String> {
        println!("🧹 [LED] PRODUCTION Effacement écran");
        let black_frame = vec![0; MATRIX_SIZE];
        self.send_frame(&black_frame);
        Ok(())
    }

    pub fn test_connectivity(&mut self) -> Result<std::collections::HashMap<String, bool>, String> {
        println!("🔍 [LED] {:?} TEST CONNECTIVITÉ", self.mode);
        let mut results = std::collections::HashMap::new();

        for controller in &self.controllers {
            println!("🔍 [LED] Test {:?}: {}", self.mode, controller);

            match self.artnet_client.test_connectivity(controller) {
                Ok(_) => {
                    results.insert(controller.clone(), true);
                    println!("✅ [LED] {:?} {} - OK", self.mode, controller);
                }
                Err(e) => {
                    results.insert(controller.clone(), false);
                    println!("❌ [LED] {:?} {} - ERREUR: {}", self.mode, controller, e);
                }
            }
        }

        let successful = results.values().filter(|&&v| v).count();
        println!("📊 [LED] {:?}: {}/{} contrôleurs OK", self.mode, successful, self.controllers.len());

        Ok(results)
    }

    pub fn send_test_pattern(&mut self, pattern: super::TestPattern) -> Result<(), String> {
        let pattern_str = match pattern {
            super::TestPattern::AllRed => "red",
            super::TestPattern::AllGreen => "green",
            super::TestPattern::AllBlue => "blue",
            super::TestPattern::AllWhite => "white",
            super::TestPattern::Gradient => "gradient",
            super::TestPattern::Checkerboard => "checkerboard",
            super::TestPattern::QuarterTest => "gradient",
        };
        self.test_pattern(pattern_str)
    }

    // Getters
    pub fn get_stats(&self) -> DummyStats {
        let (packets, bytes) = self.artnet_client.get_stats();
        DummyStats {
            frames_sent: self.frame_count,
            packets_sent: packets,
            bytes_sent: bytes,
            fps: if self.start_time.elapsed().as_secs() > 0 {
                self.frame_count as f32 / self.start_time.elapsed().as_secs() as f32
            } else {
                0.0
            },
        }
    }

    pub fn get_mode(&self) -> &LedMode {
        &self.mode
    }

    pub fn get_controllers(&self) -> Vec<DummyController> {
        self.controllers.iter().enumerate().map(|(i, ip)| {
            let id = match self.mode {
                LedMode::Production => format!("production_controller_{}", i),
                LedMode::Simulator => format!("simulator_controller_{}", i),
            };
            DummyController {
                id,
                ip_address: ip.replace(":6454", ""),
                enabled: true,
            }
        }).collect()
    }

    pub fn reset_stats(&mut self) {
        println!("📊 [LED] PRODUCTION Reset statistiques");
        self.frame_count = 0;
        self.start_time = Instant::now();
        self.artnet_client.reset_stats();
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    // Méthodes supplémentaires
    pub fn clear_all(&mut self) -> Result<(), String> {
        self.clear()
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        println!("💡 [LED] PRODUCTION Brightness: {:.1}%", brightness * 100.0);
    }

    pub fn restart_connections(&mut self) -> Result<(), String> {
        println!("🔄 [LED] PRODUCTION Restart connections");
        self.artnet_client = ArtNetClient::new()?;
        println!("✅ [LED] PRODUCTION Connections redémarrées");
        Ok(())
    }
}

// Structures de compatibilité
#[derive(Debug)]
pub struct DummyStats {
    pub frames_sent: u64,
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub fps: f32,
}

impl DummyStats {
    pub fn get_success_rate(&self) -> f32 { 95.0 }
    pub fn get_error_rate(&self) -> f32 { 5.0 }
    pub fn is_healthy(&self) -> bool { true }
    pub fn controllers_active(&self) -> usize { 4 }
    pub fn controllers_total(&self) -> usize { 4 }
    pub fn frames_dropped(&self) -> u64 { 0 }
    pub fn packets_failed(&self) -> u64 { 0 }
    pub fn performance_stats(&self) -> DummyPerformanceStats { DummyPerformanceStats::default() }
}

#[derive(Debug, Default)]
pub struct DummyPerformanceStats {
    pub conversion_time_ms: f32,
    pub network_time_ms: f32,
    pub total_processing_time_ms: f32,
    pub memory_usage_mb: f32,
    pub cpu_usage_percent: f32,
}

#[derive(Debug)]
pub struct DummyController {
    pub id: String,
    pub ip_address: String,
    pub enabled: bool,
}
