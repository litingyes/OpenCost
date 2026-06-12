use crate::db::Database;
use rusqlite::Connection;
use std::path::Path;

pub fn enrich_from_state_db(db: &Database, codex_home: &Path) -> Result<(), String> {
    let state_path = codex_home.join("state_5.sqlite");
    if !state_path.exists() {
        return Ok(());
    }

    let conn = Connection::open_with_flags(
        &state_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, cwd, model, rollout_path, first_user_message, git_origin_url
             FROM threads",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (id, title, cwd, model, _rollout, first_msg, git) =
            row.map_err(|e| e.to_string())?;
        db.enrich_session_from_threads(
            &id,
            title.as_deref(),
            cwd.as_deref(),
            model.as_deref(),
            git.as_deref(),
            first_msg.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
