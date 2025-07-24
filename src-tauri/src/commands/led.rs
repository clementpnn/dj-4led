use crate::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub async fn start_led_output(
    state: State<'_, AppState>,
    mode: String, // "simulator" or "production"
) -> Result<String, String> {
    let is_running = *state.led_running.lock();
    if is_running {
        return Ok("LED output already running".to_string());
    }

    *state.led_running.lock() = true;

    let led_mode = match mode.as_str() {
        "production" => crate::led::LedMode::Production,
        _ => crate::led::LedMode::Simulator,
    };

    let mode_clone = mode.clone();
    // FIX: Clone la AppState directement
    let app_state = state.inner().clone();

    std::thread::spawn(move || {
        match crate::led::LedController::new_with_mode(led_mode) {
            Ok(mut led) => {
                println!("LED controller started in {} mode", mode_clone);
                let mut frame_count = 0u64;
                let start_time = std::time::Instant::now();

                loop {
                    if !*app_state.led_running.lock() {
                        break;
                    }

                    let frame = app_state.led_frame.lock().clone();
                    if let Err(e) = led.send_frame(&frame) {
                        eprintln!("Error sending frame: {}", e);
                    }

                    frame_count += 1;
                    if frame_count % 100 == 0 {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let fps = frame_count as f64 / elapsed;
                        println!("LED FPS: {:.1}", fps);
                    }

                    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
                }

                println!("LED output stopped");
            }
            Err(e) => {
                eprintln!("Failed to start LED controller: {}", e);
                *app_state.led_running.lock() = false;
            }
        }
    });

    Ok(format!("LED output started in {} mode", mode))
}

#[tauri::command]
pub async fn stop_led_output(state: State<'_, AppState>) -> Result<String, String> {
    *state.led_running.lock() = false;
    Ok("LED output stopped".to_string())
}

#[tauri::command]
pub async fn is_led_running(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.led_running.lock())
}

#[tauri::command]
pub async fn set_led_brightness(
    state: State<'_, AppState>,
    brightness: f32,
) -> Result<String, String> {
    if !(0.0..=1.0).contains(&brightness) {
        return Err("Brightness must be between 0.0 and 1.0".to_string());
    }

    *state.led_brightness.lock() = brightness;
    Ok(format!("LED brightness set to {:.1}%", brightness * 100.0))
}

#[tauri::command]
pub async fn get_led_brightness(state: State<'_, AppState>) -> Result<f32, String> {
    Ok(*state.led_brightness.lock())
}

#[tauri::command]
pub async fn test_led_pattern(
    state: State<'_, AppState>,
    pattern: String,
) -> Result<String, String> {
    let test_pattern = match pattern.as_str() {
        "red" => crate::led::TestPattern::AllRed,
        "green" => crate::led::TestPattern::AllGreen,
        "blue" => crate::led::TestPattern::AllBlue,
        "white" => crate::led::TestPattern::AllWhite,
        "gradient" => crate::led::TestPattern::Gradient,
        "checkerboard" => crate::led::TestPattern::Checkerboard,
        "quarter" => crate::led::TestPattern::QuarterTest,
        _ => return Err("Unknown pattern. Available: red, green, blue, white, gradient, checkerboard, quarter".to_string()),
    };

    // Générer la frame de test
    let frame = generate_test_frame_manual(test_pattern);
    *state.led_frame.lock() = frame;

    Ok(format!("Test pattern '{}' applied", pattern))
}

// Fonction helper pour générer les frames de test manuellement
fn generate_test_frame_manual(pattern: crate::led::TestPattern) -> Vec<u8> {
    let mut frame = vec![0; 128 * 128 * 3];

    match pattern {
        crate::led::TestPattern::AllRed => {
            for i in (0..frame.len()).step_by(3) {
                frame[i] = 255; // Red
            }
        }
        crate::led::TestPattern::AllGreen => {
            for i in (1..frame.len()).step_by(3) {
                frame[i] = 255; // Green
            }
        }
        crate::led::TestPattern::AllBlue => {
            for i in (2..frame.len()).step_by(3) {
                frame[i] = 255; // Blue
            }
        }
        crate::led::TestPattern::AllWhite => {
            frame.fill(255); // All channels
        }
        crate::led::TestPattern::Gradient => {
            for y in 0..128 {
                for x in 0..128 {
                    let i = (y * 128 + x) * 3;
                    frame[i] = (x * 255 / 128) as u8;     // Red gradient
                    frame[i + 1] = (y * 255 / 128) as u8; // Green gradient
                    frame[i + 2] = 128;                    // Blue constant
                }
            }
        }
        crate::led::TestPattern::Checkerboard => {
            for y in 0..128 {
                for x in 0..128 {
                    let i = (y * 128 + x) * 3;
                    let is_white = (x / 8 + y / 8) % 2 == 0;
                    let value = if is_white { 255 } else { 0 };
                    frame[i] = value;     // R
                    frame[i + 1] = value; // G
                    frame[i + 2] = value; // B
                }
            }
        }
        crate::led::TestPattern::QuarterTest => {
            // Chaque quart de l'écran en couleur différente
            for y in 0..128 {
                for x in 0..128 {
                    let i = (y * 128 + x) * 3;

                    let (r, g, b) = match (x < 64, y < 64) {
                        (true, true) => (255, 0, 0),   // Haut-gauche: Rouge
                        (false, true) => (0, 255, 0),  // Haut-droite: Vert
                        (true, false) => (0, 0, 255),  // Bas-gauche: Bleu
                        (false, false) => (255, 255, 0), // Bas-droite: Jaune
                    };

                    frame[i] = r;
                    frame[i + 1] = g;
                    frame[i + 2] = b;
                }
            }
        }
    }

    frame
}

#[tauri::command]
pub async fn get_led_controllers() -> Result<Vec<String>, String> {
    Ok(vec![
        "192.168.1.45:6454".to_string(),
        "192.168.1.46:6454".to_string(),
        "192.168.1.47:6454".to_string(),
        "192.168.1.48:6454".to_string(),
    ])
}

#[tauri::command]
pub async fn get_led_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let is_running = *state.led_running.lock();
    let brightness = *state.led_brightness.lock();
    let frame_size = state.led_frame.lock().len();

    Ok(json!({
        "is_running": is_running,
        "brightness": brightness,
        "frame_size": frame_size,
        "matrix_size": "128x128",
        "controllers": 4,
        "mode": if is_running { "active" } else { "stopped" }
    }))
}
