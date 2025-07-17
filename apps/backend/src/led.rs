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
        let mut universe = 0;

        for col in 0..128 {
            for uni_in_col in 0..2 {
                let mut artnet_packet = vec![
                    b'A',
                    b'r',
                    b't',
                    b'-',
                    b'N',
                    b'e',
                    b't',
                    0,
                    0x00,
                    0x50,
                    0,
                    14,
                    0,
                    0,
                    (universe & 0xFF) as u8,
                    (universe >> 8) as u8,
                    0x02,
                    0x00,
                ];

                let mut dmx_data = vec![0u8; 512];

                if col % 2 == 0 {
                    let start_pixel = uni_in_col * 64;
                    let end_pixel = ((uni_in_col + 1) * 64).min(128);

                    for pixel in start_pixel..end_pixel {
                        let led_idx = pixel - start_pixel;
                        let y = 127 - pixel;
                        let pixel_idx = (y * 128 + col) * 3;

                        if pixel_idx + 2 < frame.len() && led_idx * 3 + 2 < 512 {
                            dmx_data[led_idx * 3] = frame[pixel_idx];
                            dmx_data[led_idx * 3 + 1] = frame[pixel_idx + 1];
                            dmx_data[led_idx * 3 + 2] = frame[pixel_idx + 2];
                        }
                    }
                } else {
                    let start_pixel = uni_in_col * 64;
                    let end_pixel = ((uni_in_col + 1) * 64).min(128);

                    for pixel in start_pixel..end_pixel {
                        let led_idx = pixel - start_pixel;
                        let y = pixel;
                        let pixel_idx = (y * 128 + col) * 3;

                        if pixel_idx + 2 < frame.len() && led_idx * 3 + 2 < 512 {
                            dmx_data[led_idx * 3] = frame[pixel_idx];
                            dmx_data[led_idx * 3 + 1] = frame[pixel_idx + 1];
                            dmx_data[led_idx * 3 + 2] = frame[pixel_idx + 2];
                        }
                    }
                }

                artnet_packet.extend_from_slice(&dmx_data);

                let _ = self.socket.send_to(&artnet_packet, "127.0.0.1:6454");

                universe += 1;
            }
        }
    }

    fn send_frame_production(&mut self, frame: &[u8]) {
        let mut packets_sent = 0;

        for quarter in 0..4 {
            let controller_ip = &self.controllers[quarter];
            let base_universe = quarter * 32;

            for band_in_quarter in 0..16 {
                let physical_band = quarter * 16 + band_in_quarter;

                let col_up = physical_band * 2;
                let col_down = physical_band * 2 + 1;

                for uni_in_band in 0..2 {
                    let universe = base_universe + band_in_quarter * 2 + uni_in_band;
                    let mut artnet_packet = self.create_artnet_header(universe);
                    let mut dmx_data = vec![0u8; 512];

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
            0,
            0x00,
            0x50,
            0,
            14,
            0,
            0,
            (universe & 0xFF) as u8,
            (universe >> 8) as u8,
            0x02,
            0x00,
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
        if col_up >= 128 || col_down >= 128 {
            return;
        }

        if uni_in_band == 0 {
            let mut dmx_offset = 0;

            for led in 0..130 {
                if dmx_offset + 2 < 510 {
                    let y = 127 - (led * 128 / 130);
                    let y = y.min(127);

                    let pixel_idx = (y * 128 + col_up) * 3;
                    if pixel_idx + 2 < frame.len() {
                        dmx_data[dmx_offset] = frame[pixel_idx];
                        dmx_data[dmx_offset + 1] = frame[pixel_idx + 1];
                        dmx_data[dmx_offset + 2] = frame[pixel_idx + 2];
                    }
                    dmx_offset += 3;
                }
            }

            for led in 0..40 {
                if dmx_offset + 2 < 510 {
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
        } else {
            let mut dmx_offset = 0;

            for led in 40..129 {
                if dmx_offset + 2 < 267 {
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
