use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};

pub struct AudioCapture {
    stream: cpal::Stream,
}

impl AudioCapture {
    pub fn new<F>(mut callback: F) -> Result<Self>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        println!("🔍 Initialisation de la capture audio...");

        let host = cpal::default_host();
        println!("🎛️ Host audio: {:?}", host.id());

        // Lister tous les dispositifs audio disponibles
        println!("🎤 Available audio input devices:");
        let input_devices: Vec<_> = match host.input_devices() {
            Ok(devices) => devices.collect(),
            Err(e) => {
                return Err(anyhow::anyhow!("Impossible d'énumérer les périphériques: {}", e));
            }
        };

        if input_devices.is_empty() {
            return Err(anyhow::anyhow!("Aucun périphérique d'entrée audio trouvé"));
        }

        for (idx, device) in input_devices.iter().enumerate() {
            let name = device.name().unwrap_or_default();
            println!("   {}. {}", idx, name);
        }

        // Chercher VB-Cable en priorité
        let device = match Self::find_vb_cable_device(&host)? {
            Some(vb_device) => {
                println!("✅ VB-Cable trouvé et sélectionné");
                vb_device
            }
            None => {
                println!("⚠️ VB-Cable non trouvé, utilisation du périphérique par défaut");
                host.default_input_device()
                    .ok_or_else(|| anyhow::anyhow!("Aucun périphérique d'entrée par défaut"))?
            }
        };

        let device_name = device.name().unwrap_or("Unknown".to_string());
        println!("📱 Using audio device: {}", device_name);

        // Afficher les configurations supportées
        match device.supported_input_configs() {
            Ok(supported_configs) => {
                println!("📊 Supported configurations:");
                for (idx, config) in supported_configs.enumerate() {
                    println!(
                        "   {}. Channels: {}, Sample rate: {} - {}",
                        idx,
                        config.channels(),
                        config.min_sample_rate().0,
                        config.max_sample_rate().0
                    );
                }
            }
            Err(e) => {
                println!("⚠️ Impossible de lire les configurations supportées: {}", e);
            }
        }

        // Essayer d'obtenir la configuration par défaut du périphérique
        let final_config = match device.default_input_config() {
            Ok(default_config) => {
                println!("🔧 Using device default config: channels={}, rate={}",
                         default_config.channels(), default_config.sample_rate().0);
                default_config.into()
            }
            Err(e) => {
                println!("⚠️ Config par défaut non disponible: {}, tentative config personnalisée", e);

                // Configuration de fallback
                let fallback_config = StreamConfig {
                    channels: 2,
                    sample_rate: SampleRate(44100),
                    buffer_size: cpal::BufferSize::Fixed(256),
                };

                println!("🔧 Using fallback config: {:?}", fallback_config);
                fallback_config
            }
        };

        let mut sample_counter = 0u64;
        let mut last_log_time = std::time::Instant::now();
        let channels = final_config.channels;

        println!("🔊 Configuration finale: {} canaux à {} Hz",
                 channels, final_config.sample_rate.0);

        let stream = device.build_input_stream(
            &final_config,
            move |data: &[f32], _: &_| {
                sample_counter += data.len() as u64;

                // Convertir stéréo en mono si nécessaire
                let mono_data: Vec<f32> = if channels == 2 {
                    data.chunks(2)
                        .map(|chunk| {
                            if chunk.len() >= 2 {
                                (chunk[0] + chunk[1]) / 2.0 // Moyenner les deux canaux
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

                // Log périodique (toutes les 2 secondes pour plus de clarté)
                if last_log_time.elapsed().as_secs() >= 1 { // Plus fréquent pour debug
                    let signal_detected = if avg_level > 0.0001 || max_level > 0.001 { "🎵 SIGNAL" } else { "🔇 SILENCE" };
                    println!(
                        "{} Audio: {} samples/sec, max: {:.4}, avg: {:.4} | {} échantillons mono",
                        signal_detected, sample_counter, max_level, avg_level, mono_data.len()
                    );
                    sample_counter = 0;
                    last_log_time = std::time::Instant::now();
                }

                // Seuils adaptés pour détecter même de faibles signaux
                if avg_level > 0.0001 || max_level > 0.001 {
                    // Filtrage de bruit minimal pour préserver le signal
                    let filtered_data: Vec<f32> = mono_data
                        .iter()
                        .map(|&x| {
                            let abs_x = x.abs();
                            if abs_x < 0.0005 {
                                0.0 // Seuil très bas
                            } else {
                                x
                            }
                        })
                        .collect();

                    callback(&filtered_data);
                } else {
                    // En cas de silence complet
                    let silence = vec![0.0; mono_data.len()];
                    callback(&silence);
                }
            },
            |err| {
                eprintln!("❌ Erreur du stream audio: {}", err);
                eprintln!("💡 Le périphérique a peut-être été déconnecté");
            },
            None,
        ).map_err(|e| anyhow::anyhow!("Impossible de créer le stream: {}", e))?;

        stream.play().map_err(|e| anyhow::anyhow!("Impossible de démarrer le stream: {}", e))?;
        println!("✅ Stream audio démarré avec succès sur: {}", device_name);

        Ok(Self { stream })
    }

    // Fonction pour trouver automatiquement VB-Cable
    fn find_vb_cable_device(host: &cpal::Host) -> Result<Option<cpal::Device>> {
        let devices = host.input_devices()?;

        for device in devices {
            if let Ok(name) = device.name() {
                let name_lower = name.to_lowercase();
                // Recherche plus large pour VB-Cable
                if name_lower.contains("vb-cable")
                    || name_lower.contains("vb cable")
                    || name_lower.contains("cable input")
                    || name_lower.contains("virtual cable")
                    || name_lower.contains("vb-audio")
                    || name_lower.contains("voicemeeter") {
                    println!("🎯 Found VB-Cable/Virtual device: {}", name);
                    return Ok(Some(device));
                }
            }
        }

        Ok(None)
    }

    pub fn run(&self) {
        println!("🎧 Audio capture running...");
        println!("💡 Assurez-vous que VB-Cable est installé et que votre source audio y est routée");
        println!("💡 Dans Windows, vérifiez les paramètres de son et routez votre audio vers VB-Cable");
        println!("🛑 Appuyez sur Ctrl+C pour arrêter");

        // Keep thread alive
        std::thread::park();
    }

    // Fonction utilitaire pour lister tous les périphériques disponibles
    pub fn list_devices() -> Result<()> {
        let host = cpal::default_host();

        println!("🔍 Diagnostic des périphériques audio:");
        println!("   Host: {:?}", host.id());

        // Périphériques d'entrée
        println!("\n📥 Périphériques d'ENTRÉE:");
        match host.input_devices() {
            Ok(devices) => {
                let devices: Vec<_> = devices.collect();
                if devices.is_empty() {
                    println!("   ❌ Aucun périphérique d'entrée trouvé");
                } else {
                    for (idx, device) in devices.iter().enumerate() {
                        let name = device.name().unwrap_or("Unknown".to_string());
                        println!("   {}. {}", idx, name);

                        // Détails pour les périphériques virtuels
                        if name.to_lowercase().contains("vb")
                            || name.to_lowercase().contains("cable")
                            || name.to_lowercase().contains("virtual") {
                            if let Ok(config) = device.default_input_config() {
                                println!("      ↳ Config: {} canaux, {} Hz",
                                         config.channels(), config.sample_rate().0);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("   ❌ Erreur: {}", e),
        }

        // Périphérique par défaut
        println!("\n🎯 Périphérique d'entrée par défaut:");
        match host.default_input_device() {
            Some(device) => {
                println!("   ✅ {}", device.name().unwrap_or("Unknown".to_string()));
            }
            None => println!("   ❌ Aucun périphérique par défaut"),
        }

        Ok(())
    }
}
