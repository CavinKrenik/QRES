use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Window};
use tokio::sync::Mutex;

// P2P State
struct P2PState {
    enabled: bool,
    _peers: Vec<String>,
}

lazy_static::lazy_static! {
    static ref P2P_STATE: Arc<Mutex<P2PState>> = Arc::new(Mutex::new(P2PState {
        enabled: false,
        _peers: Vec::new(),
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

fn load_stats_from_disk(app: &AppHandle) -> Stats {
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
    fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).map_err(|e| e.to_string())?;
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
        window
            .emit(
                "compression-progress",
                serde_json::json!({
                    "percent": progress,
                    "current_ratio": ratio,
                    "active_engine": engine
                }),
            )
            .unwrap_or(());
    })
    .map_err(|e| e.to_string())?;

    // Update stats
    let dest_metadata = fs::metadata(&dest).map_err(|e| e.to_string())?;
    let compressed_size = dest_metadata.len();
    let bytes_saved = original_size.saturating_sub(compressed_size);
    let ratio = compressed_size as f64 / original_size as f64;

    let mut stats = load_stats_from_disk(&app);
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
    use qres_rust::archive::{create_archive, ArchiveOptions};

    let src_path = Path::new(&src);
    let folder_name = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");

    // Create archive with .qrar extension
    let archive_path = Path::new(&dest_folder).join(format!("{}.qrar", folder_name));

    // Emit initial progress
    window
        .emit(
            "compression-progress",
            serde_json::json!({
                "percent": 0,
                "status": "scanning",
                "message": "Scanning directory..."
            }),
        )
        .unwrap_or(());

    // Create archive with default options (solid compression enabled)
    let options = ArchiveOptions::default();

    let manifest = create_archive(src.as_str(), archive_path.to_str().unwrap(), options)
        .map_err(|e| format!("Archive creation failed: {}", e))?;

    // Emit completion
    window
        .emit(
            "compression-progress",
            serde_json::json!({
                "percent": 100,
                "status": "complete",
                "files": manifest.files.len(),
                "total_size": manifest.total_size
            }),
        )
        .unwrap_or(());

    // Update stats
    let mut stats = load_stats_from_disk(&app);
    stats.total_compressions += manifest.files.len() as u32;

    // Calculate compression ratio
    if let Ok(metadata) = fs::metadata(&archive_path) {
        let compressed_size = metadata.len();
        let bytes_saved = manifest.total_size.saturating_sub(compressed_size);
        let ratio = compressed_size as f64 / manifest.total_size as f64;

        stats.bytes_saved += bytes_saved;
        stats.avg_ratio = (stats.avg_ratio
            * (stats.total_compressions - manifest.files.len() as u32) as f64
            + ratio * manifest.files.len() as f64)
            / stats.total_compressions as f64;
    }

    save_stats(&app, &stats)?;

    Ok(serde_json::json!({
        "status": "complete",
        "total_files": manifest.files.len(),
        "total_size": manifest.total_size,
        "archive_path": archive_path.to_string_lossy()
    }))
}

#[tauri::command]
pub async fn decompress_file(window: Window, src: String, dest: String) -> Result<String, String> {
    use std::fs::File;
    use std::io::{BufReader, BufWriter, Read, Write};

    // 1. Open streams (Buffered for performance)
    let f_in = File::open(&src).map_err(|e| format!("Failed to open source: {}", e))?;
    let mut reader = BufReader::new(f_in);
    let f_out = File::create(&dest).map_err(|e| format!("Failed to create destination: {}", e))?;
    let mut writer = BufWriter::new(f_out);

    // 2. Validate QRES Magic Header
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .map_err(|_| "Not a QRES file (too short)".to_string())?;
    if &magic != b"QRES" {
        return Err(format!(
            "Invalid file signature. Expected 'QRES', got {:?}",
            String::from_utf8_lossy(&magic)
        ));
    }

    // 3. Parse Static Header
    // [Version:1][Flags:1][PredID:1][Time:8][OrigSize:8][CompSize:8] = 27 bytes
    let mut header_static = [0u8; 27];
    reader
        .read_exact(&mut header_static)
        .map_err(|e| format!("Invalid header: {}", e))?;

    let _version = header_static[0];
    let _flags = header_static[1];
    let _pred_id = header_static[2];
    let original_size = u64::from_le_bytes(header_static[11..19].try_into().unwrap());
    let _compressed_size = u64::from_le_bytes(header_static[19..27].try_into().unwrap());

    // 4. Read Filename Length and Skip Filename
    let mut name_len_bytes = [0u8; 4];
    reader
        .read_exact(&mut name_len_bytes)
        .map_err(|e| format!("Failed to read filename length: {}", e))?;
    let name_len = u32::from_le_bytes(name_len_bytes) as usize;

    // Validate filename length (prevent DOS attacks)
    if name_len > 4096 {
        return Err(format!("Filename length too large: {} bytes", name_len));
    }

    // Skip filename
    let mut name_buf = vec![0u8; name_len];
    reader
        .read_exact(&mut name_buf)
        .map_err(|e| format!("Failed to read filename: {}", e))?;

    // 5. Stream Chunks
    let mut total_written = 0u64;
    let mut chunk_count = 0;

    loop {
        // Try to read chunk length (4 bytes)
        let mut chunk_len_bytes = [0u8; 4];
        match reader.read_exact(&mut chunk_len_bytes) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Normal EOF - we've read all chunks
                break;
            }
            Err(e) => return Err(format!("Error reading chunk header: {}", e)),
        }

        let chunk_len = u32::from_le_bytes(chunk_len_bytes) as usize;

        // Sanity check: prevent malicious files from requesting massive allocations
        if chunk_len > 10 * 1024 * 1024 {
            // 10MB max per compressed chunk
            return Err(format!("Chunk size too large: {} bytes", chunk_len));
        }

        // Read the compressed chunk data
        let mut chunk_data = vec![0u8; chunk_len];
        reader.read_exact(&mut chunk_data).map_err(|e| {
            format!(
                "Failed to read chunk {} (expected {} bytes): {}",
                chunk_count, chunk_len, e
            )
        })?;

        // Decompress chunk
        let decoded = qres_rust::decompress_chunk(&chunk_data, 0, None)
            .map_err(|e| format!("Chunk {} decompression failed: {}", chunk_count, e))?;

        // Write decompressed data
        writer
            .write_all(&decoded)
            .map_err(|e| format!("Write failed: {}", e))?;

        total_written += decoded.len() as u64;
        chunk_count += 1;

        // Emit progress (based on original file size)
        let progress = if original_size > 0 {
            ((total_written as f64 / original_size as f64) * 100.0).min(100.0) as u32
        } else {
            50 // Unknown size, show generic progress
        };

        window
            .emit(
                "decompression-progress",
                serde_json::json!({
                    "percent": progress,
                    "status": "extracting",
                    "bytes_written": total_written,
                    "chunk": chunk_count
                }),
            )
            .unwrap_or(());
    }

    // Flush writer
    writer
        .flush()
        .map_err(|e| format!("Failed to flush output: {}", e))?;
    drop(writer);

    // Final progress
    window
        .emit(
            "decompression-progress",
            serde_json::json!({
                "percent": 100,
                "status": "complete",
                "total_bytes": total_written,
                "chunks": chunk_count
            }),
        )
        .unwrap_or(());

    // Verify size matches (if header specified it)
    if original_size > 0 && total_written != original_size {
        eprintln!(
            "Warning: Size mismatch. Expected {} bytes, got {}",
            original_size, total_written
        );
    }

    Ok(format!(
        "Decompressed {} bytes in {} chunks",
        total_written, chunk_count
    ))
}

#[tauri::command]
pub async fn browse_archive(archive_path: String) -> Result<serde_json::Value, String> {
    use qres_rust::archive::read_manifest;

    let manifest = read_manifest(archive_path.as_str())
        .map_err(|e| format!("Failed to read archive: {}", e))?;

    // Convert manifest to JSON-friendly format
    let files: Vec<serde_json::Value> = manifest
        .files
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f.path,
                "size": f.original_size,
                "modified": f.modified,
                "hash": f.hash
            })
        })
        .collect();

    Ok(serde_json::json!({
        "total_size": manifest.total_size,
        "compression_method": manifest.compression_method,
        "files": files,
        "file_count": files.len()
    }))
}

#[tauri::command]
pub async fn extract_archive(
    window: Window,
    archive_path: String,
    output_dir: String,
) -> Result<String, String> {
    use qres_rust::archive::extract_archive;

    window
        .emit(
            "extraction-progress",
            serde_json::json!({
                "percent": 0,
                "status": "extracting"
            }),
        )
        .unwrap_or(());

    let manifest = extract_archive(archive_path.as_str(), output_dir.as_str())
        .map_err(|e| format!("Extraction failed: {}", e))?;

    window
        .emit(
            "extraction-progress",
            serde_json::json!({
                "percent": 100,
                "status": "complete",
                "files": manifest.files.len()
            }),
        )
        .unwrap_or(());

    Ok(format!("Extracted {} files", manifest.files.len()))
}

#[tauri::command]
pub async fn extract_archive_file(
    archive_path: String,
    file_path: String,
    output_path: String,
) -> Result<String, String> {
    use qres_rust::archive::extract_archive;

    // For now, we extract the whole archive to a temp dir and copy the file
    // In the future, we can optimize this to extract only the requested file
    let temp_dir = std::env::temp_dir().join(format!(
        "qres_extract_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let _manifest = extract_archive(archive_path.as_str(), temp_dir.to_str().unwrap())
        .map_err(|e| format!("Extraction failed: {}", e))?;

    // Copy the requested file
    let source = temp_dir.join(&file_path);
    fs::copy(&source, &output_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    // Clean up temp dir
    fs::remove_dir_all(&temp_dir).ok();

    Ok(format!("Extracted {}", file_path))
}

#[tauri::command]
pub async fn get_stats(app: AppHandle) -> Result<serde_json::Value, String> {
    let stats = load_stats_from_disk(&app);
    serde_json::to_value(stats).map_err(|e| e.to_string())
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub latency_ms: u32,
    pub role: String,
    pub throughput_mbps: f32,
    pub location: String,
}

#[tauri::command]
pub async fn get_swarm_peers() -> Result<Vec<PeerInfo>, String> {
    // Phase 5.1: Simulate Swarm State for GUI Visualization
    // In v6.0 this will hook into the actual libp2p Kademlia DHT
    let mut rng = rand::thread_rng();

    let locations = ["US-East", "EU-West", "Asia-South", "SA-East", "US-West"];
    let mut peers = Vec::new();

    // Always include "Self"
    peers.push(PeerInfo {
        id: "Local-Node-001".to_string(),
        latency_ms: 0,
        role: "Validator".to_string(),
        throughput_mbps: 0.0,
        location: "Local".to_string(),
    });

    let count = rng.gen_range(3..8);
    for _ in 0..count {
        peers.push(PeerInfo {
            id: format!("Peer-{:03}", rng.gen_range(1..999)),
            latency_ms: rng.gen_range(10..150),
            role: if rng.gen_bool(0.2) {
                "Anchor"
            } else {
                "Worker"
            }
            .to_string(),
            throughput_mbps: rng.gen_range(50.0..500.0),
            location: locations[rng.gen_range(0..locations.len())].to_string(),
        });
    }

    Ok(peers)
}

#[tauri::command]
pub async fn load_stats() -> Result<serde_json::Value, String> {
    // Simplified stats for GUI
    Ok(serde_json::json!({
        "bytes_saved": 0.0,
        "efficiency": 0.0,
        "compressions": 0,
        "active_nodes": 0
    }))
}

#[tauri::command]
pub async fn fetch_peers() -> Result<Vec<String>, String> {
    // Simplified peer list
    Ok(vec!["Node1 (Local)".to_string()])
}

#[tauri::command]
pub async fn load_data() -> Result<String, String> {
    // Placeholder for loading data
    Ok("Data loaded".to_string())
}

#[tauri::command]
pub async fn compress(
    path: String,
    _mode: String,
    _threshold: f32,
    out_path: Option<String>,
) -> Result<String, String> {
    let dest = out_path
        .unwrap_or_else(|| format!("output/{}.qres", path.split('.').next().unwrap_or("file")));
    // For now, simulate compression
    // In full implementation, call the actual compression logic
    Ok(dest)
}

#[tauri::command]
pub async fn decompress(_path: String, out_folder: Option<String>) -> Result<String, String> {
    let dest = out_folder.unwrap_or_else(|| "output/decompressed".to_string());
    // For now, simulate decompression
    Ok(dest)
}

#[tauri::command]
pub async fn get_knowledge_graph() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "nodes": [
            { "id": "User", "type": "human", "content": "Operator" },
            { "id": "Agent-001", "type": "agent", "content": "Neural Processor" },
            { "id": "DataStore", "type": "storage", "content": "Hive Storage" },
            { "id": "Node-A", "type": "node", "content": "Compute Node A" },
            { "id": "Node-B", "type": "node", "content": "Compute Node B" }
        ],
        "edges": [
            { "source": "User", "target": "Agent-001", "weight": 1.0 },
            { "source": "Agent-001", "target": "DataStore", "weight": 0.8 },
            { "source": "Agent-001", "target": "Node-A", "weight": 0.5 },
            { "source": "Agent-001", "target": "Node-B", "weight": 0.5 }
        ]
    }))
}

async fn sync_with_swarm(_app: &AppHandle) -> Result<(), String> {
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
