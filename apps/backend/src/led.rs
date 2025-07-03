use anyhow::Result;
use std::net::UdpSocket;

pub enum LedMode {
    Simulator,
    Production,
}

pub struct LedController {
    socket: UdpSocket,
    controllers: Vec<String>,
    mode: LedMode,
}

impl LedController {
    pub fn new() -> Result<Self> {
        Self::new_with_mode(LedMode::Simulator)
    }

    pub fn new_with_mode(mode: LedMode) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        // Adresses des contrôleurs LED
        let controllers = match mode {
            LedMode::Simulator => vec![
                "127.0.0.1:6454".to_string(),
                "127.0.0.1:6454".to_string(),
                "127.0.0.1:6454".to_string(),
                "127.0.0.1:6454".to_string(),
            ],
            LedMode::Production => vec![
                "192.168.1.45:6454".to_string(),
                "192.168.1.46:6454".to_string(),
                "192.168.1.47:6454".to_string(),
                "192.168.1.48:6454".to_string(),
            ],
        };

        Ok(Self {
            socket,
            controllers,
            mode,
        })
    }

    pub fn send_frame(&mut self, frame: &[u8]) {
        // Calculer la luminosité moyenne pour vérifier si on envoie bien des données
        let avg_brightness =
            frame.iter().map(|&b| b as u32).sum::<u32>() as f32 / frame.len() as f32;
        if avg_brightness > 1.0 {
            println!("📡 Sending frame - avg brightness: {:.1}", avg_brightness);
        }

        match self.mode {
            LedMode::Simulator => self.send_frame_simulator(frame),
            LedMode::Production => self.send_frame_production(frame),
        }
    }

    fn send_frame_simulator(&mut self, frame: &[u8]) {
        // Le simulateur s'attend à recevoir 256 univers (2 par colonne, 128 colonnes)
        let mut universe = 0;

        // Pour chaque colonne de l'écran LED
        for col in 0..128 {
            // Chaque colonne utilise 2 univers (128 pixels / 170 pixels par univers)
            for uni_in_col in 0..2 {
                let mut artnet_packet = vec![
                    b'A',
                    b'r',
                    b't',
                    b'-',
                    b'N',
                    b'e',
                    b't',
                    0, // ID
                    0x00,
                    0x50, // OpCode (OpOutput)
                    0,
                    14, // Protocol version
                    0,  // Sequence
                    0,  // Physical
                    (universe & 0xFF) as u8,
                    (universe >> 8) as u8, // Universe
                    0x02,
                    0x00, // Length (512)
                ];

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
                            dmx_data[led_idx * 3] = frame[pixel_idx]; // R
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
                            dmx_data[led_idx * 3] = frame[pixel_idx]; // R
                            dmx_data[led_idx * 3 + 1] = frame[pixel_idx + 1]; // G
                            dmx_data[led_idx * 3 + 2] = frame[pixel_idx + 2]; // B
                        }
                    }
                }

                artnet_packet.extend_from_slice(&dmx_data);

                // Envoyer le paquet
                let _ = self.socket.send_to(&artnet_packet, "127.0.0.1:6454");

                universe += 1;
            }
        }
    }

    fn send_frame_production(&mut self, frame: &[u8]) {
        // L'écran physique a 64 bandes de 259 LEDs chacune
        // Chaque bande monte puis redescend, formant 2 colonnes
        // Donc 64 bandes = 128 colonnes au total
        // Organisées en 4 contrôleurs de 16 bandes chacun

        let mut packets_sent = 0;

        for quarter in 0..4 {
            let controller_ip = &self.controllers[quarter];
            let base_universe = quarter * 32;

            // Chaque quartier a 16 bandes physiques
            for band_in_quarter in 0..16 {
                let physical_band = quarter * 16 + band_in_quarter;

                // Colonnes correspondantes dans l'écran virtuel
                let col_up = physical_band * 2; // Colonne montante
                let col_down = physical_band * 2 + 1; // Colonne descendante

                // Chaque bande physique utilise 2 univers (259 LEDs / 170 par univers)
                for uni_in_band in 0..2 {
                    let universe = base_universe + band_in_quarter * 2 + uni_in_band;
                    let mut artnet_packet = self.create_artnet_header(universe);
                    let mut dmx_data = vec![0u8; 512];

                    // Mapper les pixels de l'écran vers les LEDs physiques
                    self.map_pixels_to_band(&mut dmx_data, frame, col_up, col_down, uni_in_band);

                    artnet_packet.extend_from_slice(&dmx_data);
                    if let Err(e) = self.socket.send_to(&artnet_packet, controller_ip) {
                        println!("❌ Error sending to {}: {}", controller_ip, e);
                    } else {
                        packets_sent += 1;
                    }
                }
            }
        }

        if packets_sent > 0 && packets_sent % 64 == 0 {
            println!("✅ Sent {} ArtNet packets", packets_sent);
        }
    }

    fn create_artnet_header(&self, universe: usize) -> Vec<u8> {
        vec![
            b'A',
            b'r',
            b't',
            b'-',
            b'N',
            b'e',
            b't',
            0, // ID
            0x00,
            0x50, // OpCode (OpOutput)
            0,
            14, // Protocol version
            0,  // Sequence
            0,  // Physical
            (universe & 0xFF) as u8,
            (universe >> 8) as u8, // Universe
            0x02,
            0x00, // Length (512)
        ]
    }

    fn map_pixels_to_band(
        &self,
        dmx_data: &mut [u8],
        frame: &[u8],
        col_up: usize,
        col_down: usize,
        uni_in_band: usize,
    ) {
        // Une bande physique de 259 LEDs fait un U inversé :
        // - Monte sur 130 LEDs (col_up)
        // - Redescend sur 129 LEDs (col_down)

        // Vérifier que les colonnes sont dans les limites
        if col_up >= 128 || col_down >= 128 {
            println!(
                "⚠️  Column out of bounds: col_up={}, col_down={}",
                col_up, col_down
            );
            return;
        }

        if uni_in_band == 0 {
            // Premier univers: LEDs 0-169 (170 LEDs)
            let mut dmx_offset = 0;

            // Partie montante : LEDs 0-129 (130 LEDs)
            for led in 0..130 {
                if dmx_offset + 2 < 510 {
                    // 170 * 3 = 510
                    // La LED physique 0 est en bas, on monte vers le haut
                    let y = 127 - (led * 128 / 130); // Répartir 130 LEDs sur 128 pixels
                    let y = y.min(127); // S'assurer qu'on ne dépasse pas

                    let pixel_idx = (y * 128 + col_up) * 3;
                    if pixel_idx + 2 < frame.len() {
                        dmx_data[dmx_offset] = frame[pixel_idx];
                        dmx_data[dmx_offset + 1] = frame[pixel_idx + 1];
                        dmx_data[dmx_offset + 2] = frame[pixel_idx + 2];
                    }
                    dmx_offset += 3;
                }
            }

            // Début de la partie descendante : LEDs 130-169 (40 LEDs)
            for led in 0..40 {
                if dmx_offset + 2 < 510 {
                    // On redescend depuis le haut
                    let y = led * 128 / 129; // Répartir 129 LEDs sur 128 pixels
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

            // Suite de la partie descendante : LEDs 170-258 (89 LEDs)
            for led in 40..129 {
                if dmx_offset + 2 < 267 {
                    // 89 * 3 = 267
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
}
