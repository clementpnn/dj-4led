use parking_lot::Mutex;
use std::sync::Arc;

// Import modules
pub mod audio;
pub mod commands;
pub mod effects;
pub mod led;

// Application state shared across threads
#[derive(Clone)]
pub struct AppState {
    pub spectrum: Arc<Mutex<Vec<f32>>>,
    pub effect_engine: Arc<Mutex<effects::EffectEngine>>,
    pub led_frame: Arc<Mutex<Vec<u8>>>,
    pub audio_running: Arc<Mutex<bool>>,
    pub led_running: Arc<Mutex<bool>>,
    pub audio_gain: Arc<Mutex<f32>>,
    pub led_brightness: Arc<Mutex<f32>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        println!("🔧 [STATE] Initializing AppState...");
        Arc::new(Self {
            spectrum: Arc::new(Mutex::new(vec![0.0; 64])),
            effect_engine: Arc::new(Mutex::new(effects::EffectEngine::new())),
            led_frame: Arc::new(Mutex::new(vec![0; 128 * 128 * 3])),
            audio_running: Arc::new(Mutex::new(false)),
            led_running: Arc::new(Mutex::new(false)),
            audio_gain: Arc::new(Mutex::new(1.0)),
            led_brightness: Arc::new(Mutex::new(1.0)),
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("🚀 Starting LED Audio Visualizer...");
    println!("✅ EffectEngine initialized with 8 effects");

    let app_state = AppState::new();

    // 🔥 Démarrage automatique de l'audio en arrière-plan
    let auto_audio_state = Arc::clone(&app_state);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1000)); // Attendre que Tauri soit prêt
        println!("🎧 [AUTO] Tentative de démarrage automatique de l'audio...");
        start_auto_audio(auto_audio_state);
    });

    // 🔥 Démarrage automatique des LEDs en mode simulateur
    let auto_led_state = Arc::clone(&app_state);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500)); // Attendre un peu plus
        println!("🌐 [AUTO] Démarrage automatique du contrôleur LED...");
        start_auto_led(auto_led_state);
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state.as_ref().clone())
        .invoke_handler(tauri::generate_handler![
            // Audio commands
            commands::audio::get_audio_devices,
            commands::audio::start_audio_capture,
            commands::audio::stop_audio_capture,
            commands::audio::get_current_spectrum,
            commands::audio::set_audio_gain,
            commands::audio::get_audio_gain,
            // Effects commands
            commands::effects::get_available_effects,
            commands::effects::set_effect,
            commands::effects::get_current_effect,
            commands::effects::set_color_mode,
            commands::effects::get_color_mode,
            commands::effects::set_custom_color,
            commands::effects::get_custom_color,
            commands::effects::set_effect_parameter,
            commands::effects::get_effect_parameter,
            commands::effects::get_current_frame,
            // LED commands
            commands::led::start_led_output,
            commands::led::stop_led_output,
            commands::led::is_led_running,
            commands::led::set_led_brightness,
            commands::led::get_led_brightness,
            commands::led::test_led_pattern,
            commands::led::get_led_controllers,
            commands::led::get_led_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LED visualizer application");
}

// 🔥 Fonction pour démarrer l'audio automatiquement
fn start_auto_audio(state: Arc<AppState>) {
    // Vérifier si l'audio est déjà en cours
    if *state.audio_running.lock() {
        println!("🎧 [AUTO] Audio déjà en cours, abandon");
        return;
    }

    println!("🔍 [AUTO] Scan des périphériques audio...");

    // Debug des périphériques disponibles
    if let Err(e) = audio::AudioCapture::list_devices() {
        eprintln!("⚠️ [AUTO] Impossible de lister les périphériques: {}", e);
    }

    let audio_state = Arc::clone(&state);
    let mut callback_count = 0u64;

    println!("🔄 [AUTO] Tentative de création du stream audio...");

    match audio::AudioCapture::new(move |data| {
        callback_count += 1;

        // Debug périodique pour confirmer la réception
        if callback_count % 500 == 0 {
            println!("🔊 [AUTO] AUDIO ACTIF: {} callbacks reçus, {} échantillons",
                     callback_count, data.len());
        }

        // Analyse FFT
        let spectrum = audio::compute_spectrum(data);
        *audio_state.spectrum.lock() = spectrum.clone();

        // Génération visuelle
        let mut engine = audio_state.effect_engine.lock();
        let frame = engine.render(&spectrum);
        *audio_state.led_frame.lock() = frame;
    }) {
        Ok(_audio) => {
            println!("✅ [AUTO] CAPTURE AUDIO AUTOMATIQUE DÉMARRÉE !");
            *state.audio_running.lock() = true;

            // Garder le stream audio vivant
            loop {
                if !*state.audio_running.lock() {
                    println!("🎧 [AUTO] Arrêt demandé, fermeture audio");
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        Err(e) => {
            eprintln!("❌ [AUTO] ERREUR CAPTURE AUDIO: {}", e);
            eprintln!("💡 [AUTO] Vérifiez que VB-Cable est installé et configuré");
            eprintln!("🔄 [AUTO] BASCULEMENT EN MODE SIMULATION");

            // Fallback en mode simulation
            start_simulated_audio(state);
        }
    }
}

// 🔥 Fonction pour l'audio simulé
fn start_simulated_audio(state: Arc<AppState>) {
    println!("🎵 [AUTO-SIM] Démarrage audio simulé...");
    *state.audio_running.lock() = true;

    let mut time = 0.0f32;
    let mut frame_count = 0u64;

    while *state.audio_running.lock() {
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
            println!("🎵 [AUTO-SIM] Audio simulé actif: {} frames générées", frame_count);
        }

        std::thread::sleep(std::time::Duration::from_millis(20)); // 50 FPS
    }

    println!("🎧 [AUTO-SIM] Thread audio simulé arrêté");
}

// 🔥 Fonction pour démarrer les LEDs automatiquement
fn start_auto_led(state: Arc<AppState>) {
    if *state.led_running.lock() {
        println!("🌐 [AUTO] LED déjà en cours, abandon");
        return;
    }

    println!("🌐 [AUTO] Initialisation contrôleur LED...");

    let mut led = match led::LedController::new_with_mode(led::LedMode::Simulator) {
        Ok(controller) => {
            println!("✅ [AUTO] Contrôleur LED initialisé en mode simulateur");
            controller
        }
        Err(e) => {
            eprintln!("❌ [AUTO] Erreur initialisation LED: {}", e);
            return;
        }
    };

    *state.led_running.lock() = true;
    let mut frame_count = 0u64;
    let start_time = std::time::Instant::now();

    while *state.led_running.lock() {
        let frame = state.led_frame.lock().clone();

        if let Err(e) = led.send_frame(&frame) {
            eprintln!("❌ [AUTO] Erreur envoi frame LED: {}", e);
        }

        frame_count += 1;
        if frame_count % 100 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let fps = frame_count as f64 / elapsed;
            println!("📊 [AUTO] LED FPS: {:.1} | Frames: {}", fps, frame_count);
        }

        std::thread::sleep(std::time::Duration::from_millis(13)); // ~75 FPS
    }

    println!("🌐 [AUTO] Thread LED arrêté");
}
