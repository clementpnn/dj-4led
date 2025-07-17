use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
    println!("🎵 LED Visualizer Starting...");

    // Debug : afficher tous les arguments reçus
    let args: Vec<String> = env::args().collect();
    println!("🔍 Arguments reçus: {:?}", args);

    // Analyse des modes
    let test_mode = args.iter().any(|arg| arg == "--test");
    let force_audio = args.iter().any(|arg| arg == "--audio" || arg == "--force-audio");
    let production_mode = args.iter().any(|arg| arg == "--production");
    let no_audio_sim = args.iter().any(|arg| arg == "--no-sim" || arg == "--real-audio-only");

    println!("🎛️ Analyse des modes:");
    println!("   --test présent: {}", test_mode);
    println!("   --audio/--force-audio présent: {}", force_audio);
    println!("   --production présent: {}", production_mode);
    println!("   --no-sim/--real-audio-only présent: {}", no_audio_sim);

    // NOUVELLE LOGIQUE: En mode production, on privilégie l'audio réel par défaut
    let use_real_audio = if production_mode && !args.iter().any(|arg| arg == "--force-test") {
        // En production, audio réel par défaut sauf si --force-test
        println!("🎯 Mode production détecté → Audio réel activé par défaut");
        true
    } else if force_audio || no_audio_sim {
        // Forçage explicite de l'audio réel
        true
    } else if test_mode && !force_audio {
        // Mode test explicite sans forçage
        false
    } else {
        // Par défaut: audio réel
        true
    };

    if use_real_audio {
        println!("✅ MODE AUDIO RÉEL SÉLECTIONNÉ (VB-Cable)");
        if production_mode && !args.iter().any(|arg| arg == "--force-test") {
            println!("   → Automatique en mode production");
        } else if force_audio {
            println!("   → Forcé via --audio/--force-audio");
        } else {
            println!("   → Par défaut");
        }
        println!("🎧 Recherche de VB-Cable...");
    } else {
        println!("✅ MODE AUDIO SIMULÉ SÉLECTIONNÉ");
        if args.iter().any(|arg| arg == "--force-test") {
            println!("   → Forcé via --force-test");
        } else {
            println!("   → Via --test (utilisez --production seul pour VB-Cable)");
        }
    }

    // Mode production pour les LEDs
    if production_mode {
        println!("🚀 Mode production LED activé (vrais contrôleurs LED)");
    } else {
        println!("🧪 Mode simulateur LED activé");
    }

    // Flag pour arrêt propre
    let running = Arc::new(AtomicBool::new(true));

    // État partagé entre threads
    let state = Arc::new(AppState {
        spectrum: Mutex::new(vec![0.0; 64]),
        effect_engine: Mutex::new(EffectEngine::new()),
        led_frame: Mutex::new(vec![0; 128 * 128 * 3]),
    });

    // Thread audio (temps réel)
    let audio_state = state.clone();
    let audio_running = running.clone();

    let audio_handle = std::thread::spawn(move || {
        if use_real_audio {
            println!("🎧 DÉMARRAGE CAPTURE AUDIO RÉELLE");
            run_real_audio(audio_state, audio_running);
        } else {
            println!("🎧 DÉMARRAGE AUDIO SIMULÉ");
            run_test_audio(audio_state, audio_running);
        }
    });

    // Thread LED (envoi réseau)
    let led_state = state.clone();
    let led_running = running.clone();

    let led_handle = std::thread::spawn(move || {
        run_led_controller(led_state, production_mode, led_running);
    });

    // Serveur UDP dans le thread principal
    println!("🌐 Démarrage du serveur UDP...");
    println!("✅ Tous les services démarrés");
    println!("🎮 Le programme tourne - fermez la console pour arrêter");

    // Serveur UDP bloque le thread principal
    let server = UdpServer::new(state)?;
    if let Err(e) = server.run() {
        eprintln!("❌ Erreur serveur UDP: {}", e);
    }

    // Attendre que les threads se terminent (si le serveur UDP s'arrête)
    let _ = audio_handle.join();
    let _ = led_handle.join();

    println!("✅ Arrêt terminé");
    Ok(())
}

fn run_test_audio(state: Arc<AppState>, running: Arc<AtomicBool>) {
    println!("🎵 AUDIO SIMULÉ - Génération de données de test");
    let mut time = 0.0f32;
    let mut frame_count = 0u64;

    while running.load(Ordering::Relaxed) {
        // Simuler un spectre audio avec des ondes sinusoïdales
        let mut spectrum = vec![0.0; 64];
        for i in 0..64 {
            spectrum[i] = ((time * (i as f32 + 1.0) * 0.1).sin() + 1.0)
                * 0.5
                * if i < 8 { 1.0 } else { 0.5 }; // Boost les basses
        }
        *state.spectrum.lock() = spectrum.clone();

        // Génération visuelle
        let mut engine = state.effect_engine.lock();
        let frame = engine.render(&spectrum);
        *state.led_frame.lock() = frame;

        time += 0.05;
        frame_count += 1;

        // Log périodique pour confirmer que ça tourne
        if frame_count % 250 == 0 { // Toutes les 5 secondes environ
            println!("🎵 Audio simulé actif: {} frames générées", frame_count);
        }

        std::thread::sleep(std::time::Duration::from_millis(20)); // 50 FPS
    }
    println!("🎧 Thread audio simulé arrêté");
}

fn run_real_audio(state: Arc<AppState>, running: Arc<AtomicBool>) {
    println!("🔍 SCAN DES PÉRIPHÉRIQUES AUDIO");

    // Debug des périphériques disponibles
    if let Err(e) = AudioCapture::list_devices() {
        eprintln!("⚠️ Impossible de lister les périphériques: {}", e);
    }

    let state_clone = state.clone();
    let mut callback_count = 0u64;

    println!("🔄 TENTATIVE DE CRÉATION DU STREAM AUDIO");

    match AudioCapture::new(move |data| {
        callback_count += 1;

        // Debug périodique pour confirmer la réception
        if callback_count % 500 == 0 {
            println!("🔊 AUDIO RÉEL ACTIF: {} callbacks reçus, {} échantillons",
                     callback_count, data.len());
        }

        // Analyse FFT
        let spectrum = fft::compute_spectrum(data);
        *state_clone.spectrum.lock() = spectrum.clone();

        // Génération visuelle
        let mut engine = state_clone.effect_engine.lock();
        let frame = engine.render(&spectrum);
        *state_clone.led_frame.lock() = frame;
    }) {
        Ok(_audio) => {
            println!("✅ CAPTURE AUDIO RÉELLE DÉMARRÉE AVEC SUCCÈS");
            println!("🎵 Stream audio actif - en attente de données...");

            // Dans un thread séparé pour permettre l'arrêt propre
            while running.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            println!("🎧 Thread audio réel arrêté");
        }
        Err(e) => {
            eprintln!("❌ ERREUR CAPTURE AUDIO: {}", e);
            eprintln!("💡 Vérifiez que VB-Cable est installé et configuré");
            eprintln!("💡 Ou relancez avec --test pour forcer le mode simulation");
            eprintln!("🔄 BASCULEMENT AUTOMATIQUE EN MODE SIMULATION");

            // Fallback en mode simulation
            run_test_audio(state, running);
        }
    }
}

fn run_led_controller(state: Arc<AppState>, production: bool, running: Arc<AtomicBool>) {
    let mode = if production {
        LedMode::Production
    } else {
        LedMode::Simulator
    };

    let mut led = match LedController::new_with_mode(mode) {
        Ok(controller) => controller,
        Err(e) => {
            eprintln!("❌ Erreur initialisation LED: {}", e);
            return;
        }
    };

    println!(
        "🌐 Contrôleur LED démarré en mode {}",
        if production { "production" } else { "simulateur" }
    );

    let mut frame_count = 0u64;
    let start_time = std::time::Instant::now();

    while running.load(Ordering::Relaxed) {
        let frame = state.led_frame.lock().clone();
        led.send_frame(&frame);

        frame_count += 1;
        if frame_count % 100 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let fps = frame_count as f64 / elapsed;
            println!("📊 LED FPS: {:.1} | Frames: {}", fps, frame_count);
        }

        std::thread::sleep(std::time::Duration::from_millis(13)); // ~75 FPS
    }

    println!("🌐 Thread LED arrêté");
}
