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
        let device = host.default_input_device().ok_or_else(|| anyhow::anyhow!("No default input device"))?;

        for (idx, device) in host.input_devices()?.enumerate() {}

        let supported_configs = device.supported_input_configs()?;
        for (idx, config) in supported_configs.enumerate() {}

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

                let max_level = data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
                let avg_level = data.iter().map(|&x| x.abs()).sum::<f32>() / data.len() as f32;

                if last_log_time.elapsed().as_secs() >= 1 {
                    sample_counter = 0;
                    last_log_time = std::time::Instant::now();
                }

                if avg_level > 0.002 || max_level > 0.01 {
                    let filtered_data: Vec<f32> = data
                        .iter()
                        .map(|&x| {
                            let abs_x = x.abs();
                            if abs_x < 0.004 {
                                0.0
                            } else {
                                x
                            }
                        })
                        .collect();

                    callback(&filtered_data);
                } else {
                    let silence = vec![0.0; data.len()];
                    callback(&silence);
                }
            },
            |err| eprintln!(""),
            None,
        ).map_err(|e| anyhow::anyhow!("Failed to create stream: {}", e))?;

        stream.play()?;

        Ok(Self { stream })
    }

    pub fn run(&self) {
        std::thread::park();
    }
}
