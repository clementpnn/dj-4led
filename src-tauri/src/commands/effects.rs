// src-tauri/src/commands/effects.rs

use crate::AppState;
use serde_json::json;
use tauri::{Emitter, State, Window};

#[tauri::command]
pub async fn effects_get_list() -> Result<serde_json::Value, String> {
    Ok(json!({
        "effects": [
            { "id": 0, "name": "SpectrumBars", "display_name": "Spectrum Bars", "description": "Classic spectrum analyzer bars" },
            { "id": 1, "name": "CircularWave", "display_name": "Circular Wave", "description": "Circular ripple effect from center" },
            { "id": 2, "name": "ParticleSystem", "display_name": "Particle System", "description": "Dynamic particle effects" },
            { "id": 3, "name": "Heartbeat", "display_name": "Heartbeat", "description": "Pulsing heartbeat effect" },
            { "id": 4, "name": "Starfall", "display_name": "Starfall", "description": "Falling stars effect" },
            { "id": 5, "name": "Rain", "display_name": "Rain", "description": "Rain drops effect" },
            { "id": 6, "name": "Flames", "display_name": "Flames", "description": "Fire simulation" },
            { "id": 7, "name": "Applaudimetre", "display_name": "Applaudimètre", "description": "Applause meter with peak detection" }
        ]
    }))
}

#[tauri::command]
pub async fn effects_set_current(
    window: Window,
    state: State<'_, AppState>,
    effect_id: usize,
) -> Result<serde_json::Value, String> {
    if effect_id >= 8 {
        return Err(format!("Invalid effect ID: {}. Valid range: 0-7", effect_id));
    }

    let mut engine = state.effect_engine.lock();

    match engine.set_effect(effect_id) {
        Ok(_) => {
            let effect_name = engine.get_current_effect_name();
            println!("✅ [EFFECTS] Changed to {} (ID: {})", effect_name, effect_id);

            let _ = window.emit("effect_changed", json!({
                "id": effect_id,
                "name": effect_name
            }));

            Ok(json!({
                "id": effect_id,
                "name": effect_name,
                "message": format!("Effect changed to {}", effect_name)
            }))
        }
        Err(e) => {
            eprintln!("❌ [EFFECTS] Failed to set effect {}: {}", effect_id, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn effects_set_by_name(
    window: Window,
    state: State<'_, AppState>,
    effect_name: String,
) -> Result<serde_json::Value, String> {
    let mut engine = state.effect_engine.lock();

    match engine.set_effect_by_name(&effect_name) {
        Ok(_) => {
            let current_id = engine.get_current_effect();
            let current_name = engine.get_current_effect_name();

            println!("✅ [EFFECTS] Changed to '{}' (ID: {})", current_name, current_id);

            let _ = window.emit("effect_changed", json!({
                "id": current_id,
                "name": current_name
            }));

            Ok(json!({
                "id": current_id,
                "name": current_name,
                "message": format!("Effect set to {}", current_name)
            }))
        }
        Err(e) => {
            eprintln!("❌ [EFFECTS] Failed to set effect '{}': {}", effect_name, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn effects_get_current(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let engine = state.effect_engine.lock();
    let stats = engine.get_stats();

    Ok(json!({
        "id": stats.current_effect,
        "name": stats.current_effect_name,
        "transitioning": stats.transitioning,
        "transition_progress": stats.transition_progress
    }))
}

#[tauri::command]
pub async fn effects_get_info(
    effect_id: usize,
) -> Result<serde_json::Value, String> {
    let effects_info = [
        ("SpectrumBars", "Classic frequency spectrum visualization with vertical bars", "low"),
        ("CircularWave", "Circular ripple effects emanating from the center", "low"),
        ("ParticleSystem", "Dynamic particle effects with physics simulation", "high"),
        ("Heartbeat", "Pulsing heartbeat effect synchronized with audio", "low"),
        ("Starfall", "Falling stars effect with sparkle trails", "medium"),
        ("Rain", "Realistic rain drops with splash effects", "medium"),
        ("Flames", "Fire simulation with heat distortion", "high"),
        ("Applaudimetre", "Advanced applause meter with peak detection", "medium"),
    ];

    if effect_id >= effects_info.len() {
        return Err(format!("Effect ID {} not found", effect_id));
    }

    let (name, description, performance) = effects_info[effect_id];

    Ok(json!({
        "id": effect_id,
        "name": name,
        "description": description,
        "performance_impact": performance,
        "supports_transitions": effect_id != 2, // ParticleSystem doesn't support transitions
        "supports_custom_colors": true
    }))
}

#[tauri::command]
pub async fn effects_set_color_mode(
    window: Window,
    state: State<'_, AppState>,
    mode: String,
) -> Result<serde_json::Value, String> {
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

    println!("🎨 [EFFECTS] Color mode changed to: {}", mode);

    let _ = window.emit("color_mode_changed", json!({
        "mode": mode
    }));

    Ok(json!({
        "mode": mode,
        "message": format!("Color mode set to {}", mode)
    }))
}

#[tauri::command]
pub async fn effects_get_color_mode(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
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
pub async fn effects_set_custom_color(
    window: Window,
    state: State<'_, AppState>,
    r: f32,
    g: f32,
    b: f32,
) -> Result<serde_json::Value, String> {
    if !(0.0..=1.0).contains(&r) || !(0.0..=1.0).contains(&g) || !(0.0..=1.0).contains(&b) {
        return Err(format!(
            "Color values must be between 0.0 and 1.0. Got: R={:.3}, G={:.3}, B={:.3}",
            r, g, b
        ));
    }

    let mut engine = state.effect_engine.lock();
    engine.set_custom_color(r, g, b);

    println!("🎨 [EFFECTS] Custom color set to RGB({:.2}, {:.2}, {:.2})", r, g, b);

    let _ = window.emit("custom_color_changed", json!({
        "r": r, "g": g, "b": b
    }));

    Ok(json!({
        "r": r,
        "g": g,
        "b": b,
        "hex": format!("#{:02X}{:02X}{:02X}",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        ),
        "message": format!("Custom color set to RGB({:.2}, {:.2}, {:.2})", r, g, b)
    }))
}

#[tauri::command]
pub async fn effects_get_custom_color(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
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
pub async fn effects_reset_all(
    window: Window,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut engine = state.effect_engine.lock();
    engine.reset_all_effects();

    println!("🔄 [EFFECTS] All effects reset");

    let _ = window.emit("effects_reset", json!({}));

    Ok(json!({
        "status": "reset",
        "message": "All effects have been reset"
    }))
}

#[tauri::command]
pub async fn effects_get_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
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
