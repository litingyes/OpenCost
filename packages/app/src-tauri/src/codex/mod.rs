mod discover;
mod enrich;
mod parser;
pub mod plugin;

pub use discover::codex_home;
pub use enrich::enrich_from_state_db;
pub use parser::{sync_codex_files, SyncReport};
