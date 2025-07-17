use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;

mod audio;
mod effects;
mod fft;
mod ihub;
mod led;
mod udp;

use audio::AudioCapture;
use effects::EffectEngine;
use led::{LedController, LedMode};
use std::env;
use udp::UdpServer;

pub struct AppState {
    pub spectrum: Mutex<Vec<f32>>,
    pub effect_engine: Mutex<EffectEngine>,
    pub led_frame: Mutex<Vec<u8>>,
}

fn main() -> Result<()> {
    let test_mode = env::args().any(|arg| arg == "--test");
    let production_mode = env::args().any(|arg| arg == "--production");

    let state = Arc::new(AppState {
        spectrum: Mutex::new(vec![0.0; 64]),
        effect_engine: Mutex::new(EffectEngine::new()),
        led_frame: Mutex::new(vec![0; 128 * 128 * 3]),
    });

    let audio_state = state.clone();
    std::thread::spawn(move || {
        if test_mode {
            let mut time = 0.0f32;
            loop {
                let mut spectrum = vec![0.0; 64];
                for i in 0..64 {
                    spectrum[i] = ((time * (i as f32 + 1.0) * 0.1).sin() + 1.0)
                        * 0.5
                        * if i < 8 { 1.0 } else { 0.5 };
                }
                *audio_state.spectrum.lock() = spectrum.clone();

                let mut engine = audio_state.effect_engine.lock();
                let frame = engine.render(&spectrum);
                *audio_state.led_frame.lock() = frame;

                time += 0.05;
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        } else {
            match AudioCapture::new(move |data| {
                let spectrum = fft::compute_spectrum(data);
                *audio_state.spectrum.lock() = spectrum;

                let mut engine = audio_state.effect_engine.lock();
                let frame = engine.render(&audio_state.spectrum.lock());
                *audio_state.led_frame.lock() = frame;
            }) {
                Ok(audio) => {
                    audio.run();
                }
                Err(e) => {}
            }
        }
    });

    let led_state = state.clone();
    let production = production_mode;
    std::thread::spawn(move || {
        let mode = if production {
            LedMode::Production
        } else {
            LedMode::Simulator
        };
        let mut led = LedController::new_with_mode(mode).expect("Failed to init LED");

        let mut frame_count = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let frame = led_state.led_frame.lock().clone();
            led.send_frame(&frame);

            frame_count += 1;
            if frame_count % 100 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let fps = frame_count as f64 / elapsed;
            }

            std::thread::sleep(std::time::Duration::from_millis(13));
        }
    });

    let server = UdpServer::new(state)?;
    server.run()?;

    Ok(())
}
