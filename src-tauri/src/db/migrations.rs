use chrono::Utc;
use rusqlite::{params, Connection};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001",
        include_str!("../../migrations/001_initial_schema.sql"),
    ),
    (
        "002",
        include_str!("../../migrations/002_add_favorites.sql"),
    ),
    (
        "003",
        include_str!("../../migrations/003_add_wayback_metadata.sql"),
    ),
    (
        "004",
        include_str!("../../migrations/004_add_profile_indexes.sql"),
    ),
    (
        "005",
        include_str!("../../migrations/005_use_trigram_fts.sql"),
    ),
    (
        "006",
        include_str!("../../migrations/006_unique_page_source_urls.sql"),
    ),
];

pub fn run(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT NOT NULL PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    for (version, sql) in MIGRATIONS {
        if is_applied(conn, version)? {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
            params![version, now_utc()],
        )?;
        tx.commit()?;
    }

    Ok(())
}

fn is_applied(conn: &Connection, version: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version=?1",
        params![version],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn now_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
