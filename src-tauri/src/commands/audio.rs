// src-tauri/src/commands/audio.rs

use crate::AppState;
use serde_json::json;
use tauri::{Emitter, State, Window};

#[tauri::command]
pub async fn audio_start_capture(
    window: Window,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("🎧 [AUDIO] Start capture requested");

    if *state.audio_running.lock() {
        return Ok(json!({
            "status": "already_running",
            "message": "Audio capture is already running"
        }));
    }

    *state.audio_running.lock() = true;

    let app_state = state.inner().clone();
    let audio_window = window.clone();

    std::thread::spawn(move || {
        let audio_state = app_state.clone();
        let window_clone = audio_window.clone();
        let mut callback_count = 0u64;

        let result = crate::audio::AudioCapture::new(move |data| {
            callback_count += 1;

            // Compute spectrum
            let spectrum = crate::audio::compute_spectrum(data);

            // Update shared state
            *audio_state.spectrum.lock() = spectrum.clone();

            // Generate LED frame
            let mut engine = audio_state.effect_engine.lock();
            let frame = engine.render(&spectrum);
            *audio_state.led_frame.lock() = frame;

            // Emit to frontend (throttled) - FIX: Send as array directly
            if callback_count % 2 == 0 {
                let _ = window_clone.emit("spectrum_update", json!({
                    "spectrum": spectrum,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                }));
            }

            // Debug log
            if callback_count % 1000 == 0 {
                let max_val = spectrum.iter().cloned().fold(0.0, f32::max);
                println!("🔊 [AUDIO] Callback #{}: max spectrum = {:.3}", callback_count, max_val);
            }
        });

        match result {
            Ok(_audio) => {
                println!("✅ [AUDIO] Capture started successfully");
                let _ = audio_window.emit("audio_status", json!({
                    "status": "started",
                    "message": "Audio capture started"
                }));

                // Keep thread alive
                while *app_state.audio_running.lock() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                println!("🛑 [AUDIO] Capture thread stopping");
            }
            Err(e) => {
                eprintln!("❌ [AUDIO] Failed to start: {}", e);
                *app_state.audio_running.lock() = false;
                let _ = audio_window.emit("audio_status", json!({
                    "status": "error",
                    "message": format!("Failed to start audio: {}", e)
                }));
            }
        }
    });

    Ok(json!({
        "status": "starting",
        "message": "Audio capture is starting..."
    }))
}

#[tauri::command]
pub async fn audio_stop_capture(
    window: Window,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("🛑 [AUDIO] Stop capture requested");

    *state.audio_running.lock() = false;

    // Clear spectrum data
    *state.spectrum.lock() = Vec::new();

    let _ = window.emit("audio_status", json!({
        "status": "stopped",
        "message": "Audio capture stopped"
    }));

    Ok(json!({
        "status": "stopped",
        "message": "Audio capture stopped"
    }))
}

#[tauri::command]
pub async fn audio_get_spectrum(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let spectrum = state.spectrum.lock().clone();
    let max_val = spectrum.iter().cloned().fold(0.0, f32::max);
    let avg_val = if spectrum.is_empty() {
        0.0
    } else {
        spectrum.iter().sum::<f32>() / spectrum.len() as f32
    };

    Ok(json!({
        "spectrum": spectrum,
        "max": max_val,
        "average": avg_val,
        "size": spectrum.len()
    }))
}

#[tauri::command]
pub async fn audio_set_gain(
    window: Window,
    state: State<'_, AppState>,
    gain: f32,
) -> Result<serde_json::Value, String> {
    if !(0.1..=5.0).contains(&gain) {
        return Err("Gain must be between 0.1 and 5.0".to_string());
    }

    *state.audio_gain.lock() = gain;
    println!("🔊 [AUDIO] Gain set to {:.1}", gain);

    // Emit gain change event to frontend
    let _ = window.emit("audio_gain_changed", json!({
        "gain": gain,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }));

    Ok(json!({
        "gain": gain,
        "message": format!("Audio gain set to {:.1}", gain)
    }))
}

#[tauri::command]
pub async fn audio_get_gain(
    state: State<'_, AppState>,
) -> Result<f32, String> {
    Ok(*state.audio_gain.lock())
}

#[tauri::command]
pub async fn audio_get_devices() -> Result<serde_json::Value, String> {
    use cpal::traits::{HostTrait, DeviceTrait};

    let host = cpal::default_host();

    // Get default input device for comparison
    let default_device = host.default_input_device();
    let default_name = default_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let devices: Result<Vec<_>, _> = host
        .input_devices()
        .map_err(|e| format!("Failed to get devices: {}", e))?
        .enumerate()
        .map(|(index, device)| {
            let name = device.name().unwrap_or_else(|_| format!("Unknown Device {}", index));
            let is_default = name == default_name;

            Ok(json!({
                "name": name,
                "index": index,
                "is_default": is_default
            }))
        })
        .collect();

    let device_list = devices.map_err(|e: std::io::Error| e.to_string())?;

    Ok(json!({
        "devices": device_list,
        "count": device_list.len(),
        "default_device": default_name
    }))
}



#[tauri::command]
pub async fn audio_get_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let running = *state.audio_running.lock();
    let gain = *state.audio_gain.lock();
    let spectrum = state.spectrum.lock();
    let spectrum_size = spectrum.len();
    let has_signal = !spectrum.is_empty() && spectrum.iter().any(|&x| x > 0.001);

    Ok(json!({
        "running": running,
        "gain": gain,
        "spectrum_size": spectrum_size,
        "has_signal": has_signal
    }))
}

#[tauri::command]
pub async fn audio_test_input() -> Result<serde_json::Value, String> {
    println!("🔍 [AUDIO] Testing audio input...");

    // List available devices for diagnostics
    match crate::audio::AudioCapture::list_devices() {
        Ok(_) => {
            Ok(json!({
                "status": "success",
                "message": "Audio devices listed successfully in console"
            }))
        }
        Err(e) => {
            Err(format!("Failed to list devices: {}", e))
        }
    }
}
