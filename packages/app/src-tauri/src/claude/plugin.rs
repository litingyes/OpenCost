use crate::claude::discover::{claude_home, claude_roots};
use crate::claude::parser::sync_claude_files;
use crate::db::Database;
use crate::providers::{ProviderSyncReport, UsageProvider};
use std::path::PathBuf;

pub struct ClaudePlugin;

impl UsageProvider for ClaudePlugin {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn display_name(&self) -> &'static str {
        "Claude Code"
    }

    fn home_dir(&self) -> PathBuf {
        claude_home()
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        claude_roots()
            .into_iter()
            .map(|r| r.join("projects"))
            .collect()
    }

    fn sync(&self, db: &Database, force: bool) -> Result<ProviderSyncReport, String> {
        let roots = claude_roots();
        let report = sync_claude_files(db, &roots, force)?;
        db.recompute_session_aggregates(self.id())
            .map_err(|e| e.to_string())?;
        Ok(ProviderSyncReport {
            id: self.id(),
            files_scanned: report.files_scanned,
            events_ingested: report.events_ingested,
        })
    }
}
