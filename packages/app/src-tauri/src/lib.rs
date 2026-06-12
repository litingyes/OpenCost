pub mod claude;
pub mod codex;
mod commands;
pub mod db;
pub mod providers;
mod watcher;

use commands::{
    get_providers, get_session_breakdown, get_session_detail, get_sync_settings, get_sync_status,
    get_usage_timeseries, initial_sync, set_provider_enabled, set_sync_settings, sync_all,
    AppState,
};
use db::Database;
use providers::all_provider_ids;
use tauri::Manager;
use tauri_plugin_log::log::{info, warn};

#[cfg(desktop)]
fn cli_force_sync(app: &tauri::App) -> bool {
    use tauri_plugin_cli::CliExt;

    match app.cli().matches() {
        Ok(matches) => {
            info!("CLI matches: {matches:?}");
            matches
                .args
                .get("sync")
                .and_then(|arg| arg.value.as_bool())
                .unwrap_or(false)
        }
        Err(err) => {
            warn!("Failed to parse CLI arguments: {err}");
            false
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_window_state::Builder::new().build())
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_focus();
                }
            }))
            .plugin(tauri_plugin_autostart::Builder::new().build())
            .plugin(tauri_plugin_cli::init());
    }

    builder
        .setup(|app| {
            info!("OpenCost starting");

            #[cfg(desktop)]
            let force_sync = cli_force_sync(app);
            #[cfg(not(desktop))]
            let force_sync = false;

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
                initial_sync(handle, force_sync);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sync_all,
            get_providers,
            set_provider_enabled,
            get_sync_status,
            get_sync_settings,
            set_sync_settings,
            get_usage_timeseries,
            get_session_breakdown,
            get_session_detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
