use std::path::{Path, PathBuf};

pub fn claude_home() -> PathBuf {
    if let Ok(path) = std::env::var("CLAUDE_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    dirs::home_dir()
        .map(|h| h.join(".claude"))
        .unwrap_or_else(|| PathBuf::from(".claude"))
}

pub fn claude_roots() -> Vec<PathBuf> {
    let mut roots = vec![claude_home()];
    if let Some(config) = dirs::config_dir() {
        let xdg = config.join("claude");
        if xdg != roots[0] {
            roots.push(xdg);
        }
    }
    roots
}

pub fn discover_jsonl_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in roots {
        collect_jsonl(root.join("projects").as_path(), &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_jsonl(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl(&path, out);
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            out.push(path);
        }
    }
}
