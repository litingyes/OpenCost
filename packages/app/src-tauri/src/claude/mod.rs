mod discover;
mod parser;
pub mod plugin;

pub use discover::{claude_home, claude_roots};
pub use parser::{sync_claude_files, SyncReport};
