use app_lib::codex::codex_home;
use app_lib::db::Database;
use app_lib::providers::{all_provider_ids, sync_enabled_providers};

#[test]
fn sync_local_codex_rollouts() {
    let home = codex_home();
    if !home.join("sessions").exists() {
        eprintln!("Skipping: no Codex sessions at {}", home.display());
        return;
    }

    let db_path = std::env::temp_dir().join("opencost-sync-test.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::open(&db_path).expect("open db");
    db.seed_provider_settings(&all_provider_ids())
        .expect("seed");

    let reports = sync_enabled_providers(&db, true).expect("sync");
    let codex = reports.iter().find(|r| r.id == "codex").expect("codex report");
    assert!(codex.files_scanned > 0, "expected rollout files");

    let (sessions, tokens) = db.provider_stats("codex").expect("stats");
    eprintln!(
        "Synced {} files, ingested {} events, {} sessions, {} total tokens",
        codex.files_scanned, codex.events_ingested, sessions, tokens
    );

    let _ = std::fs::remove_file(&db_path);
}
