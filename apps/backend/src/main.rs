use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Runtime;

mod audio;

mod effects;
mod ehub;
mod fft;
mod led;
mod websocket;

use audio::AudioCapture;

use effects::EffectEngine;
use led::{LedController, LedMode};
use std::env;
use websocket::WebSocketServer;

pub struct AppState {
    pub spectrum: Mutex<Vec<f32>>,
    pub effect_engine: Mutex<EffectEngine>,
    pub led_frame: Mutex<Vec<u8>>,
}

fn main() -> Result<()> {
    println!("🎵 LED Visualizer Starting...");

    // Mode test sans audio réel
    let test_mode = env::args().any(|arg| arg == "--test");
    if test_mode {
        println!("🧪 Mode test activé (audio simulé)");
    }

    // Mode production pour le vrai panneau LED
    let production_mode = env::args().any(|arg| arg == "--production");
    if production_mode {
        println!("🚀 Mode production activé (vrais contrôleurs LED)");
    }

    // État partagé entre threads

    let state = Arc::new(AppState {
        spectrum: Mutex::new(vec![0.0; 64]),
        effect_engine: Mutex::new(EffectEngine::new()),
        led_frame: Mutex::new(vec![0; 128 * 128 * 3]),
    });

    // Thread audio (temps réel)
    let audio_state = state.clone();
    std::thread::spawn(move || {
        if test_mode {
            // Mode test : générer des données audio simulées
            println!("🎧 Audio simulé démarré");
            let mut time = 0.0f32;
            loop {
                // Simuler un spectre audio avec des ondes sinusoïdales
                let mut spectrum = vec![0.0; 64];
                for i in 0..64 {
                    // Créer des patterns intéressants
                    spectrum[i] = ((time * (i as f32 + 1.0) * 0.1).sin() + 1.0)
                        * 0.5
                        * if i < 8 { 1.0 } else { 0.5 }; // Boost les basses
                }
                *audio_state.spectrum.lock() = spectrum.clone();

                // Génération visuelle
                let mut engine = audio_state.effect_engine.lock();
                let frame = engine.render(&spectrum);
                *audio_state.led_frame.lock() = frame;

                time += 0.05;
                std::thread::sleep(std::time::Duration::from_millis(20)); // 50 FPS
            }
        } else {
            match AudioCapture::new(move |data| {
                // Analyse FFT
                let spectrum = fft::compute_spectrum(data);
                *audio_state.spectrum.lock() = spectrum;

                // Génération visuelle
                let mut engine = audio_state.effect_engine.lock();
                let frame = engine.render(&audio_state.spectrum.lock());
                *audio_state.led_frame.lock() = frame;
            }) {
                Ok(audio) => {
                    println!("✅ Capture audio démarrée");
                    audio.run();
                }
                Err(e) => {
                    eprintln!("❌ Erreur capture audio: {}", e);
                    eprintln!("💡 Essayez avec --test pour le mode simulation");
                }
            }
        }
    });

    // Thread LED (envoi réseau)
    let led_state = state.clone();
    let production = production_mode;
    std::thread::spawn(move || {
        let mode = if production {
            LedMode::Production
        } else {
            LedMode::Simulator
        };
        let mut led = LedController::new_with_mode(mode).expect("Failed to init LED");
        println!(
            "🌐 Contrôleur LED démarré en mode {}",
            if production {
                "production"
            } else {
                "simulateur"
            }
        );

        let mut frame_count = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let frame = led_state.led_frame.lock().clone();
            led.send_frame(&frame);

            frame_count += 1;
            if frame_count % 100 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let fps = frame_count as f64 / elapsed;
                println!("📊 LED FPS: {:.1} | Frames: {}", fps, frame_count);
            }

            std::thread::sleep(std::time::Duration::from_millis(13)); // ~75 FPS
        }
    });

    // Serveur WebSocket (async)
    let rt = Runtime::new()?;
    rt.block_on(async {
        let server = WebSocketServer::new(state);
        server.run().await
    })?;

    Ok(())
}
