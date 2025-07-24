use crate::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub async fn get_available_effects() -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![
        json!({
            "id": 0,
            "name": "SpectrumBars",
            "display_name": "Spectrum Bars",
            "description": "Classic spectrum analyzer bars"
        }),
        json!({
            "id": 1,
            "name": "CircularWave",
            "display_name": "Circular Wave",
            "description": "Circular ripple effect from center"
        }),
        json!({
            "id": 2,
            "name": "ParticleSystem",
            "display_name": "Particle System",
            "description": "Dynamic particle effects"
        }),
        json!({
            "id": 3,
            "name": "Heartbeat",
            "display_name": "Heartbeat",
            "description": "Pulsing heartbeat effect"
        }),
        json!({
            "id": 4,
            "name": "Starfall",
            "display_name": "Starfall",
            "description": "Falling stars effect"
        }),
        json!({
            "id": 5,
            "name": "Rain",
            "display_name": "Rain",
            "description": "Rain drops effect"
        }),
        json!({
            "id": 6,
            "name": "Flames",
            "display_name": "Flames",
            "description": "Fire simulation"
        }),
        json!({
            "id": 7,
            "name": "Applaudimetre",
            "display_name": "Applaudimètre",
            "description": "Applause meter with peak detection"
        }),
    ])
}

#[tauri::command]
pub async fn set_effect(
    state: State<'_, AppState>,
    effect_id: usize,
) -> Result<String, String> {
    let mut engine = state.effect_engine.lock();

    // Validation de l'ID
    if effect_id >= 8 {
        return Err(format!("Invalid effect ID: {}. Valid range: 0-7", effect_id));
    }

    // Changer l'effet
    match engine.set_effect(effect_id) {
        Ok(_) => {
            let effect_name = engine.get_current_effect_name();
            println!("✅ Effect changed to ID {} ({})", effect_id, effect_name);
            Ok(format!("Effect set to: {} (ID: {})", effect_name, effect_id))
        }
        Err(e) => {
            eprintln!("❌ Failed to set effect {}: {}", effect_id, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn get_current_effect(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let engine = state.effect_engine.lock();
    let current_id = engine.get_current_effect();
    let current_name = engine.get_current_effect_name();

    Ok(json!({
        "id": current_id,
        "name": current_name,
        "transitioning": engine.get_stats().transitioning,
        "transition_progress": engine.get_stats().transition_progress
    }))
}

#[tauri::command]
pub async fn set_color_mode(
    state: State<'_, AppState>,
    mode: String,
) -> Result<String, String> {
    // Validation du mode couleur
    let valid_modes = &["rainbow", "fire", "ocean", "sunset", "custom"];
    if !valid_modes.contains(&mode.as_str()) {
        return Err(format!(
            "Invalid color mode '{}'. Valid modes: {}",
            mode,
            valid_modes.join(", ")
        ));
    }

    let mut engine = state.effect_engine.lock();
    engine.set_color_mode(&mode);

    println!("🎨 Color mode changed to: {}", mode);
    Ok(format!("Color mode set to: {}", mode))
}

#[tauri::command]
pub async fn get_color_mode(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let engine = state.effect_engine.lock();
    let config = engine.get_color_config();

    Ok(json!({
        "mode": config.mode,
        "custom_color": {
            "r": config.custom_color.0,
            "g": config.custom_color.1,
            "b": config.custom_color.2
        },
        "available_modes": ["rainbow", "fire", "ocean", "sunset", "custom"]
    }))
}

#[tauri::command]
pub async fn set_custom_color(
    state: State<'_, AppState>,
    r: f32,
    g: f32,
    b: f32,
) -> Result<String, String> {
    // Validation des valeurs
    if !(0.0..=1.0).contains(&r) || !(0.0..=1.0).contains(&g) || !(0.0..=1.0).contains(&b) {
        return Err(format!(
            "Color values must be between 0.0 and 1.0. Got: R={:.3}, G={:.3}, B={:.3}",
            r, g, b
        ));
    }

    let mut engine = state.effect_engine.lock();
    engine.set_custom_color(r, g, b);

    println!("🎨 Custom color set to RGB({:.2}, {:.2}, {:.2})", r, g, b);
    Ok(format!("Custom color set to RGB({:.2}, {:.2}, {:.2})", r, g, b))
}

#[tauri::command]
pub async fn get_custom_color(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let engine = state.effect_engine.lock();
    let (r, g, b) = engine.get_custom_color();

    Ok(json!({
        "r": r,
        "g": g,
        "b": b,
        "hex": format!("#{:02X}{:02X}{:02X}",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        )
    }))
}


#[tauri::command]
pub async fn get_effect_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let engine = state.effect_engine.lock();
    let stats = engine.get_stats();

    Ok(json!({
        "current_effect": {
            "id": stats.current_effect,
            "name": stats.current_effect_name,
        },
        "total_effects": stats.total_effects,
        "transition": {
            "active": stats.transitioning,
            "progress": stats.transition_progress
        },
        "color": {
            "mode": stats.color_mode,
            "custom_color": {
                "r": stats.custom_color.0,
                "g": stats.custom_color.1,
                "b": stats.custom_color.2
            }
        }
    }))
}

#[tauri::command]
pub async fn set_effect_by_name(
    state: State<'_, AppState>,
    effect_name: String,
) -> Result<serde_json::Value, String> {
    let mut engine = state.effect_engine.lock();

    match engine.set_effect_by_name(&effect_name) {
        Ok(_) => {
            let current_id = engine.get_current_effect();
            let current_name = engine.get_current_effect_name();

            println!("✅ Effect changed to '{}' (ID: {})", current_name, current_id);
            Ok(json!({
                "id": current_id,
                "name": current_name,
                "message": format!("Effect set to: {}", current_name)
            }))
        }
        Err(e) => {
            eprintln!("❌ Failed to set effect '{}': {}", effect_name, e);
            Err(e)
        }
    }
}

// Commandes héritées pour compatibilité (avec implémentations améliorées)
#[tauri::command]
pub async fn set_effect_parameter(
    state: State<'_, AppState>,
    parameter: String,
    value: f32,
) -> Result<String, String> {
    // TODO: Implémenter la configuration des paramètres d'effet
    // Pour l'instant, on log juste l'appel
    let engine = state.effect_engine.lock();
    let current_effect = engine.get_current_effect_name();

    println!("🔧 Setting parameter '{}' to {:.2} for effect '{}'", parameter, value, current_effect);
    Ok(format!("Parameter '{}' set to {:.2} for effect '{}'", parameter, value, current_effect))
}

#[tauri::command]
pub async fn get_effect_parameter(
    state: State<'_, AppState>,
    parameter: String,
) -> Result<serde_json::Value, String> {
    // TODO: Récupérer la valeur du paramètre d'effet
    let engine = state.effect_engine.lock();
    let current_effect = engine.get_current_effect_name();

    println!("🔍 Getting parameter '{}' for effect '{}'", parameter, current_effect);
    Ok(json!({
        "parameter": parameter,
        "value": 1.0,
        "effect": current_effect,
        "note": "Parameter system not yet implemented"
    }))
}

#[tauri::command]
pub async fn get_current_frame(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let frame_data = state.led_frame.lock().clone();
    let frame_size = frame_data.len();

    Ok(json!({
        "width": 128,
        "height": 128,
        "format": "RGB",
        "data_size": frame_size,
        "data": frame_data,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }))
}

#[tauri::command]
pub async fn get_effect_info(
    effect_id: usize,
) -> Result<serde_json::Value, String> {
    let effects_info = vec![
        ("SpectrumBars", "Classic frequency spectrum visualization with vertical bars"),
        ("CircularWave", "Circular ripple effects emanating from the center"),
        ("ParticleSystem", "Dynamic particle effects with physics simulation"),
        ("Heartbeat", "Pulsing heartbeat effect synchronized with audio"),
        ("Starfall", "Falling stars effect with sparkle trails"),
        ("Rain", "Realistic rain drops with splash effects"),
        ("Flames", "Fire simulation with heat distortion"),
        ("Applaudimetre", "Advanced applause meter with peak detection and sparkles"),
    ];

    if effect_id >= effects_info.len() {
        return Err(format!("Effect ID {} not found", effect_id));
    }

    let (name, description) = effects_info[effect_id];
    Ok(json!({
        "id": effect_id,
        "name": name,
        "description": description,
        "supports_transitions": effect_id != 2, // ParticleSystem doesn't support transitions yet
        "supports_custom_colors": true,
        "performance_impact": match effect_id {
            2 => "high",    // ParticleSystem
            6 => "high",    // Flames
            7 => "medium",  // Applaudimetre
            _ => "low"
        }
    }))
}
