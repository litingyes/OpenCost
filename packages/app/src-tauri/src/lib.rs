pub mod claude;
pub mod codex;
mod commands;
pub mod db;
pub mod providers;
mod watcher;

use commands::{
    get_providers, get_session_breakdown, get_session_detail, get_sync_status, get_usage_timeseries,
    initial_sync, set_provider_enabled, sync_all, AppState,
};
use db::Database;
use providers::all_provider_ids;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let db_path = data_dir.join("opencost.db");
            let database = Database::open(&db_path).expect("failed to open database");
            database
                .seed_provider_settings(&all_provider_ids())
                .expect("failed to seed provider settings");

            let state = AppState {
                db: std::sync::Mutex::new(database),
                watcher: std::sync::Mutex::new(None),
            };
            app.manage(state);

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                initial_sync(handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sync_all,
            get_providers,
            set_provider_enabled,
            get_sync_status,
            get_usage_timeseries,
            get_session_breakdown,
            get_session_detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
