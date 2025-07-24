use crate::AppState;
use serde_json::json;
use tauri::{Emitter, State, Window};

#[tauri::command]
pub async fn get_audio_devices() -> Result<Vec<String>, String> {
    println!("🎤 [CMD] Récupération des périphériques audio...");

    use cpal::traits::{HostTrait, DeviceTrait};

    let host = cpal::default_host();
    let devices: Result<Vec<String>, _> = host
        .input_devices()
        .map_err(|e| format!("Failed to get input devices: {}", e))?
        .map(|device| device.name().map_err(|e| format!("Failed to get device name: {}", e)))
        .collect();

    let device_list = devices.map_err(|e| e.to_string())?;
    println!("🎤 [CMD] Trouvé {} périphériques", device_list.len());

    Ok(device_list)
}

#[tauri::command]
pub async fn start_audio_capture(
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    println!("🎧 [CMD] Commande de démarrage audio reçue");

    let is_running = *state.audio_running.lock();
    if is_running {
        println!("🎧 [CMD] Audio déjà en cours");
        return Ok("Audio capture already running".to_string());
    }

    *state.audio_running.lock() = true;

    // Clone la AppState directement
    let app_state = state.inner().clone();
    let window_for_thread = window.clone();

    std::thread::spawn(move || {
        println!("🎧 [CMD] Thread audio démarré");

        let audio_state = app_state.clone();
        let window_clone = window_for_thread.clone();
        let mut callback_count = 0u64;

        let result = crate::audio::AudioCapture::new(move |data| {
            callback_count += 1;

            // Log périodique
            if callback_count % 500 == 0 {
                println!("🔊 [CMD] Audio callback #{}: {} échantillons",
                         callback_count, data.len());
            }

            let spectrum = crate::audio::compute_spectrum(data);

            // Update shared state
            *audio_state.spectrum.lock() = spectrum.clone();

            // Emit spectrum data to frontend
            let _ = window_clone.emit("spectrum_data", &spectrum);

            // Update LED frame
            let mut engine = audio_state.effect_engine.lock();
            let frame = engine.render(&spectrum);
            *audio_state.led_frame.lock() = frame;

            // Emit frame data to frontend
            let _ = window_clone.emit("frame_data", json!({
                "width": 128,
                "height": 128,
                "format": 1, // RGB
                "data": audio_state.led_frame.lock().clone()
            }));
        });

        match result {
            Ok(_audio) => {
                println!("✅ [CMD] Audio capture démarré avec succès");
                let _ = window_for_thread.emit("audio_status", json!({
                    "status": "started",
                    "message": "Audio capture started"
                }));

                // Maintenir le stream vivant
                loop {
                    if !*app_state.audio_running.lock() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            Err(e) => {
                eprintln!("❌ [CMD] Échec démarrage audio: {}", e);
                let _ = window_for_thread.emit("audio_status", json!({
                    "status": "error",
                    "message": format!("Failed to start audio: {}", e)
                }));
                *app_state.audio_running.lock() = false;
            }
        }
    });

    Ok("Audio capture starting...".to_string())
}

#[tauri::command]
pub async fn stop_audio_capture(state: State<'_, AppState>) -> Result<String, String> {
    println!("🛑 [CMD] Arrêt audio demandé");
    *state.audio_running.lock() = false;
    Ok("Audio capture stopped".to_string())
}

#[tauri::command]
pub async fn get_current_spectrum(state: State<'_, AppState>) -> Result<Vec<f32>, String> {
    Ok(state.spectrum.lock().clone())
}

#[tauri::command]
pub async fn set_audio_gain(
    state: State<'_, AppState>,
    gain: f32,
) -> Result<String, String> {
    if !(0.1..=5.0).contains(&gain) {
        return Err("Gain must be between 0.1 and 5.0".to_string());
    }

    *state.audio_gain.lock() = gain;
    println!("🔊 [CMD] Gain audio défini à {:.1}", gain);
    Ok(format!("Audio gain set to {:.1}", gain))
}

#[tauri::command]
pub async fn get_audio_gain(state: State<'_, AppState>) -> Result<f32, String> {
    Ok(*state.audio_gain.lock())
}
