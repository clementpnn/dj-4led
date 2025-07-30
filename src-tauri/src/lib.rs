// src-tauri/src/lib.rs

use parking_lot::Mutex;
use std::sync::Arc;
use tauri::Emitter;

// Import modules
pub mod audio;
pub mod commands;
pub mod effects;
pub mod led;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub spectrum: Arc<Mutex<Vec<f32>>>,
    pub effect_engine: Arc<Mutex<effects::EffectEngine>>,
    pub led_frame: Arc<Mutex<Vec<u8>>>,
    pub audio_running: Arc<Mutex<bool>>,
    pub led_running: Arc<Mutex<bool>>,
    pub audio_gain: Arc<Mutex<f32>>,
    pub led_brightness: Arc<Mutex<f32>>,
    pub led_mode: Arc<Mutex<String>>,
    pub last_error: Arc<Mutex<Option<String>>>,
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
            led_mode: Arc::new(Mutex::new("production".to_string())), // FORCER PRODUCTION PAR DÉFAUT
            last_error: Arc::new(Mutex::new(None)),
        })
    }

    pub fn set_error(&self, error: String) {
        println!("❌ [STATE] Error set: {}", error);
        *self.last_error.lock() = Some(error);
    }

    pub fn get_error(&self) -> Option<String> {
        self.last_error.lock().clone()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("🚀 LED Audio Visualizer - Production Mode");
    println!("🌐 Matrix: 128x128 = {} pixels total", 128 * 128);

    let app_state = AppState::new();

    // FORCER LE MODE PRODUCTION PAR DÉFAUT (mais permettre override)
    std::env::set_var("LED_MODE", std::env::var("LED_MODE").unwrap_or("production".to_string()));
    std::env::set_var("TAURI_LED_MODE", std::env::var("TAURI_LED_MODE").unwrap_or("production".to_string()));

    let final_mode = std::env::var("LED_MODE").unwrap_or("production".to_string());
    println!("🏭 [INIT] MODE LED: {}", final_mode);
    println!("🏭 [INIT] - LED_MODE: {}", std::env::var("LED_MODE").unwrap_or("non défini".to_string()));
    println!("🏭 [INIT] - TAURI_LED_MODE: {}", std::env::var("TAURI_LED_MODE").unwrap_or("non défini".to_string()));

    // NOTE: Auto-start audio removed - manual start only
    println!("🎧 [INIT] Audio system ready for manual start");

    // Auto-start LED en mode PRODUCTION
    let auto_led_state = Arc::clone(&app_state);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        println!("💡 [AUTO] Starting LED in mode: {}", final_mode);
        start_auto_led(auto_led_state);
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage((*app_state).clone())
        .setup(|app| {
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(2000));
                let _ = app_handle.emit("app_ready", serde_json::json!({
                    "status": "ready",
                    "message": "Application ready - PRODUCTION MODE"
                }));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Audio commands
            commands::audio_start_capture,
            commands::audio_stop_capture,
            commands::audio_get_spectrum,
            commands::audio_set_gain,
            commands::audio_get_gain,
            commands::audio_get_devices,
            commands::audio_get_status,
            commands::audio_test_input,

            // Effects commands
            commands::effects_get_list,
            commands::effects_set_current,
            commands::effects_set_by_name,
            commands::effects_get_current,
            commands::effects_get_info,
            commands::effects_set_color_mode,
            commands::effects_get_color_mode,
            commands::effects_set_custom_color,
            commands::effects_get_custom_color,
            commands::effects_reset_all,
            commands::effects_get_stats,

            // LED commands
            commands::led_start_output,
            commands::led_stop_output,
            commands::led_set_brightness,
            commands::led_get_brightness,
            commands::led_send_test_pattern,
            commands::led_get_status,
            commands::led_test_connectivity,
            commands::led_get_controllers,
            commands::led_get_test_patterns,
            commands::led_clear_display,
            commands::led_get_frame_data,

            // System commands
            commands::system_get_status,
            commands::system_get_performance,
            commands::system_restart_all,
            commands::system_get_config,
            commands::system_set_config,
            commands::system_get_diagnostics,
            commands::system_export_config,
            commands::system_import_config,
        ])
        .run(tauri::generate_context!())
        .expect("Error running application");
}

// LED auto-start logic - Mode intelligent selon environnement
fn start_auto_led(state: Arc<AppState>) {
    if *state.led_running.lock() {
        println!("💡 [AUTO] LED already running, skipping");
        return;
    }

    println!("💡 [AUTO] Creating LED controller...");

    // Détection intelligente du mode
    let mode = match std::env::var("LED_MODE").as_deref() {
        Ok("production") => {
            println!("🏭 [AUTO] Mode PRODUCTION via LED_MODE");
            *state.led_mode.lock() = "production".to_string();
            led::LedMode::Production
        }
        Ok("simulator") => {
            println!("🧪 [AUTO] Mode SIMULATOR via LED_MODE");
            *state.led_mode.lock() = "simulator".to_string();
            led::LedMode::Simulator
        }
        _ => match std::env::var("TAURI_LED_MODE").as_deref() {
            Ok("production") => {
                println!("🏭 [AUTO] Mode PRODUCTION via TAURI_LED_MODE");
                *state.led_mode.lock() = "production".to_string();
                led::LedMode::Production
            }
            Ok("simulator") => {
                println!("🧪 [AUTO] Mode SIMULATOR via TAURI_LED_MODE");
                *state.led_mode.lock() = "simulator".to_string();
                led::LedMode::Simulator
            }
            _ => {
                println!("🏭 [AUTO] Mode PRODUCTION par défaut");
                *state.led_mode.lock() = "production".to_string();
                led::LedMode::Production
            }
        }
    };

    println!("💡 [AUTO] LED Mode Final: {:?}", mode);

    // Create LED controller
    match led::LedController::new_with_mode(mode) {
        Ok(mut led_controller) => {
            println!("✅ [AUTO] LED controller created successfully");
            *state.led_running.lock() = true;

            let mut frame_count = 0u64;
            let start_time = std::time::Instant::now();

            println!("💡 [AUTO] LED loop started in PRODUCTION MODE");

            while *state.led_running.lock() {
                let frame = state.led_frame.lock().clone();
                let brightness = *state.led_brightness.lock();

                // Apply brightness if needed
                if brightness != 1.0 {
                    let adjusted_frame: Vec<u8> = frame.iter()
                        .map(|&byte| ((byte as f32) * brightness) as u8)
                        .collect();
                    led_controller.send_frame(&adjusted_frame);
                } else {
                    led_controller.send_frame(&frame);
                }

                frame_count += 1;

                // Performance stats
                if frame_count % 1000 == 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let fps = frame_count as f64 / elapsed;
                    let avg_brightness = frame.iter().map(|&b| b as u32).sum::<u32>() as f32 / frame.len() as f32;
                    println!("📊 [AUTO] LED {:?}: {:.1} FPS, Frame #{}, Avg brightness: {:.1}",
                             mode, fps, frame_count, avg_brightness);
                }

                std::thread::sleep(std::time::Duration::from_millis(13)); // ~77 FPS
            }

            println!("💡 [AUTO] LED loop stopped");
            let _ = led_controller.clear();
        }
        Err(e) => {
            println!("❌ [AUTO] LED controller creation failed: {}", e);
            state.set_error(format!("LED {:?} failed: {}", mode, e));
        }
    }
}
