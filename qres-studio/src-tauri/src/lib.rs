mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_websocket::init())
        .invoke_handler(tauri::generate_handler![
            commands::compress_file,
            commands::decompress_file,
            commands::get_stats,
            commands::toggle_swarm,
            commands::get_swarm_status,
            commands::train_on_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
