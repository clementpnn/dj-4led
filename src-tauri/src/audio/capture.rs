use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};

pub struct AudioCapture {
    #[allow(dead_code)]
    stream: cpal::Stream,
}

impl AudioCapture {
    pub fn new<F>(mut callback: F) -> Result<Self>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        println!("🔍 [CAPTURE] Init capture audio");

        let host = cpal::default_host();

        // Chercher VB-Cable en priorité
        let device = match Self::find_vb_cable_device(&host)? {
            Some(vb_device) => {
                println!("✅ [CAPTURE] VB-Cable trouvé");
                vb_device
            }
            None => {
                println!("⚠️ [CAPTURE] VB-Cable non trouvé, device par défaut");
                host.default_input_device()
                    .ok_or_else(|| anyhow::anyhow!("Aucun périphérique d'entrée par défaut"))?
            }
        };

        let device_name = device.name().unwrap_or("Unknown".to_string());
        println!("📱 [CAPTURE] Device: {}", device_name);

        // Essayer d'obtenir la configuration par défaut du périphérique
        let final_config = match device.default_input_config() {
            Ok(default_config) => {
                println!("🔧 [CAPTURE] Config: {}ch @ {}Hz",
                         default_config.channels(), default_config.sample_rate().0);
                default_config.into()
            }
            Err(_) => {
                // Configuration de fallback
                let fallback_config = StreamConfig {
                    channels: 2,
                    sample_rate: SampleRate(44100),
                    buffer_size: cpal::BufferSize::Fixed(256),
                };

                println!("🔧 [CAPTURE] Config fallback: 2ch @ 44100Hz");
                fallback_config
            }
        };

        let mut sample_counter = 0u64;
        let mut last_log_time = std::time::Instant::now();
        let channels = final_config.channels;

        let stream = device.build_input_stream(
            &final_config,
            move |data: &[f32], _: &_| {
                sample_counter += data.len() as u64;

                // Convertir stéréo en mono si nécessaire
                let mono_data: Vec<f32> = if channels == 2 {
                    data.chunks(2)
                        .map(|chunk| {
                            if chunk.len() >= 2 {
                                (chunk[0] + chunk[1]) / 2.0
                            } else {
                                chunk[0]
                            }
                        })
                        .collect()
                } else {
                    data.to_vec()
                };

                // Calculer le niveau audio
                let max_level = mono_data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
                let avg_level = mono_data.iter().map(|&x| x.abs()).sum::<f32>() / mono_data.len() as f32;

                // Log périodique (toutes les 5 secondes)
                if last_log_time.elapsed().as_secs() >= 5 {
                    let signal_detected = if avg_level > 0.0001 || max_level > 0.001 {
                        "🎵 SIGNAL"
                    } else {
                        "🔇 SILENCE"
                    };
                    println!(
                        "{} [CAPTURE] Audio: {} samples/sec, max: {:.4}, avg: {:.4} | {} échantillons mono",
                        signal_detected, sample_counter, max_level, avg_level, mono_data.len()
                    );
                    sample_counter = 0;
                    last_log_time = std::time::Instant::now();
                }

                if avg_level > 0.0001 || max_level > 0.001 {
                    let filtered_data: Vec<f32> = mono_data
                        .iter()
                        .map(|&x| {
                            if x.abs() < 0.0005 {
                                0.0
                            } else {
                                x
                            }
                        })
                        .collect();

                    callback(&filtered_data);
                } else {
                    let silence = vec![0.0; mono_data.len()];
                    callback(&silence);
                }
            },
            |err| {
                eprintln!("❌ [CAPTURE] Stream error: {}", err);
            },
            None,
        ).map_err(|e| anyhow::anyhow!("Impossible de créer le stream: {}", e))?;

        stream.play().map_err(|e| anyhow::anyhow!("Impossible de démarrer le stream: {}", e))?;
        println!("✅ [CAPTURE] Stream démarré sur: {}", device_name);

        Ok(Self { stream })
    }

    // Fonction pour trouver automatiquement VB-Cable
    fn find_vb_cable_device(host: &cpal::Host) -> Result<Option<cpal::Device>> {
        let devices = host.input_devices()?;

        for device in devices {
            if let Ok(name) = device.name() {
                let name_lower = name.to_lowercase();
                if name_lower.contains("vb-cable")
                    || name_lower.contains("vb cable")
                    || name_lower.contains("cable input")
                    || name_lower.contains("virtual cable")
                    || name_lower.contains("vb-audio")
                    || name_lower.contains("voicemeeter") {
                    return Ok(Some(device));
                }
            }
        }

        Ok(None)
    }

    #[allow(dead_code)]
    pub fn run(&self) {
        println!("🎧 [CAPTURE] Audio capture running...");
        std::thread::park();
    }

    // Fonction utilitaire pour lister tous les périphériques disponibles
    pub fn list_devices() -> Result<()> {
        let host = cpal::default_host();

        println!("🔍 [CAPTURE] Diagnostic audio:");

        // Périphériques d'entrée
        println!("📥 [CAPTURE] Périphériques d'ENTRÉE:");
        match host.input_devices() {
            Ok(devices) => {
                let devices: Vec<_> = devices.collect();
                if devices.is_empty() {
                    println!("   ❌ Aucun périphérique trouvé");
                } else {
                    for (idx, device) in devices.iter().enumerate() {
                        let name = device.name().unwrap_or("Unknown".to_string());
                        println!("   {}. {}", idx, name);
                    }
                }
            }
            Err(e) => println!("   ❌ Erreur: {}", e),
        }

        // Périphérique par défaut
        println!("🎯 [CAPTURE] Device par défaut:");
        match host.default_input_device() {
            Some(device) => {
                println!("   ✅ {}", device.name().unwrap_or("Unknown".to_string()));
            }
            None => println!("   ❌ Aucun device par défaut"),
        }

        Ok(())
    }
}
