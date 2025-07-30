// src-tauri/src/commands/led.rs

use crate::AppState;
use serde_json::json;
use tauri::{Emitter, State, Window};
use std::time::Duration;

#[tauri::command]
pub async fn led_start_output(
    window: Window,
    state: State<'_, AppState>,
    mode: String,
) -> Result<serde_json::Value, String> {
    println!("🚀 [LED] Start output requested - Mode: {}", mode);

    if *state.led_running.lock() {
        return Ok(json!({
            "status": "already_running",
            "mode": state.led_mode.lock().clone()
        }));
    }

    let led_mode = match mode.as_str() {
        "production" => {
            println!("🏭 [LED] Production mode activated");
            crate::led::LedMode::Production
        }
        _ => {
            println!("🧪 [LED] Simulator mode activated");
            crate::led::LedMode::Simulator
        }
    };

    *state.led_running.lock() = true;
    *state.led_mode.lock() = mode.clone();

    let app_state = state.inner().clone();
    let led_window = window.clone();
    let mode_clone = mode.clone();

    std::thread::spawn(move || {
        match crate::led::LedController::new_with_mode(led_mode) {
            Ok(mut led_controller) => {
                println!("✅ [LED] Controller created successfully");

                // Initial connectivity test
                if let Ok(results) = led_controller.test_connectivity() {
                    let successful = results.values().filter(|&&v| v).count();
                    let _ = led_window.emit("led_status", json!({
                        "status": "started",
                        "mode": mode_clone,
                        "controllers_tested": results.len(),
                        "controllers_working": successful
                    }));
                }

                let mut last_stats_emit = std::time::Instant::now();
                let mut frame_count = 0u64;

                while *app_state.led_running.lock() {
                    let frame = app_state.led_frame.lock().clone();
                    let brightness = *app_state.led_brightness.lock();

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

                    // Emit stats every 2 seconds
                    if last_stats_emit.elapsed() > Duration::from_millis(2000) {
                        let stats = led_controller.get_stats();
                        let _ = led_window.emit("led_stats", json!({
                            "fps": stats.fps,
                            "frames_sent": stats.frames_sent,
                            "packets_sent": stats.packets_sent,
                            "bytes_sent": stats.bytes_sent,
                            "frame_count": frame_count
                        }));
                        last_stats_emit = std::time::Instant::now();
                    }

                    std::thread::sleep(Duration::from_millis(crate::led::FRAME_TIME_MS));
                }

                // Clean shutdown
                println!("🛑 [LED] Stopping output");
                let _ = led_controller.clear();
                let _ = led_window.emit("led_status", json!({ "status": "stopped" }));
            }
            Err(e) => {
                println!("❌ [LED] Failed to create controller: {}", e);
                *app_state.led_running.lock() = false;
                let _ = led_window.emit("led_status", json!({
                    "status": "error",
                    "error": e
                }));
            }
        }
    });

    Ok(json!({
        "status": "starting",
        "mode": mode,
        "target_fps": crate::led::TARGET_FPS,
        "matrix_size": format!("{}x{}", crate::led::MATRIX_WIDTH, crate::led::MATRIX_HEIGHT)
    }))
}

#[tauri::command]
pub async fn led_stop_output(
    window: Window,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("🛑 [LED] Stop output requested");

    *state.led_running.lock() = false;

    let _ = window.emit("led_status", json!({ "status": "stopping" }));

    // Wait for thread to finish
    std::thread::sleep(Duration::from_millis(200));

    Ok(json!({ "status": "stopped" }))
}

#[tauri::command]
pub async fn led_set_brightness(
    window: Window,
    state: State<'_, AppState>,
    brightness: f32,
) -> Result<serde_json::Value, String> {
    crate::led::validate_brightness(brightness)
        .map_err(|e| format!("Invalid brightness: {}", e))?;

    *state.led_brightness.lock() = brightness;
    println!("💡 [LED] Brightness set to {:.1}%", brightness * 100.0);

    let _ = window.emit("led_brightness_changed", json!({
        "brightness": brightness,
        "percentage": brightness * 100.0
    }));

    Ok(json!({
        "brightness": brightness,
        "percentage": brightness * 100.0
    }))
}

#[tauri::command]
pub async fn led_get_brightness(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let brightness = *state.led_brightness.lock();
    Ok(json!({
        "brightness": brightness,
        "percentage": brightness * 100.0
    }))
}

#[tauri::command]
pub async fn led_send_test_pattern(
    window: Window,
    state: State<'_, AppState>,
    pattern: String,
    duration_ms: Option<u64>,
) -> Result<serde_json::Value, String> {
    let duration = duration_ms.unwrap_or(3000);
    println!("🎨 [LED] Test pattern '{}' for {} ms", pattern, duration);

    let original_frame = state.led_frame.lock().clone();

    let _ = window.emit("led_test_started", json!({
        "pattern": pattern,
        "duration_ms": duration
    }));

    // Create test pattern
    let test_frame = crate::led::create_test_pattern(
        &pattern,
        crate::led::MATRIX_WIDTH,
        crate::led::MATRIX_HEIGHT
    );

    *state.led_frame.lock() = test_frame;

    // Restore after delay
    let restore_state = state.inner().clone();
    let restore_window = window.clone();
    let restore_pattern = pattern.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(duration));
        *restore_state.led_frame.lock() = original_frame;

        let _ = restore_window.emit("led_test_completed", json!({
            "pattern": restore_pattern
        }));
    });

    Ok(json!({
        "status": "started",
        "pattern": pattern,
        "duration_ms": duration
    }))
}

#[tauri::command]
pub async fn led_get_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let running = *state.led_running.lock();
    let brightness = *state.led_brightness.lock();
    let mode = state.led_mode.lock().clone();
    let frame_size = state.led_frame.lock().len();

    Ok(json!({
        "running": running,
        "mode": mode,
        "brightness": brightness,
        "frame_size": frame_size,
        "matrix_size": format!("{}x{}", crate::led::MATRIX_WIDTH, crate::led::MATRIX_HEIGHT),
        "target_fps": crate::led::TARGET_FPS,
        "frame_time_ms": crate::led::FRAME_TIME_MS
    }))
}

#[tauri::command]
pub async fn led_test_connectivity(
    window: Window,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("🔍 [LED] Testing connectivity...");

    let mode_str = state.led_mode.lock().clone();
    let led_mode = match mode_str.as_str() {
        "production" => crate::led::LedMode::Production,
        _ => crate::led::LedMode::Simulator,
    };

    let _ = window.emit("led_connectivity_test_started", json!({ "mode": mode_str }));

    match crate::led::LedController::new_with_mode(led_mode) {
        Ok(mut controller) => {
            match controller.test_connectivity() {
                Ok(results) => {
                    let successful = results.values().filter(|&&v| v).count();

                    let _ = window.emit("led_connectivity_test_completed", json!({
                        "results": results,
                        "success": true
                    }));

                    Ok(json!({
                        "status": "success",
                        "results": results,
                        "total_controllers": results.len(),
                        "active_controllers": successful
                    }))
                }
                Err(e) => {
                    let _ = window.emit("led_connectivity_test_failed", json!({ "error": e }));
                    Err(format!("Connectivity test failed: {}", e))
                }
            }
        }
        Err(e) => {
            let _ = window.emit("led_connectivity_test_failed", json!({ "error": e }));
            Err(format!("Failed to create controller: {}", e))
        }
    }
}

#[tauri::command]
pub async fn led_get_controllers(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mode = state.led_mode.lock().clone();

    let controllers = if mode == "production" {
        vec![
            json!({"id": "controller_1", "ip": "192.168.1.45", "enabled": true}),
            json!({"id": "controller_2", "ip": "192.168.1.46", "enabled": true}),
            json!({"id": "controller_3", "ip": "192.168.1.47", "enabled": true}),
            json!({"id": "controller_4", "ip": "192.168.1.48", "enabled": true}),
        ]
    } else {
        vec![
            json!({"id": "simulator", "ip": "127.0.0.1", "enabled": true}),
        ]
    };

    Ok(json!({
        "mode": mode,
        "controllers": controllers,
        "total": controllers.len()
    }))
}

#[tauri::command]
pub async fn led_get_test_patterns() -> Result<serde_json::Value, String> {
    Ok(json!({
        "basic": [
            { "id": "red", "name": "Red", "description": "All LEDs red" },
            { "id": "green", "name": "Green", "description": "All LEDs green" },
            { "id": "blue", "name": "Blue", "description": "All LEDs blue" },
            { "id": "white", "name": "White", "description": "All LEDs white" },
            { "id": "off", "name": "Off", "description": "All LEDs off" }
        ],
        "patterns": [
            { "id": "gradient", "name": "Gradient", "description": "Horizontal gradient" },
            { "id": "checkerboard", "name": "Checkerboard", "description": "Checkerboard pattern" }
        ]
    }))
}

#[tauri::command]
pub async fn led_clear_display(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!("🧹 [LED] Clearing display");

    let black_frame = vec![0; crate::led::MATRIX_SIZE];
    *state.led_frame.lock() = black_frame;

    Ok(json!({
        "status": "cleared",
        "message": "Display cleared"
    }))
}

#[tauri::command]
pub async fn led_get_frame_data(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let frame = state.led_frame.lock().clone();
    let avg_brightness = frame.iter().map(|&b| b as u32).sum::<u32>() as f32 / frame.len() as f32;

    Ok(json!({
        "width": crate::led::MATRIX_WIDTH,
        "height": crate::led::MATRIX_HEIGHT,
        "format": "RGB",
        "data_size": frame.len(),
        "average_brightness": avg_brightness,
        "data": frame
    }))
}
