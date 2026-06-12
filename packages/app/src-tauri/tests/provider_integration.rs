use app_lib::claude::sync_claude_files;
use app_lib::db::Database;
use app_lib::providers::{all_provider_ids, enabled_providers, sync_enabled_providers};
use std::path::PathBuf;

#[test]
fn provider_settings_seed_and_toggle() {
    let db_path = std::env::temp_dir().join("opencost-provider-test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::open(&db_path).expect("open db");

    db.seed_provider_settings(&all_provider_ids())
        .expect("seed");

    assert!(db.is_provider_enabled("codex").unwrap());
    assert!(db.is_provider_enabled("claude").unwrap());

    db.set_provider_enabled("claude", false).expect("disable");
    let enabled = enabled_providers(&db).expect("enabled");
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].id(), "codex");

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn claude_fixture_sync() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude");
    let roots = vec![fixture_root];
    let db_path = std::env::temp_dir().join("opencost-claude-fixture-test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::open(&db_path).expect("open db");

    let report = sync_claude_files(&db, &roots, true, None).expect("sync");
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.events_ingested, 1);

    db.recompute_session_aggregates("claude").expect("aggregate");
    let (sessions, tokens) = db.provider_stats("claude").expect("stats");
    assert_eq!(sessions, 1);
    assert_eq!(tokens, 135);

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn sync_enabled_providers_runs_without_error() {
    let db_path = std::env::temp_dir().join("opencost-sync-all-test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::open(&db_path).expect("open db");
    db.seed_provider_settings(&all_provider_ids())
        .expect("seed");

    let reports = sync_enabled_providers(&db, false).expect("sync all");
    assert_eq!(reports.len(), 2);

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn claude_fixture_sync_respects_since_ms() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude");
    let roots = vec![fixture_root];
    let db_path = std::env::temp_dir().join("opencost-claude-since-test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::open(&db_path).expect("open db");

    // Fixture assistant event is at 2026-03-18T10:00:01Z
    let after_event = chrono::DateTime::parse_from_rfc3339("2026-03-19T00:00:00Z")
        .unwrap()
        .timestamp_millis();
    let report = sync_claude_files(&db, &roots, true, Some(after_event)).expect("sync");
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.events_ingested, 0);

    let before_event = chrono::DateTime::parse_from_rfc3339("2026-03-18T00:00:00Z")
        .unwrap()
        .timestamp_millis();
    let report = sync_claude_files(&db, &roots, true, Some(before_event)).expect("sync");
    assert_eq!(report.events_ingested, 1);

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn sync_settings_shrink_purges_old_events() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude");
    let roots = vec![fixture_root];
    let db_path = std::env::temp_dir().join("opencost-sync-shrink-test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::open(&db_path).expect("open db");
    db.set_sync_settings("all", None).expect("all history");

    sync_claude_files(&db, &roots, true, None).expect("sync");
    let (_, tokens_before) = db.provider_stats("claude").expect("stats");
    assert_eq!(tokens_before, 135);

    let after_event = chrono::DateTime::parse_from_rfc3339("2026-03-19T00:00:00Z")
        .unwrap()
        .timestamp_millis();
    db.set_sync_settings("custom", Some(after_event))
        .expect("shrink");

    let (_, tokens_after) = db.provider_stats("claude").expect("stats");
    assert_eq!(tokens_after, 0);

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn sync_settings_expand_resets_cursors() {
    let db_path = std::env::temp_dir().join("opencost-sync-expand-test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::open(&db_path).expect("open db");

    db.set_sync_settings("1d", None).expect("set 1d");
    db.set_cursor("/tmp/example.jsonl", 100, 1).expect("cursor");

    db.set_sync_settings("all", None)
        .expect("expand to all");
    let cursor = db.get_cursor("/tmp/example.jsonl").expect("cursor");
    assert!(cursor.is_none());

    let _ = std::fs::remove_file(&db_path);
}
