use crate::claude::discover::discover_jsonl_files;
use crate::db::{Database, UsageEventRow};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const SOURCE: &str = "claude";

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncReport {
    pub files_scanned: usize,
    pub events_ingested: usize,
}

pub fn sync_claude_files(
    db: &Database,
    roots: &[PathBuf],
    force: bool,
    since_ms: Option<i64>,
) -> Result<SyncReport, String> {
    let files = discover_jsonl_files(roots);
    let mut events_ingested = 0usize;

    for path in &files {
        let ingested = parse_jsonl_file(db, path, force, since_ms).map_err(|e| e.to_string())?;
        events_ingested += ingested;
    }

    Ok(SyncReport {
        files_scanned: files.len(),
        events_ingested,
    })
}

fn parse_jsonl_file(
    db: &Database,
    path: &Path,
    force: bool,
    since_ms: Option<i64>,
) -> Result<usize, String> {
    let file_path = path.to_string_lossy().to_string();
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let mut offset = 0i64;
    if !force {
        if let Ok(Some((saved_offset, saved_mtime))) = db.get_cursor(&file_path) {
            if saved_mtime == mtime {
                offset = saved_offset;
            }
        }
    }

    let mut file = File::open(path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut ingested = 0usize;

    let fallback_session_id = session_id_from_filename(path);
    let mut ctx = ParseContext {
        session_id: fallback_session_id,
        cwd: None,
        model: None,
        title: None,
        file_path: file_path.clone(),
    };

    while {
        line.clear();
        reader.read_line(&mut line).map_err(|e| e.to_string())?
    } > 0
    {
        offset += line.len() as i64;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(event) = parse_line(trimmed, &ctx) {
            if let Some(since) = since_ms {
                if event.ts < since {
                    continue;
                }
            }
            if db.insert_usage_event(&event).map_err(|e| e.to_string())? {
                ingested += 1;
            }
        }

        apply_line_to_context(trimmed, &mut ctx);
    }

    db.set_cursor(&file_path, offset, mtime)
        .map_err(|e| e.to_string())?;

    if let Some(ref sid) = ctx.session_id {
        db.upsert_session_meta(
            sid,
            SOURCE,
            ctx.title.as_deref(),
            ctx.cwd.as_deref(),
            ctx.model.as_deref(),
            None,
            None,
            Some(&file_path),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(ingested)
}

struct ParseContext {
    session_id: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
    title: Option<String>,
    file_path: String,
}

fn session_id_from_filename(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

fn parse_timestamp(value: &Value) -> Option<i64> {
    if let Some(s) = value.as_str() {
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.timestamp_millis());
        }
        if let Ok(dt) = s.parse::<DateTime<Utc>>() {
            return Some(dt.timestamp_millis());
        }
    }
    if let Some(n) = value.as_i64() {
        return Some(if n < 10_000_000_000 { n * 1000 } else { n });
    }
    None
}

fn apply_line_to_context(line: &str, ctx: &mut ParseContext) {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return;
    };

    let top_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    if let Some(sid) = v.get("sessionId").and_then(|s| s.as_str()) {
        ctx.session_id = Some(sid.to_string());
    }

    if let Some(cwd) = v.get("cwd").and_then(|c| c.as_str()) {
        ctx.cwd = Some(cwd.to_string());
    }

    match top_type {
        "user" => {
            if ctx.title.is_none() {
                if let Some(text) = first_user_text(&v) {
                    ctx.title = Some(truncate_str(&text, 80));
                }
            }
        }
        "assistant" => {
            if let Some(message) = v.get("message") {
                if let Some(model) = message.get("model").and_then(|m| m.as_str()) {
                    ctx.model = Some(model.to_string());
                }
            }
        }
        _ => {}
    }
}

fn first_user_text(v: &Value) -> Option<String> {
    let message = v.get("message")?;
    let content = message.get("content")?;

    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }

    if let Some(arr) = content.as_array() {
        for item in arr {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
        }
    }

    None
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let end = s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len());
    format!("{}…", &s[..end])
}

fn parse_line(line: &str, ctx: &ParseContext) -> Option<UsageEventRow> {
    let v: Value = serde_json::from_str(line).ok()?;
    if v.get("type").and_then(|t| t.as_str())? != "assistant" {
        return None;
    }

    let message = v.get("message")?;
    let usage = message.get("usage")?;
    let session_id = v
        .get("sessionId")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .or_else(|| ctx.session_id.clone())?;

    let ts = v
        .get("timestamp")
        .and_then(parse_timestamp)
        .unwrap_or_else(|| Utc::now().timestamp_millis());

    let uuid = v.get("uuid").and_then(|u| u.as_str()).unwrap_or("unknown");

    let cache_read = usage
        .get("cache_read_input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let cache_create = usage
        .get("cache_creation_input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let input = usage
        .get("input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + cache_create;
    let output = usage
        .get("output_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let total = input + cache_read + output;

    let model = message
        .get("model")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
        .or_else(|| ctx.model.clone());

    Some(UsageEventRow {
        id: format!("claude:{session_id}:{uuid}"),
        source: SOURCE.to_string(),
        session_id,
        turn_id: None,
        ts,
        model,
        input_tokens: input,
        cached_input_tokens: cache_read,
        output_tokens: output,
        reasoning_output_tokens: 0,
        total_tokens: total,
        rollout_path: ctx.file_path.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_assistant_usage() {
        let line = r#"{"type":"assistant","sessionId":"sess-1","uuid":"u-1","timestamp":"2026-03-18T12:00:00Z","cwd":"/proj","message":{"model":"claude-sonnet","usage":{"input_tokens":100,"cache_read_input_tokens":20,"cache_creation_input_tokens":5,"output_tokens":30}}}"#;
        let ctx = ParseContext {
            session_id: Some("sess-1".into()),
            cwd: Some("/proj".into()),
            model: None,
            title: None,
            file_path: "/tmp/test.jsonl".into(),
        };
        let event = parse_line(line, &ctx).unwrap();
        assert_eq!(event.input_tokens, 105);
        assert_eq!(event.cached_input_tokens, 20);
        assert_eq!(event.output_tokens, 30);
        assert_eq!(event.total_tokens, 155);
        assert_eq!(event.model.as_deref(), Some("claude-sonnet"));
    }
}
