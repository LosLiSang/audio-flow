#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tracing_subscriber::fmt::init();

            tracing::info!("Starting Audio Flow v0.1.0");

            let state = audio_flow::AppState::new();
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio_flow::commands::list_devices,
            audio_flow::commands::add_route,
            audio_flow::commands::remove_route,
            audio_flow::commands::set_gain,
            audio_flow::commands::start_engine,
            audio_flow::commands::stop_engine,
            audio_flow::commands::get_peak_levels,
            audio_flow::commands::get_routes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
