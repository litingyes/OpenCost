use crate::codex::discover::discover_rollout_files;
use crate::db::{Database, UsageEventRow};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

const SOURCE: &str = "codex";

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncReport {
    pub files_scanned: usize,
    pub events_ingested: usize,
}

pub fn sync_codex_files(
    db: &Database,
    home: &Path,
    force: bool,
    since_ms: Option<i64>,
) -> Result<SyncReport, String> {
    if force {
        db.reset_cursors().map_err(|e| e.to_string())?;
    }

    let files = discover_rollout_files(home);
    let mut events_ingested = 0usize;

    for path in &files {
        let ingested =
            parse_rollout_file(db, path, force, since_ms).map_err(|e| e.to_string())?;
        events_ingested += ingested;
    }

    Ok(SyncReport {
        files_scanned: files.len(),
        events_ingested,
    })
}

fn parse_rollout_file(
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
        session_id: fallback_session_id.clone(),
        turn_id: None,
        model: None,
        cwd: None,
        originator: None,
        git_repo: None,
        title: None,
        rollout_path: file_path.clone(),
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
            ctx.originator.as_deref(),
            ctx.git_repo.as_deref(),
            Some(&file_path),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(ingested)
}

struct ParseContext {
    session_id: Option<String>,
    turn_id: Option<String>,
    model: Option<String>,
    cwd: Option<String>,
    originator: Option<String>,
    git_repo: Option<String>,
    title: Option<String>,
    rollout_path: String,
}

fn session_id_from_filename(path: &Path) -> Option<String> {
    let name = path.file_stem()?.to_str()?;
    // rollout-2026-03-17T23-00-03-019cfc4f-a27e-7590-9149-d6ea7d0b8450
    if name.len() >= 36 {
        let candidate = &name[name.len() - 36..];
        if candidate.chars().filter(|c| *c == '-').count() == 4 {
            return Some(candidate.to_string());
        }
    }
    None
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
    let ts = v.get("timestamp").and_then(parse_timestamp);

    match top_type {
        "session_meta" => {
            if let Some(payload) = v.get("payload") {
                if let Some(id) = payload.get("id").and_then(|i| i.as_str()) {
                    ctx.session_id = Some(id.to_string());
                }
                if let Some(cwd) = payload.get("cwd").and_then(|c| c.as_str()) {
                    ctx.cwd = Some(cwd.to_string());
                }
                if let Some(originator) = payload.get("originator").and_then(|o| o.as_str()) {
                    ctx.originator = Some(originator.to_string());
                }
                if let Some(git) = payload.get("git") {
                    if let Some(url) = git.get("repository_url").and_then(|u| u.as_str()) {
                        ctx.git_repo = Some(url.to_string());
                    }
                }
            }
        }
        "turn_context" => {
            if let Some(payload) = v.get("payload") {
                if let Some(turn) = payload.get("turn_id").and_then(|t| t.as_str()) {
                    ctx.turn_id = Some(turn.to_string());
                }
                if let Some(model) = payload.get("model").and_then(|m| m.as_str()) {
                    ctx.model = Some(model.to_string());
                }
                if let Some(cwd) = payload.get("cwd").and_then(|c| c.as_str()) {
                    ctx.cwd = Some(cwd.to_string());
                }
            }
        }
        "event_msg" => {
            if let Some(payload) = v.get("payload") {
                let inner = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if inner == "task_started" {
                    if let Some(turn) = payload.get("turn_id").and_then(|t| t.as_str()) {
                        ctx.turn_id = Some(turn.to_string());
                    }
                }
            }
        }
        "response_item" => {
            if let Some(payload) = v.get("payload") {
                if payload.get("role").and_then(|r| r.as_str()) == Some("user")
                    || payload.get("type").and_then(|t| t.as_str()) == Some("message")
                        && payload.get("role").and_then(|r| r.as_str()) == Some("user")
                {
                    if ctx.title.is_none() {
                        if let Some(text) = first_user_text(payload) {
                            ctx.title = Some(truncate_str(&text, 80));
                        }
                    }
                }
            }
            let _ = ts;
        }
        _ => {}
    }

    // Legacy flat format session hints
    if top_type == "task_started" {
        if let Some(model) = v.get("model").and_then(|m| m.as_str()) {
            ctx.model = Some(model.to_string());
        }
    }
}

fn first_user_text(payload: &Value) -> Option<String> {
    let content = payload.get("content")?.as_array()?;
    for item in content {
        if item.get("type").and_then(|t| t.as_str()) == Some("input_text") {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                if !text.starts_with('#') && !text.starts_with('<') {
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
    let top_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let ts = v
        .get("timestamp")
        .or_else(|| v.get("ts"))
        .and_then(parse_timestamp)
        .unwrap_or_else(|| Utc::now().timestamp_millis());

    let session_id = ctx.session_id.clone()?;

    if top_type == "event_msg" {
        let payload = v.get("payload")?;
        if payload.get("type").and_then(|t| t.as_str())? != "token_count" {
            return None;
        }
        let info = payload.get("info")?;
        let usage = info.get("last_token_usage").or_else(|| info.get("total_token_usage"))?;
        return Some(build_event(session_id, ctx, ts, usage));
    }

    if top_type == "token_usage" || top_type == "token_count" {
        let usage = v
            .get("usage")
            .or_else(|| v.get("info").and_then(|i| i.get("last_token_usage")))?;
        return Some(build_event(session_id, ctx, ts, usage));
    }

    None
}

fn build_event(session_id: String, ctx: &ParseContext, ts: i64, usage: &Value) -> UsageEventRow {
    let input = usage.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
    let cached = usage
        .get("cached_input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let output = usage.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
    let reasoning = usage
        .get("reasoning_output_tokens")
        .or_else(|| usage.get("reasoning_tokens"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let total = usage
        .get("total_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(input + output);

    let turn_id = ctx.turn_id.clone();
    let id = format!(
        "{}:{}:{}",
        session_id,
        turn_id.as_deref().unwrap_or("turn"),
        ts
    );

    UsageEventRow {
        id,
        source: SOURCE.to_string(),
        session_id,
        turn_id,
        ts,
        model: ctx.model.clone(),
        input_tokens: input,
        cached_input_tokens: cached,
        output_tokens: output,
        reasoning_output_tokens: reasoning,
        total_tokens: total,
        rollout_path: ctx.rollout_path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_token_count_envelope() {
        let line = r#"{"timestamp":"2026-03-18T12:57:39.881Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":20,"output_tokens":30,"reasoning_output_tokens":5,"total_tokens":130}}}}"#;
        let ctx = ParseContext {
            session_id: Some("sess-1".into()),
            turn_id: Some("turn-1".into()),
            model: Some("gpt-5.4".into()),
            cwd: None,
            originator: None,
            git_repo: None,
            title: None,
            rollout_path: "/tmp/test.jsonl".into(),
        };
        let event = parse_line(line, &ctx).unwrap();
        assert_eq!(event.input_tokens, 100);
        assert_eq!(event.output_tokens, 30);
        assert_eq!(event.total_tokens, 130);
    }

    #[test]
    fn parses_legacy_token_usage() {
        let line = r#"{"type":"token_usage","timestamp":"2026-03-18T12:57:39.881Z","usage":{"input_tokens":50,"output_tokens":10,"total_tokens":60}}"#;
        let ctx = ParseContext {
            session_id: Some("sess-2".into()),
            turn_id: None,
            model: None,
            cwd: None,
            originator: None,
            git_repo: None,
            title: None,
            rollout_path: "/tmp/legacy.jsonl".into(),
        };
        let event = parse_line(line, &ctx).unwrap();
        assert_eq!(event.input_tokens, 50);
        assert_eq!(event.total_tokens, 60);
    }
}
