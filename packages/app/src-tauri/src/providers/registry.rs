use crate::claude::plugin::ClaudePlugin;
use crate::codex::plugin::CodexPlugin;
use crate::db::Database;
use crate::providers::traits::UsageProvider;

static CODEX: CodexPlugin = CodexPlugin;
static CLAUDE: ClaudePlugin = ClaudePlugin;

pub fn all_providers() -> [&'static dyn UsageProvider; 2] {
    [&CODEX, &CLAUDE]
}

pub fn get_provider(id: &str) -> Option<&'static dyn UsageProvider> {
    all_providers().into_iter().find(|p| p.id() == id)
}

pub fn all_provider_ids() -> Vec<&'static str> {
    all_providers().iter().map(|p| p.id()).collect()
}

pub fn enabled_providers(db: &Database) -> Result<Vec<&'static dyn UsageProvider>, String> {
    let enabled_ids = db.enabled_source_ids().map_err(|e| e.to_string())?;
    Ok(all_providers()
        .into_iter()
        .filter(|p| enabled_ids.iter().any(|id| id == p.id()))
        .collect())
}
