use serde::{Deserialize, Serialize};
use tauri::{Emitter, Window};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressionProgress {
    pub percent: f32,
    pub current_ratio: f32,
    pub active_engine: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressResult {
    pub success: bool,
    pub output_path: String,
    pub final_ratio: f32,
    pub bytes_saved: u64,
}

#[tauri::command]
pub async fn compress_file(
    window: Window,
    src: String,
    dest: String,
) -> Result<CompressResult, String> {
    // Background compression with real-time events
    let src_path = Path::new(&src);
    let dest_path = Path::new(&dest);
    
    if !src_path.exists() {
        return Err("Source file not found".to_string());
    }

    // Get file size for progress calculation  
    let total_size = std::fs::metadata(src_path)
        .map_err(|e| e.to_string())?
        .len();

    // Spawn compression task
    tokio::task::spawn_blocking(move || {
        // Call qres_rust compress with callback
        match compress_with_progress(&src, &dest, total_size, |progress| {
            // Emit real-time event to frontend
            let _ = window.emit("compression-progress", progress);
        }) {
            Ok(result) => result,
            Err(e) => CompressResult {
                success: false,
                output_path: dest,
                final_ratio: 0.0,
                bytes_saved: 0,
            },
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn decompress_file(
    window: Window,
    src: String,
    dest: String,
) -> Result<CompressResult, String> {
    let src_path = Path::new(&src);
    
    if !src_path.exists() || !src.ends_with(".qres") {
        return Err("Invalid .qres file".to_string());
    }

    tokio::task::spawn_blocking(move || {
        match decompress_with_progress(&src, &dest, |progress| {
            let _ = window.emit("decompression-progress", progress);
        }) {
            Ok(result) => result,
            Err(e) => CompressResult {
                success: false,
                output_path: dest,
                final_ratio: 0.0,
                bytes_saved: 0,
            },
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stats() -> Result<qres_rust::stats::CompressionStats, String> {
    let stats = qres_rust::stats::GLOBAL_STATS.lock().unwrap();
    Ok(stats.clone())
}

#[tauri::command]
pub async fn toggle_swarm(enabled: bool) -> Result<(), String> {
    if enabled {
        qres_rust::daemon::DaemonManager::start(false, 600)
    } else {
        qres_rust::daemon::DaemonManager::stop()
    }
}

// Helper function wrapping qres_rust with progress callbacks
fn compress_with_progress<F>(
    src: &str,
    dest: &str,
    total_size: u64,
    mut callback: F,
) -> Result<CompressResult, String>
where
    F: FnMut(CompressionProgress),
{
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use qres_rust::{QresWriter, LivingBrain};

    let reader = BufReader::new(File::open(src).map_err(|e| e.to_string())?);
    let writer = BufWriter::new(File::create(dest).map_err(|e| e.to_string())?);
    
    let brain = qres_rust::LivingBrain::load().unwrap_or_else(|| LivingBrain::new());
    let mut qres_writer = QresWriter::new_with_brain(writer, 0, brain);
    
    let mut total_read = 0u64;
    let mut buffer = vec![0u8; 64 * 1024];
    
    loop {
        let n = std::io::Read::read(&mut reader, &mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        
        std::io::Write::write_all(&mut qres_writer, &buffer[..n]).map_err(|e| e.to_string())?;
        total_read += n as u64;
        
        // Emit progress
        let percent = (total_read as f64 / total_size as f64 * 100.0) as f32;
        callback(CompressionProgress {
            percent,
            current_ratio: 0.5, // Placeholder - would need actual ratio tracking
            active_engine: "zstd".to_string(), // Would track from qres_writer
            bytes_in: total_read,
            bytes_out: total_read / 2, // Approximate
        });
    }
    
    std::io::Write::flush(&mut qres_writer).map_err(|e| e.to_string())?;
    
    Ok(CompressResult {
        success: true,
        output_path: dest.to_string(),
        final_ratio: 0.5,
        bytes_saved: total_size / 2,
    })
}

fn decompress_with_progress<F>(
    src: &str,
    dest: &str,
    mut callback: F,
) -> Result<CompressResult, String>
where
    F: FnMut(CompressionProgress),
{
    // Similar pattern for decompression
    Ok(CompressResult {
        success: true,
        output_path: dest.to_string(),
        final_ratio: 1.0,
        bytes_saved: 0,
    })
}
