use tauri::{Window, Emitter, AppHandle, Manager};
use qres_rust;
use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use tokio::sync::Mutex;
use std::sync::Arc;

// P2P State
struct P2PState {
    enabled: bool,
    peers: Vec<String>,
}

lazy_static::lazy_static! {
    static ref P2P_STATE: Arc<Mutex<P2PState>> = Arc::new(Mutex::new(P2PState {
        enabled: false,
        peers: Vec::new(),
    }));
}

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

fn get_swarm_config_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("swarm_config.json")
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

fn load_swarm_state(app: &AppHandle) -> bool {
    let path = get_swarm_config_path(app);
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            return config["enabled"].as_bool().unwrap_or(false);
        }
    }
    false
}

fn save_swarm_state(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let path = get_swarm_config_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let config = serde_json::json!({ "enabled": enabled });
    fs::write(&path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn is_data_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "csv" | "json" | "txt" | "log" | "dat")
    } else {
        false
    }
}

#[tauri::command]
pub async fn compress_file(
    window: Window,
    app: AppHandle,
    src: String,
    dest: String,
) -> Result<serde_json::Value, String> {
    let src_path = Path::new(&src);
    
    // Check if it's a directory
    if src_path.is_dir() {
        return compress_folder(window, app, src, dest).await;
    }
    
    let src_metadata = fs::metadata(&src).map_err(|e| e.to_string())?;
    let original_size = src_metadata.len();
    let is_trainable = is_data_file(src_path);
    
    // Compress the file
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
    let ratio = compressed_size as f64 / original_size as f64;
    
    let mut stats = load_stats(&app);
    stats.bytes_saved += bytes_saved;
    stats.total_compressions += 1;
    stats.avg_ratio = (stats.avg_ratio * (stats.total_compressions - 1) as f64 + ratio) 
                      / stats.total_compressions as f64;
    
    save_stats(&app, &stats)?;
    
    // Sync with swarm if enabled
    if load_swarm_state(&app) {
        sync_with_swarm(&app).await?;
    }
    
    Ok(serde_json::json!({
        "status": "complete",
        "is_trainable": is_trainable,
        "ratio": ratio
    }))
}

async fn compress_folder(
    window: Window,
    app: AppHandle,
    src: String,
    dest_folder: String,
) -> Result<serde_json::Value, String> {
    let src_path = Path::new(&src);
    let dest_path = Path::new(&dest_folder);
    
    // Create destination folder
    fs::create_dir_all(dest_path).map_err(|e| e.to_string())?;
    
    let mut total_files = 0;
    let mut compressed_files = 0;
    
    // Walk directory and compress each file
    for entry in WalkDir::new(src_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            total_files += 1;
            
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(src_path).unwrap();
            let dest_file = dest_path.join(relative_path).with_extension("qres");
            
            // Create parent directories
            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent).ok();
            }
            
            // Compress file
            match qres_rust::compress_with_callback(
                &file_path.to_string_lossy(),
                &dest_file.to_string_lossy(),
                |progress, ratio, engine| {
                    window.emit("compression-progress", serde_json::json!({
                        "percent": progress,
                        "current_ratio": ratio,
                        "active_engine": engine,
                        "file": relative_path.to_string_lossy()
                    })).unwrap_or(());
                },
            ) {
                Ok(_) => compressed_files += 1,
                Err(e) => eprintln!("Failed to compress {}: {}", file_path.display(), e),
            }
        }
    }
    
    // Update stats
    let mut stats = load_stats(&app);
    stats.total_compressions += compressed_files;
    save_stats(&app, &stats)?;
    
    Ok(serde_json::json!({
        "status": "complete",
        "total_files": total_files,
        "compressed_files": compressed_files
    }))
}

#[tauri::command]
pub async fn decompress_file(
    window: Window,
    src: String,
    dest: String,
) -> Result<String, String> {
    let compressed = fs::read(&src).map_err(|e| e.to_string())?;
    let decompressed = qres_rust::decompress_chunk(&compressed, 0, None)
        .map_err(|e| format!("Decompression failed: {}", e))?;
    
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
pub async fn toggle_swarm(app: AppHandle, enabled: bool) -> Result<String, String> {
    // Save persistent state
    save_swarm_state(&app, enabled)?;
    
    // Update runtime state
    let mut state = P2P_STATE.lock().await;
    state.enabled = enabled;
    drop(state);
    
    if enabled {
        // Start P2P listener and sync
        sync_with_swarm(&app).await?;
        Ok("Swarm enabled and synced".to_string())
    } else {
        Ok("Swarm disabled".to_string())
    }
}

#[tauri::command]
pub async fn get_swarm_status(app: AppHandle) -> Result<bool, String> {
    Ok(load_swarm_state(&app))
}

#[tauri::command]
pub async fn train_on_file(file_path: String) -> Result<String, String> {
    let python = if cfg!(windows) { "python" } else { "python3" };
    
    let output = Command::new(python)
        .arg("../../ai/train_meta.py")
        .arg("--data_file")
        .arg(&file_path)
        .output()
        .map_err(|e| format!("Failed to run training: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Training failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

async fn sync_with_swarm(app: &AppHandle) -> Result<(), String> {
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
    
    Ok(())
}
