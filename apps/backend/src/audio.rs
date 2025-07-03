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
        let host = cpal::default_host();

        // Lister tous les dispositifs audio disponibles
        println!("🎤 Available audio input devices:");
        for (idx, device) in host.input_devices()?.enumerate() {
            println!("   {}. {}", idx, device.name().unwrap_or_default());
        }

        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device"))?;

        println!("📱 Using audio device: {}", device.name()?);

        // Afficher les configurations supportées
        let supported_configs = device.supported_input_configs()?;
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

        // Configuration minimale pour latence
        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: cpal::BufferSize::Fixed(64),
        };

        let mut sample_counter = 0u64;
        let mut last_log_time = std::time::Instant::now();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                sample_counter += data.len() as u64;

                // Calculer le niveau audio
                let max_level = data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
                let avg_level = data.iter().map(|&x| x.abs()).sum::<f32>() / data.len() as f32;

                // Log toutes les secondes
                if last_log_time.elapsed().as_secs() >= 1 {
                    println!(
                        "🎵 Audio capture: {} samples/sec, max: {:.3}, avg: {:.3}",
                        sample_counter, max_level, avg_level
                    );
                    sample_counter = 0;
                    last_log_time = std::time::Instant::now();
                }

                callback(data);
            },
            |err| eprintln!("❌ Audio stream error: {}", err),
            None,
        )?;

        stream.play()?;
        println!("✅ Audio stream started");

        Ok(Self { stream })
    }

    pub fn run(&self) {
        println!("🎧 Audio capture running... (Press Ctrl+C to stop)");
        // Keep thread alive
        std::thread::park();
    }
}
