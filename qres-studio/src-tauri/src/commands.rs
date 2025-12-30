use tauri::{Window, Emitter};
use qres_rust;

#[tauri::command]
pub async fn compress_file(window: Window, src: String, dest: String) -> Result<String, String> {
    qres_rust::compress_with_callback(&src, &dest, |progress, ratio, engine| {
        window.emit("compression-progress", serde_json::json!({
            "percent": progress,
            "current_ratio": ratio,
            "active_engine": engine
        })).unwrap_or(());
    }).map_err(|e| e.to_string())?;
    
    Ok("Complete".to_string())
}

#[tauri::command]
pub async fn get_stats() -> Result<serde_json::Value, String> {
    // Placeholder stats
    Ok(serde_json::json!({
        "bytes_saved": 1024 * 1024 * 5,
        "total_compressions": 42,
        "avg_ratio": 0.4,
        "engines_used": { "lstm": 10, "tensor": 32 }
    }))
}

#[tauri::command]
pub async fn toggle_swarm(enabled: bool) -> Result<(), String> {
    // Placeholder
    Ok(())
}
