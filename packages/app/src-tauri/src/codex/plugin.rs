use crate::codex::discover::codex_home;
use crate::codex::enrich::enrich_from_state_db;
use crate::codex::parser::sync_codex_files;
use crate::db::Database;
use crate::providers::{ProviderSyncReport, UsageProvider};
use std::path::PathBuf;

pub struct CodexPlugin;

impl UsageProvider for CodexPlugin {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    fn home_dir(&self) -> PathBuf {
        codex_home()
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        let home = codex_home();
        vec![home.join("sessions"), home.join("archived_sessions")]
    }

    fn sync(&self, db: &Database, force: bool) -> Result<ProviderSyncReport, String> {
        let home = codex_home();
        let report = sync_codex_files(db, &home, force)?;
        enrich_from_state_db(db, &home)?;
        db.recompute_session_aggregates(self.id())
            .map_err(|e| e.to_string())?;
        Ok(ProviderSyncReport {
            id: self.id(),
            files_scanned: report.files_scanned,
            events_ingested: report.events_ingested,
        })
    }
}
