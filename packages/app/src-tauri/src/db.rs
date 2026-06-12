use chrono::{Local, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct UsageEventRow {
    pub id: String,
    pub source: String,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub ts: i64,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
    pub rollout_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRow {
    pub id: String,
    pub source: String,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
    pub turn_count: i64,
    pub originator: Option<String>,
    pub git_repo: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimeseriesPoint {
    pub ts: i64,
    pub input: i64,
    pub cached: i64,
    pub output: i64,
    pub reasoning: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionBreakdownItem {
    pub id: String,
    pub source: String,
    pub title: Option<String>,
    pub total_tokens: i64,
    pub model: Option<String>,
    pub cwd: Option<String>,
    pub started_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub display_name: String,
    pub home: String,
    pub enabled: bool,
    pub session_count: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatus {
    pub last_sync: Option<i64>,
    pub providers: Vec<ProviderInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SyncSettings {
    pub preset: String,
    pub custom_since_ms: Option<i64>,
    pub effective_since_ms: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncWindowChange {
    None,
    Shrunk,
    Expanded,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> SqlResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS usage_events (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                session_id TEXT NOT NULL,
                turn_id TEXT,
                ts INTEGER NOT NULL,
                model TEXT,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                cached_input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                reasoning_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                rollout_path TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_usage_events_ts ON usage_events(ts);
            CREATE INDEX IF NOT EXISTS idx_usage_events_session ON usage_events(session_id);

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                title TEXT,
                cwd TEXT,
                model TEXT,
                started_at INTEGER,
                ended_at INTEGER,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                cached_input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                reasoning_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                turn_count INTEGER NOT NULL DEFAULT 0,
                originator TEXT,
                git_repo TEXT,
                rollout_path TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_total ON sessions(total_tokens DESC);

            CREATE TABLE IF NOT EXISTS ingest_cursors (
                file_path TEXT PRIMARY KEY,
                byte_offset INTEGER NOT NULL DEFAULT 0,
                mtime INTEGER NOT NULL DEFAULT 0,
                last_synced_at INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS sync_meta (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_sync_at INTEGER
            );
            INSERT OR IGNORE INTO sync_meta (id, last_sync_at) VALUES (1, NULL);

            CREATE TABLE IF NOT EXISTS provider_settings (
                id TEXT PRIMARY KEY,
                enabled INTEGER NOT NULL DEFAULT 1,
                updated_at INTEGER NOT NULL
            );
            ",
        )?;
        self.migrate_sync_settings()?;
        Ok(())
    }

    fn migrate_sync_settings(&self) -> SqlResult<()> {
        let has_preset: bool = self
            .conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('sync_meta') WHERE name = 'sync_since_preset'")?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|n| n > 0)
            .unwrap_or(false);

        if !has_preset {
            self.conn.execute_batch(
                "ALTER TABLE sync_meta ADD COLUMN sync_since_preset TEXT NOT NULL DEFAULT '7d';
                 ALTER TABLE sync_meta ADD COLUMN sync_since_custom_ms INTEGER;",
            )?;
        }
        Ok(())
    }

    pub fn seed_provider_settings(&self, provider_ids: &[&str]) -> SqlResult<()> {
        let now = Utc::now().timestamp_millis();
        for id in provider_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO provider_settings (id, enabled, updated_at) VALUES (?1, 1, ?2)",
                params![id, now],
            )?;
        }
        Ok(())
    }

    pub fn is_provider_enabled(&self, id: &str) -> SqlResult<bool> {
        let enabled: Option<i64> = self
            .conn
            .query_row(
                "SELECT enabled FROM provider_settings WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(enabled.unwrap_or(1) != 0)
    }

    pub fn set_provider_enabled(&self, id: &str, enabled: bool) -> SqlResult<()> {
        let now = Utc::now().timestamp_millis();
        self.conn.execute(
            "INSERT INTO provider_settings (id, enabled, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET enabled = excluded.enabled, updated_at = excluded.updated_at",
            params![id, enabled as i64, now],
        )?;
        Ok(())
    }

    pub fn enabled_source_ids(&self) -> SqlResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM provider_settings WHERE enabled = 1 ORDER BY id")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    pub fn provider_stats(&self, source: &str) -> SqlResult<(i64, i64)> {
        let session_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE source = ?1",
            params![source],
            |row| row.get(0),
        )?;
        let total_tokens: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM usage_events WHERE source = ?1",
            params![source],
            |row| row.get(0),
        )?;
        Ok((session_count, total_tokens))
    }

    pub fn get_last_sync(&self) -> SqlResult<Option<i64>> {
        self.conn.query_row(
            "SELECT last_sync_at FROM sync_meta WHERE id = 1",
            [],
            |row| row.get(0),
        )
    }

    pub fn get_cursor(&self, file_path: &str) -> SqlResult<Option<(i64, i64)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT byte_offset, mtime FROM ingest_cursors WHERE file_path = ?1")?;
        let mut rows = stmt.query(params![file_path])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }

    pub fn set_cursor(&self, file_path: &str, byte_offset: i64, mtime: i64) -> SqlResult<()> {
        let now = Utc::now().timestamp_millis();
        self.conn.execute(
            "INSERT INTO ingest_cursors (file_path, byte_offset, mtime, last_synced_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(file_path) DO UPDATE SET
               byte_offset = excluded.byte_offset,
               mtime = excluded.mtime,
               last_synced_at = excluded.last_synced_at",
            params![file_path, byte_offset, mtime, now],
        )?;
        Ok(())
    }

    pub fn insert_usage_event(&self, event: &UsageEventRow) -> SqlResult<bool> {
        let changed = self.conn.execute(
            "INSERT OR IGNORE INTO usage_events
             (id, source, session_id, turn_id, ts, model, input_tokens, cached_input_tokens,
              output_tokens, reasoning_output_tokens, total_tokens, rollout_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                event.id,
                event.source,
                event.session_id,
                event.turn_id,
                event.ts,
                event.model,
                event.input_tokens,
                event.cached_input_tokens,
                event.output_tokens,
                event.reasoning_output_tokens,
                event.total_tokens,
                event.rollout_path,
            ],
        )?;
        Ok(changed > 0)
    }

    pub fn upsert_session_meta(
        &self,
        id: &str,
        source: &str,
        title: Option<&str>,
        cwd: Option<&str>,
        model: Option<&str>,
        originator: Option<&str>,
        git_repo: Option<&str>,
        rollout_path: Option<&str>,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, source, title, cwd, model, originator, git_repo, rollout_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
               title = COALESCE(excluded.title, sessions.title),
               cwd = COALESCE(excluded.cwd, sessions.cwd),
               model = COALESCE(excluded.model, sessions.model),
               originator = COALESCE(excluded.originator, sessions.originator),
               git_repo = COALESCE(excluded.git_repo, sessions.git_repo),
               rollout_path = COALESCE(excluded.rollout_path, sessions.rollout_path)",
            params![id, source, title, cwd, model, originator, git_repo, rollout_path],
        )?;
        Ok(())
    }

    pub fn recompute_session_aggregates(&self, source: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, source, started_at, ended_at, input_tokens, cached_input_tokens,
              output_tokens, reasoning_output_tokens, total_tokens, turn_count)
             SELECT
               session_id, source,
               MIN(ts), MAX(ts),
               SUM(input_tokens), SUM(cached_input_tokens), SUM(output_tokens),
               SUM(reasoning_output_tokens), SUM(total_tokens), COUNT(*)
             FROM usage_events
             WHERE source = ?1
             GROUP BY session_id
             ON CONFLICT(id) DO UPDATE SET
               started_at = excluded.started_at,
               ended_at = excluded.ended_at,
               input_tokens = excluded.input_tokens,
               cached_input_tokens = excluded.cached_input_tokens,
               output_tokens = excluded.output_tokens,
               reasoning_output_tokens = excluded.reasoning_output_tokens,
               total_tokens = excluded.total_tokens,
               turn_count = excluded.turn_count",
            params![source],
        )?;
        Ok(())
    }

    pub fn enrich_session_from_threads(
        &self,
        id: &str,
        title: Option<&str>,
        cwd: Option<&str>,
        model: Option<&str>,
        git_repo: Option<&str>,
        first_message: Option<&str>,
    ) -> SqlResult<()> {
        let display_title = title
            .filter(|t| !t.is_empty())
            .or(first_message.map(|m| truncate(m, 80)))
            .map(|s| s.to_string());

        self.conn.execute(
            "UPDATE sessions SET
               title = COALESCE(?2, title),
               cwd = COALESCE(?3, cwd),
               model = COALESCE(?4, model),
               git_repo = COALESCE(?5, git_repo)
             WHERE id = ?1",
            params![id, display_title, cwd, model, git_repo],
        )?;
        Ok(())
    }

    pub fn set_last_sync(&self) -> SqlResult<()> {
        let now = Utc::now().timestamp_millis();
        self.conn.execute(
            "UPDATE sync_meta SET last_sync_at = ?1 WHERE id = 1",
            params![now],
        )?;
        Ok(())
    }

    pub fn preset_duration_ms(preset: &str) -> Option<i64> {
        match preset {
            "1d" | "24h" => Some(24 * 3_600_000),
            "7d" => Some(7 * 24 * 3_600_000),
            "30d" => Some(30 * 24 * 3_600_000),
            "180d" => Some(180 * 24 * 3_600_000),
            "365d" => Some(365 * 24 * 3_600_000),
            "all" | "custom" => None,
            _ => Some(7 * 24 * 3_600_000),
        }
    }

    pub fn effective_since_ms(preset: &str, custom_since_ms: Option<i64>) -> Option<i64> {
        if preset == "all" {
            return None;
        }
        if preset == "custom" {
            return custom_since_ms;
        }
        let now = Local::now().timestamp_millis();
        Self::preset_duration_ms(preset).map(|duration| now - duration)
    }

    pub fn get_sync_settings(&self) -> SqlResult<SyncSettings> {
        let (preset, custom_since_ms): (String, Option<i64>) = self.conn.query_row(
            "SELECT sync_since_preset, sync_since_custom_ms FROM sync_meta WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let effective_since_ms = Self::effective_since_ms(&preset, custom_since_ms);
        Ok(SyncSettings {
            preset,
            custom_since_ms,
            effective_since_ms,
        })
    }

    pub fn set_sync_settings(
        &self,
        preset: &str,
        custom_since_ms: Option<i64>,
    ) -> SqlResult<SyncWindowChange> {
        let old = self.get_sync_settings()?;
        let new_effective = Self::effective_since_ms(preset, custom_since_ms);

        let change = if old.preset == preset && old.custom_since_ms == custom_since_ms {
            SyncWindowChange::None
        } else {
            Self::compare_sync_windows(old.effective_since_ms, new_effective)
        };

        self.conn.execute(
            "UPDATE sync_meta SET sync_since_preset = ?1, sync_since_custom_ms = ?2 WHERE id = 1",
            params![preset, custom_since_ms],
        )?;
        match change {
            SyncWindowChange::Shrunk => {
                if let Some(since) = new_effective {
                    self.purge_events_before(since)?;
                }
            }
            SyncWindowChange::Expanded => {
                self.reset_cursors()?;
            }
            SyncWindowChange::None => {}
        }
        Ok(change)
    }

    fn compare_sync_windows(old: Option<i64>, new: Option<i64>) -> SyncWindowChange {
        match (old, new) {
            (None, Some(_)) => SyncWindowChange::Shrunk,
            (Some(_old_ms), None) => SyncWindowChange::Expanded,
            (Some(old_ms), Some(new_ms)) if new_ms > old_ms => SyncWindowChange::Shrunk,
            (Some(old_ms), Some(new_ms)) if new_ms < old_ms => SyncWindowChange::Expanded,
            _ => SyncWindowChange::None,
        }
    }

    pub fn purge_events_before(&self, since_ms: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM usage_events WHERE ts < ?1", params![since_ms])?;
        self.conn.execute(
            "DELETE FROM sessions
             WHERE id NOT IN (SELECT DISTINCT session_id FROM usage_events)",
            [],
        )?;
        for source in self.enabled_source_ids()? {
            self.recompute_session_aggregates(&source)?;
        }
        Ok(())
    }

    pub fn range_start_ms(range: &str) -> i64 {
        let now = Local::now().timestamp_millis();
        match range {
            "1d" | "24h" => now - 24 * 3_600_000,
            "30d" => now - 30 * 24 * 3_600_000,
            "180d" => now - 180 * 24 * 3_600_000,
            "365d" => now - 365 * 24 * 3_600_000,
            "7d" => now - 7 * 24 * 3_600_000,
            _ => now - 7 * 24 * 3_600_000,
        }
    }

    pub fn get_usage_timeseries(
        &self,
        range: &str,
        bucket: &str,
        source: &str,
    ) -> SqlResult<Vec<TimeseriesPoint>> {
        let start = Self::range_start_ms(range);
        let divisor = if bucket == "day" {
            86_400_000_i64
        } else {
            3_600_000_i64
        };

        if source == "all" {
            let sources = self.enabled_source_ids()?;
            if sources.is_empty() {
                return Ok(Vec::new());
            }
            let placeholders = sources
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!(
                "SELECT (ts / {divisor}) * {divisor} AS bucket_ts,
                        SUM(input_tokens), SUM(cached_input_tokens), SUM(output_tokens),
                        SUM(reasoning_output_tokens), SUM(total_tokens)
                 FROM usage_events
                 WHERE ts >= ?1 AND source IN ({placeholders})
                 GROUP BY bucket_ts
                 ORDER BY bucket_ts"
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(start)];
            for s in &sources {
                params.push(Box::new(s.clone()));
            }
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(TimeseriesPoint {
                    ts: row.get(0)?,
                    input: row.get(1)?,
                    cached: row.get(2)?,
                    output: row.get(3)?,
                    reasoning: row.get(4)?,
                    total: row.get(5)?,
                })
            })?;
            return rows.collect();
        }

        let sql = format!(
            "SELECT (ts / {divisor}) * {divisor} AS bucket_ts,
                    SUM(input_tokens), SUM(cached_input_tokens), SUM(output_tokens),
                    SUM(reasoning_output_tokens), SUM(total_tokens)
             FROM usage_events
             WHERE ts >= ?1 AND source = ?2
             GROUP BY bucket_ts
             ORDER BY bucket_ts"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![start, source], |row| {
            Ok(TimeseriesPoint {
                ts: row.get(0)?,
                input: row.get(1)?,
                cached: row.get(2)?,
                output: row.get(3)?,
                reasoning: row.get(4)?,
                total: row.get(5)?,
            })
        })?;

        rows.collect()
    }

    pub fn get_session_breakdown(
        &self,
        range: &str,
        limit: i64,
        source: &str,
    ) -> SqlResult<Vec<SessionBreakdownItem>> {
        let start = Self::range_start_ms(range);

        if source == "all" {
            let sources = self.enabled_source_ids()?;
            if sources.is_empty() {
                return Ok(Vec::new());
            }
            let placeholders = sources
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!(
                "SELECT e.session_id, e.source, s.title, SUM(e.total_tokens), s.model, s.cwd, MIN(e.ts)
                 FROM usage_events e
                 LEFT JOIN sessions s ON s.id = e.session_id
                 WHERE e.ts >= ?1 AND e.source IN ({placeholders})
                 GROUP BY e.session_id, e.source
                 HAVING SUM(e.total_tokens) > 0
                 ORDER BY SUM(e.total_tokens) DESC
                 LIMIT ?{}",
                sources.len() + 2
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(start)];
            for s in &sources {
                params.push(Box::new(s.clone()));
            }
            params.push(Box::new(limit));
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(SessionBreakdownItem {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    title: row.get(2)?,
                    total_tokens: row.get(3)?,
                    model: row.get(4)?,
                    cwd: row.get(5)?,
                    started_at: row.get(6)?,
                })
            })?;
            return rows.collect();
        }

        let mut stmt = self.conn.prepare(
            "SELECT e.session_id, e.source, s.title, SUM(e.total_tokens), s.model, s.cwd, MIN(e.ts)
             FROM usage_events e
             LEFT JOIN sessions s ON s.id = e.session_id
             WHERE e.source = ?1 AND e.ts >= ?2
             GROUP BY e.session_id, e.source
             HAVING SUM(e.total_tokens) > 0
             ORDER BY SUM(e.total_tokens) DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![source, start, limit], |row| {
            Ok(SessionBreakdownItem {
                id: row.get(0)?,
                source: row.get(1)?,
                title: row.get(2)?,
                total_tokens: row.get(3)?,
                model: row.get(4)?,
                cwd: row.get(5)?,
                started_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_session_detail(&self, session_id: &str) -> SqlResult<Option<SessionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, title, cwd, model, started_at, ended_at,
                    input_tokens, cached_input_tokens, output_tokens,
                    reasoning_output_tokens, total_tokens, turn_count, originator, git_repo
             FROM sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(SessionRow {
                id: row.get(0)?,
                source: row.get(1)?,
                title: row.get(2)?,
                cwd: row.get(3)?,
                model: row.get(4)?,
                started_at: row.get(5)?,
                ended_at: row.get(6)?,
                input_tokens: row.get(7)?,
                cached_input_tokens: row.get(8)?,
                output_tokens: row.get(9)?,
                reasoning_output_tokens: row.get(10)?,
                total_tokens: row.get(11)?,
                turn_count: row.get(12)?,
                originator: row.get(13)?,
                git_repo: row.get(14)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_session_events(&self, session_id: &str) -> SqlResult<Vec<UsageEventRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, session_id, turn_id, ts, model, input_tokens, cached_input_tokens,
                    output_tokens, reasoning_output_tokens, total_tokens, rollout_path
             FROM usage_events WHERE session_id = ?1 ORDER BY ts ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(UsageEventRow {
                id: row.get(0)?,
                source: row.get(1)?,
                session_id: row.get(2)?,
                turn_id: row.get(3)?,
                ts: row.get(4)?,
                model: row.get(5)?,
                input_tokens: row.get(6)?,
                cached_input_tokens: row.get(7)?,
                output_tokens: row.get(8)?,
                reasoning_output_tokens: row.get(9)?,
                total_tokens: row.get(10)?,
                rollout_path: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    pub fn reset_cursors(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM ingest_cursors", [])?;
        Ok(())
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.chars().count() <= max {
        return s;
    }
    let end = s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len());
    &s[..end]
}
