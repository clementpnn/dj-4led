// src-tauri/src/commands/system.rs

use crate::AppState;
use serde_json::json;
use tauri::{Emitter, State, Window};

#[tauri::command]
pub async fn system_get_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let audio_running = *state.audio_running.lock();
    let led_running = *state.led_running.lock();
    let led_mode = state.led_mode.lock().clone();
    let audio_gain = *state.audio_gain.lock();
    let led_brightness = *state.led_brightness.lock();
    let spectrum_size = state.spectrum.lock().len();

    let engine = state.effect_engine.lock();
    let effect_stats = engine.get_stats();

    Ok(json!({
        "system": {
            "uptime": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "version": env!("CARGO_PKG_VERSION"),
            "name": env!("CARGO_PKG_NAME")
        },
        "audio": {
            "running": audio_running,
            "gain": audio_gain,
            "spectrum_size": spectrum_size,
            "fft_size": 1024,
            "sample_rate": 44100
        },
        "led": {
            "running": led_running,
            "mode": led_mode,
            "brightness": led_brightness,
            "matrix_size": format!("{}x{}", crate::led::MATRIX_WIDTH, crate::led::MATRIX_HEIGHT),
            "target_fps": crate::led::TARGET_FPS
        },
        "effects": {
            "current_id": effect_stats.current_effect,
            "current_name": effect_stats.current_effect_name,
            "total_effects": effect_stats.total_effects,
            "transitioning": effect_stats.transitioning,
            "color_mode": effect_stats.color_mode
        }
    }))
}

#[tauri::command]
pub async fn system_get_performance() -> Result<serde_json::Value, String> {
    // Version simplifiée sans ThreadId qui cause le problème de sérialisation
    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let memory_info = if cfg!(target_os = "linux") {
        match std::fs::read_to_string("/proc/meminfo") {
            Ok(content) => {
                let total_line = content.lines()
                    .find(|line| line.starts_with("MemTotal:"))
                    .unwrap_or("MemTotal: 0 kB");
                let available_line = content.lines()
                    .find(|line| line.starts_with("MemAvailable:"))
                    .unwrap_or("MemAvailable: 0 kB");

                format!("{}, {}", total_line, available_line)
            }
            Err(_) => "Memory info not available".to_string()
        }
    } else {
        "Memory monitoring not available on this platform".to_string()
    };

    Ok(json!({
        "cpu": {
            "cores": cpu_cores,
            "usage_percent": 0.0,
            "temperature": null
        },
        "memory": {
            "info": memory_info,
            "usage_mb": 0,
            "available_mb": 0
        },
        "system": {
            "load_average": [0.0, 0.0, 0.0],
            "disk_usage_percent": 0.0
        },
        "application": {
            "threads_count": cpu_cores,
            "memory_usage_mb": 0
        }
    }))
}

#[tauri::command]
pub async fn system_restart_all(
    window: Window,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("🔄 [SYSTEM] Restarting all systems...");

    let _ = window.emit("system_restart_started", json!({}));

    // Stop everything first
    *state.audio_running.lock() = false;
    *state.led_running.lock() = false;

    // Wait for threads to stop
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Reset effect engine
    {
        let mut engine = state.effect_engine.lock();
        engine.reset_all_effects();
    }

    // Clear LED frame
    *state.led_frame.lock() = vec![0; crate::led::MATRIX_SIZE];

    // Clear spectrum
    *state.spectrum.lock() = vec![0.0; 64];

    println!("✅ [SYSTEM] All systems restarted");

    let _ = window.emit("system_restart_completed", json!({}));

    Ok(json!({
        "status": "restarted",
        "message": "All systems have been restarted",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }))
}

#[tauri::command]
pub async fn system_get_config(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let engine = state.effect_engine.lock();
    let color_config = engine.get_color_config();

    Ok(json!({
        "audio": {
            "gain": *state.audio_gain.lock(),
            "fft_size": 1024,
            "spectrum_bins": 64,
            "sample_rate": 44100
        },
        "led": {
            "brightness": *state.led_brightness.lock(),
            "mode": state.led_mode.lock().clone(),
            "matrix_width": crate::led::MATRIX_WIDTH,
            "matrix_height": crate::led::MATRIX_HEIGHT,
            "target_fps": crate::led::TARGET_FPS
        },
        "effects": {
            "color_mode": color_config.mode,
            "custom_color": {
                "r": color_config.custom_color.0,
                "g": color_config.custom_color.1,
                "b": color_config.custom_color.2
            }
        }
    }))
}

#[tauri::command]
pub async fn system_set_config(
    window: Window,
    state: State<'_, AppState>,
    config: serde_json::Value,
) -> Result<serde_json::Value, String> {
    println!("⚙️ [SYSTEM] Applying configuration...");

    // Apply audio config if present
    if let Some(audio_config) = config.get("audio") {
        if let Some(gain) = audio_config.get("gain").and_then(|v| v.as_f64()) {
            let gain = gain as f32;
            if (0.1..=5.0).contains(&gain) {
                *state.audio_gain.lock() = gain;
                println!("🔊 [SYSTEM] Audio gain set to {:.1}", gain);
            }
        }
    }

    // Apply LED config if present
    if let Some(led_config) = config.get("led") {
        if let Some(brightness) = led_config.get("brightness").and_then(|v| v.as_f64()) {
            let brightness = brightness as f32;
            if (0.0..=1.0).contains(&brightness) {
                *state.led_brightness.lock() = brightness;
                println!("💡 [SYSTEM] LED brightness set to {:.1}%", brightness * 100.0);
            }
        }

        if let Some(mode) = led_config.get("mode").and_then(|v| v.as_str()) {
            *state.led_mode.lock() = mode.to_string();
            println!("🔧 [SYSTEM] LED mode set to {}", mode);
        }
    }

    // Apply effects config if present
    if let Some(effects_config) = config.get("effects") {
        let mut engine = state.effect_engine.lock();

        if let Some(color_mode) = effects_config.get("color_mode").and_then(|v| v.as_str()) {
            engine.set_color_mode(color_mode);
            println!("🎨 [SYSTEM] Color mode set to {}", color_mode);
        }

        if let Some(custom_color) = effects_config.get("custom_color") {
            if let (Some(r), Some(g), Some(b)) = (
                custom_color.get("r").and_then(|v| v.as_f64()),
                custom_color.get("g").and_then(|v| v.as_f64()),
                custom_color.get("b").and_then(|v| v.as_f64()),
            ) {
                let (r, g, b) = (r as f32, g as f32, b as f32);
                if (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b) {
                    engine.set_custom_color(r, g, b);
                    println!("🎨 [SYSTEM] Custom color set to RGB({:.2}, {:.2}, {:.2})", r, g, b);
                }
            }
        }
    }

    let _ = window.emit("system_config_updated", json!({}));

    Ok(json!({
        "status": "applied",
        "message": "Configuration applied successfully"
    }))
}

#[tauri::command]
pub async fn system_get_diagnostics(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("🔍 [SYSTEM] Running diagnostics...");

    let audio_running = *state.audio_running.lock();
    let led_running = *state.led_running.lock();
    let spectrum = state.spectrum.lock();
    let frame = state.led_frame.lock();

    // Audio diagnostics
    let audio_diagnostics = json!({
        "status": if audio_running { "running" } else { "stopped" },
        "spectrum_active": spectrum.iter().any(|&x| x > 0.01),
        "spectrum_max": spectrum.iter().cloned().fold(0.0, f32::max),
        "spectrum_avg": spectrum.iter().sum::<f32>() / spectrum.len() as f32
    });

    // LED diagnostics
    let frame_avg = frame.iter().map(|&b| b as u32).sum::<u32>() as f32 / frame.len() as f32;
    let led_diagnostics = json!({
        "status": if led_running { "running" } else { "stopped" },
        "frame_active": frame_avg > 1.0,
        "frame_avg_brightness": frame_avg,
        "frame_size": frame.len()
    });

    // Effects diagnostics
    let engine = state.effect_engine.lock();
    let effect_stats = engine.get_stats();
    let effects_diagnostics = json!({
        "current_effect": effect_stats.current_effect_name,
        "transitioning": effect_stats.transitioning,
        "color_mode": effect_stats.color_mode
    });

    let overall_health = if audio_running && led_running { "healthy" } else { "partial" };

    Ok(json!({
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        "overall_status": overall_health,
        "audio": audio_diagnostics,
        "led": led_diagnostics,
        "effects": effects_diagnostics,
        "recommendations": if !audio_running {
            vec!["Start audio capture for real-time visualization"]
        } else if !led_running {
            vec!["Start LED output to see effects"]
        } else {
            vec![]
        }
    }))
}

#[tauri::command]
pub async fn system_export_config(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = system_get_config(state).await?;

    let export_data = json!({
        "version": "1.0",
        "exported_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        "config": config
    });

    Ok(export_data)
}

#[tauri::command]
pub async fn system_import_config(
    window: Window,
    state: State<'_, AppState>,
    config_data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    println!("📥 [SYSTEM] Importing configuration...");

    // Validate config format
    if let Some(config) = config_data.get("config") {
        system_set_config(window, state, config.clone()).await
    } else {
        Err("Invalid configuration format".to_string())
    }
}
