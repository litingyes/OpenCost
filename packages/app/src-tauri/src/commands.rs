use crate::db::{
    Database, ProviderInfo, SessionBreakdownItem, SessionRow, SyncSettings, SyncStatus,
    SyncWindowChange, TimeseriesPoint, UsageEventRow,
};
use crate::providers::{all_providers, get_provider, restart_watcher, sync_enabled_providers};
use crate::watcher::WatcherHandle;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub db: Mutex<Database>,
    pub watcher: Mutex<Option<WatcherHandle>>,
}

#[derive(Debug, Deserialize)]
pub struct SyncAllArgs {
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TimeseriesArgs {
    pub range: Option<String>,
    pub bucket: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionBreakdownArgs {
    pub range: Option<String>,
    pub limit: Option<i64>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionDetailArgs {
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SetProviderEnabledArgs {
    pub id: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetSyncSettingsArgs {
    pub preset: String,
    pub custom_since_ms: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct SetSyncSettingsResponse {
    pub settings: SyncSettings,
    pub needs_sync: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct SessionDetailResponse {
    pub session: SessionRow,
    pub events: Vec<UsageEventRow>,
}

#[derive(Debug, serde::Serialize)]
pub struct SyncAllResponse {
    pub reports: Vec<crate::providers::ProviderSyncReport>,
}

fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&Database) -> Result<T, String>,
{
    let db = state.db.lock().map_err(|e| e.to_string())?;
    f(&db)
}

fn build_provider_infos(db: &Database) -> Result<Vec<ProviderInfo>, String> {
    let mut infos = Vec::new();
    for provider in all_providers() {
        let enabled = db
            .is_provider_enabled(provider.id())
            .map_err(|e| e.to_string())?;
        let (session_count, total_tokens) = db
            .provider_stats(provider.id())
            .map_err(|e| e.to_string())?;
        infos.push(ProviderInfo {
            id: provider.id().to_string(),
            display_name: provider.display_name().to_string(),
            home: provider.home_dir().to_string_lossy().to_string(),
            enabled,
            session_count,
            total_tokens,
        });
    }
    Ok(infos)
}

#[tauri::command]
pub fn sync_all(
    state: State<AppState>,
    args: Option<SyncAllArgs>,
) -> Result<SyncAllResponse, String> {
    let force = args.and_then(|a| a.force).unwrap_or(false);
    let reports = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        sync_enabled_providers(&db, force)?
    };
    Ok(SyncAllResponse { reports })
}

#[tauri::command]
pub fn get_providers(state: State<AppState>) -> Result<Vec<ProviderInfo>, String> {
    with_db(&state, |db| build_provider_infos(db))
}

#[tauri::command]
pub fn set_provider_enabled(
    app: AppHandle,
    state: State<AppState>,
    args: SetProviderEnabledArgs,
) -> Result<Vec<ProviderInfo>, String> {
    if get_provider(&args.id).is_none() {
        return Err(format!("Unknown provider: {}", args.id));
    }

    let infos = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.set_provider_enabled(&args.id, args.enabled)
            .map_err(|e| e.to_string())?;
        build_provider_infos(&db)?
    };

    restart_watcher(&app);
    let _ = app.emit("providers:changed", ());
    Ok(infos)
}

#[tauri::command]
pub fn get_sync_settings(state: State<AppState>) -> Result<SyncSettings, String> {
    with_db(&state, |db| {
        db.get_sync_settings().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn set_sync_settings(
    app: AppHandle,
    state: State<AppState>,
    args: SetSyncSettingsArgs,
) -> Result<SetSyncSettingsResponse, String> {
    let (settings, change) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let change = db
            .set_sync_settings(&args.preset, args.custom_since_ms)
            .map_err(|e| e.to_string())?;
        let settings = db.get_sync_settings().map_err(|e| e.to_string())?;
        (settings, change)
    };

    let needs_sync = change == SyncWindowChange::Expanded;
    if needs_sync {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            if let Some(state) = app_clone.try_state::<AppState>() {
                if let Ok(db) = state.db.lock() {
                    let _ = sync_enabled_providers(&db, false);
                    let _ = app_clone.emit("usage:updated", ());
                }
            }
        });
    } else if change == SyncWindowChange::Shrunk {
        let _ = app.emit("usage:updated", ());
    }

    Ok(SetSyncSettingsResponse {
        settings,
        needs_sync,
    })
}

#[tauri::command]
pub fn get_sync_status(state: State<AppState>) -> Result<SyncStatus, String> {
    with_db(&state, |db| {
        let last_sync = db.get_last_sync().map_err(|e| e.to_string())?;
        let providers = build_provider_infos(db)?;
        Ok(SyncStatus {
            last_sync,
            providers,
        })
    })
}

#[tauri::command]
pub fn get_usage_timeseries(
    state: State<AppState>,
    args: TimeseriesArgs,
) -> Result<Vec<TimeseriesPoint>, String> {
    let range = args.range.as_deref().unwrap_or("7d");
    let bucket = args.bucket.as_deref().unwrap_or("day");
    let source = args.source.as_deref().unwrap_or("all");

    with_db(&state, |db| {
        db.get_usage_timeseries(range, bucket, source)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_session_breakdown(
    state: State<AppState>,
    args: SessionBreakdownArgs,
) -> Result<Vec<SessionBreakdownItem>, String> {
    let range = args.range.as_deref().unwrap_or("7d");
    let limit = args.limit.unwrap_or(20);
    let source = args.source.as_deref().unwrap_or("all");

    with_db(&state, |db| {
        db.get_session_breakdown(range, limit, source)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_session_detail(
    state: State<AppState>,
    args: SessionDetailArgs,
) -> Result<Option<SessionDetailResponse>, String> {
    with_db(&state, |db| {
        let session = db
            .get_session_detail(&args.session_id)
            .map_err(|e| e.to_string())?;
        let Some(session) = session else {
            return Ok(None);
        };
        let events = db
            .get_session_events(&args.session_id)
            .map_err(|e| e.to_string())?;
        Ok(Some(SessionDetailResponse { session, events }))
    })
}

pub fn initial_sync(app: AppHandle, force: bool) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(db) = state.db.lock() {
            let ids: Vec<&str> = all_providers().iter().map(|p| p.id()).collect();
            let _ = db.seed_provider_settings(&ids);
            let _ = sync_enabled_providers(&db, force);
        }
    }

    if let Ok(mut watcher) = app.state::<AppState>().watcher.lock() {
        if watcher.is_none() {
            *watcher = Some(crate::watcher::start_watcher(app.clone()));
        }
    }
}
