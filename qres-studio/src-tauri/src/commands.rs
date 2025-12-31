use tauri::{Window, Emitter, AppHandle, Manager};
use qres_rust;
use std::process::Command;
use std::fs;
use std::path::PathBuf;

// Stats storage structure
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Stats {
    bytes_saved: u64,
    total_compressions: u32,
    avg_ratio: f64,
    engines_used: std::collections::HashMap<String, u32>,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            bytes_saved: 0,
            total_compressions: 0,
            avg_ratio: 0.0,
            engines_used: std::collections::HashMap::new(),
        }
    }
}

fn get_stats_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("stats.json")
}

fn load_stats(app: &AppHandle) -> Stats {
    let path = get_stats_path(app);
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Stats::default()
    }
}

fn save_stats(app: &AppHandle, stats: &Stats) -> Result<(), String> {
    let path = get_stats_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(stats).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn compress_file(window: Window, app: AppHandle, src: String, dest: String) -> Result<String, String> {
    let src_metadata = fs::metadata(&src).map_err(|e| e.to_string())?;
    let original_size = src_metadata.len();
    
    qres_rust::compress_with_callback(&src, &dest, |progress, ratio, engine| {
        window.emit("compression-progress", serde_json::json!({
            "percent": progress,
            "current_ratio": ratio,
            "active_engine": engine
        })).unwrap_or(());
    }).map_err(|e| e.to_string())?;
    
    // Update stats
    let dest_metadata = fs::metadata(&dest).map_err(|e| e.to_string())?;
    let compressed_size = dest_metadata.len();
    let bytes_saved = original_size.saturating_sub(compressed_size);
    
    let mut stats = load_stats(&app);
    stats.bytes_saved += bytes_saved;
    stats.total_compressions += 1;
    stats.avg_ratio = (stats.avg_ratio * (stats.total_compressions - 1) as f64 
                      + (compressed_size as f64 / original_size as f64)) 
                      / stats.total_compressions as f64;
    
    save_stats(&app, &stats)?;
    
    Ok("Complete".to_string())
}

#[tauri::command]
pub async fn decompress_file(window: Window, src: String, dest: String) -> Result<String, String> {
    // Read compressed file
    let compressed = fs::read(&src).map_err(|e| e.to_string())?;
    
    // Decompress using qres_rust
    let decompressed = qres_rust::decode_bytes(&compressed, 0, None)
        .map_err(|e| format!("Decompression failed: {}", e))?;
    
    // Write decompressed data
    fs::write(&dest, decompressed).map_err(|e| e.to_string())?;
    
    window.emit("decompression-progress", serde_json::json!({
        "percent": 100,
        "status": "complete"
    })).unwrap_or(());
    
    Ok("Complete".to_string())
}

#[tauri::command]
pub async fn get_stats(app: AppHandle) -> Result<serde_json::Value, String> {
    let stats = load_stats(&app);
    Ok(serde_json::to_value(stats).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn toggle_swarm(enabled: bool) -> Result<String, String> {
    if !enabled {
        return Ok("Swarm disabled".to_string());
    }
    
    // Run hive_sync.py subprocess
    let python = if cfg!(windows) { "python" } else { "python3" };
    
    let output = Command::new(python)
        .arg("../../utils/hive_sync.py")
        .env("HIVE_URL", "http://localhost:5000")
        .output()
        .map_err(|e| format!("Failed to run hive_sync: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Hive sync failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(format!("Swarm synced: {}", stdout))
}

#[tauri::command]
pub async fn run_training(data_file: Option<String>) -> Result<String, String> {
    let python = if cfg!(windows) { "python" } else { "python3" };
    
    let mut cmd = Command::new(python);
    cmd.arg("../../ai/train_meta.py");
    
    if let Some(file) = data_file {
        cmd.arg("--data_file").arg(file);
    }
    
    let output = cmd.output()
        .map_err(|e| format!("Failed to run training: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Training failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

#[tauri::command]
pub async fn query_lm(prompt: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama3",
            "prompt": prompt,
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| format!("Ollama connection failed: {}. Is Ollama running? Try 'ollama serve'", e))?;
        
    let json_res: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    
    Ok(json_res["response"].as_str().unwrap_or("").to_string())
}

#[tauri::command]
pub async fn save_ai_data(filename: String, content: String) -> Result<String, String> {
    let path = PathBuf::from("../../ai/generated_data").join(&filename);
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    fs::write(&path, content).map_err(|e| e.to_string())?;
    
    Ok(format!("Saved to {}", path.display()))
}

