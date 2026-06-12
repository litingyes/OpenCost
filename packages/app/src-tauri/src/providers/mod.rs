//! Provider plugin system for agent usage tracking.
//!
//! To add a new built-in provider:
//! 1. Create `src/{agent}/` with `plugin.rs` implementing `UsageProvider`
//! 2. Register it in `registry.rs`
//! 3. Add frontend metadata in `features/usage/providers.ts`

mod registry;
mod traits;

pub use registry::{all_provider_ids, all_providers, enabled_providers, get_provider};
pub use traits::{ProviderSyncReport, UsageProvider};

use crate::db::Database;
use crate::watcher;
use tauri::AppHandle;

pub fn sync_enabled_providers(db: &Database, force: bool) -> Result<Vec<ProviderSyncReport>, String> {
    if force {
        db.reset_cursors().map_err(|e| e.to_string())?;
    }

    let since_ms = db
        .get_sync_settings()
        .map_err(|e| e.to_string())?
        .effective_since_ms;

    let providers = enabled_providers(db)?;
    let mut reports = Vec::new();

    for provider in providers {
        reports.push(provider.sync(db, force, since_ms)?);
    }

    db.set_last_sync().map_err(|e| e.to_string())?;
    Ok(reports)
}

pub fn restart_watcher(app: &AppHandle) {
    watcher::request_restart(app);
}
