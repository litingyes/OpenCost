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

    let report = sync_claude_files(&db, &roots, true).expect("sync");
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
