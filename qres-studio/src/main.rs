// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::compress_file,
            commands::decompress_file,
            commands::get_stats,
            commands::toggle_swarm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
