use crate::db::Database;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderSyncReport {
    pub id: &'static str,
    pub files_scanned: usize,
    pub events_ingested: usize,
}

pub trait UsageProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn home_dir(&self) -> PathBuf;
    fn watch_paths(&self) -> Vec<PathBuf>;
    fn sync(&self, db: &Database, force: bool) -> Result<ProviderSyncReport, String>;
}
