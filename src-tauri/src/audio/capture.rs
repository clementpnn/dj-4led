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
        let host = cpal::default_host();

        // Sélectionner le meilleur device disponible
        let device = Self::find_working_device(&host)?;
        let device_name = device.name().unwrap_or("Unknown".to_string());

        // Obtenir la meilleure configuration
        let config = Self::get_best_config(&device)?;
        let channels = config.channels;

        // Buffer pour accumuler les échantillons
        let mut audio_buffer = Vec::with_capacity(4096);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                // Convertir en mono si nécessaire
                let mono_data: Vec<f32> = if channels == 1 {
                    data.to_vec()
                } else {
                    // Moyenne de tous les canaux
                    data.chunks(channels as usize)
                        .map(|chunk| {
                            chunk.iter().sum::<f32>() / chunk.len() as f32
                        })
                        .collect()
                };

                // Ajouter au buffer d'accumulation
                audio_buffer.extend_from_slice(&mono_data);

                // Envoyer des chunks de 1024 échantillons pour FFT
                while audio_buffer.len() >= 1024 {
                    let chunk: Vec<f32> = audio_buffer.drain(0..1024).collect();
                    callback(&chunk);
                }
            },
            |err| {
                eprintln!("❌ [CAPTURE] Stream error: {}", err);
            },
            None,
        ).map_err(|e| anyhow::anyhow!("Cannot create audio stream: {}", e))?;

        // Démarrer le stream
        stream.play().map_err(|e| anyhow::anyhow!("Cannot start audio stream: {}", e))?;
        println!("✅ [CAPTURE] Audio capture started on: {}", device_name);

        Ok(Self { stream })
    }

    // Trouver un device audio qui fonctionne
    fn find_working_device(host: &cpal::Host) -> Result<cpal::Device> {
        // 1. Essayer le device par défaut
        if let Some(default_device) = host.default_input_device() {
            return Ok(default_device);
        }

        // 2. Prendre le premier device disponible
        if let Ok(mut devices) = host.input_devices() {
            if let Some(first_device) = devices.next() {
                return Ok(first_device);
            }
        }

        Err(anyhow::anyhow!("No audio input device found"))
    }

    // Obtenir la meilleure configuration pour un device
    fn get_best_config(device: &cpal::Device) -> Result<StreamConfig> {
        // Essayer la config par défaut d'abord
        if let Ok(default_config) = device.default_input_config() {
            let mut config: StreamConfig = default_config.into();
            // Optimiser le buffer pour FFT
            config.buffer_size = cpal::BufferSize::Fixed(1024);
            return Ok(config);
        }

        // Configurations de fallback
        let fallback_configs = vec![
            StreamConfig {
                channels: 2,
                sample_rate: SampleRate(44100),
                buffer_size: cpal::BufferSize::Fixed(1024),
            },
            StreamConfig {
                channels: 1,
                sample_rate: SampleRate(44100),
                buffer_size: cpal::BufferSize::Fixed(1024),
            },
            StreamConfig {
                channels: 2,
                sample_rate: SampleRate(48000),
                buffer_size: cpal::BufferSize::Fixed(1024),
            },
            StreamConfig {
                channels: 1,
                sample_rate: SampleRate(22050),
                buffer_size: cpal::BufferSize::Fixed(512),
            },
        ];

        for config in fallback_configs {
            if device.supported_input_configs().is_ok() {
                return Ok(config);
            }
        }

        Err(anyhow::anyhow!("Cannot find valid audio configuration"))
    }

    // Diagnostic des périphériques disponibles
    pub fn list_devices() -> Result<()> {
        let host = cpal::default_host();

        println!("🔍 [CAPTURE] Available audio devices:");

        // Périphériques d'entrée
        match host.input_devices() {
            Ok(devices) => {
                let devices: Vec<_> = devices.collect();
                if devices.is_empty() {
                    println!("   ❌ No input devices found");
                } else {
                    for (idx, device) in devices.iter().enumerate() {
                        let name = device.name().unwrap_or("Unknown".to_string());
                        println!("   {}. {}", idx, name);

                        if let Ok(config) = device.default_input_config() {
                            println!("      └─ {}ch @ {}Hz",
                                     config.channels(),
                                     config.sample_rate().0);
                        }
                    }
                }
            }
            Err(e) => println!("   ❌ Error: {}", e),
        }

        // Périphérique par défaut
        match host.default_input_device() {
            Some(device) => {
                let name = device.name().unwrap_or("Unknown".to_string());
                println!("🎯 [CAPTURE] Default device: {}", name);
            }
            None => println!("❌ [CAPTURE] No default device"),
        }

        Ok(())
    }
}
