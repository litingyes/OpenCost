use std::path::{Path, PathBuf};

pub fn codex_home() -> PathBuf {
    if let Ok(path) = std::env::var("CODEX_HOME") {
        return PathBuf::from(path);
    }
    dirs::home_dir()
        .map(|h| h.join(".codex"))
        .unwrap_or_else(|| PathBuf::from(".codex"))
}

pub fn discover_rollout_files(home: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rollouts(home.join("sessions").as_path(), &mut files);
    collect_rollouts(home.join("archived_sessions").as_path(), &mut files);
    files.sort();
    files
}

fn collect_rollouts(dir: &Path, out: &mut Vec<PathBuf>) {
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
            collect_rollouts(&path, out);
        } else if is_rollout_file(&path) {
            out.push(path);
        }
    }
}

fn is_rollout_file(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "jsonl")
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("rollout-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollout_filename_detection() {
        assert!(is_rollout_file(Path::new(
            "/tmp/rollout-2026-03-17T23-00-03-uuid.jsonl"
        )));
        assert!(!is_rollout_file(Path::new("/tmp/other.jsonl")));
    }
}
