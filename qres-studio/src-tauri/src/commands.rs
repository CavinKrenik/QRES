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
pub async fn toggle_swarm(_enabled: bool) -> Result<(), String> {
    // Placeholder
    Ok(())
}

#[tauri::command]
pub async fn query_lm(prompt: String) -> Result<String, String> {
    // Basic Ollama integration
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama3", // Default, user can change via GUI later
            "prompt": prompt,
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| format!("Ollama connection failed: {}", e))?;
        
    let json_res: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    
    Ok(json_res["response"].as_str().unwrap_or("").to_string())
}
