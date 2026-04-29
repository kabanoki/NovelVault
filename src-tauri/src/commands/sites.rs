use super::now_utc;
use crate::{
    db::DbState,
    error::{CommandError, CommandResult},
    types::{IdArgs, Site, SiteCreateArgs, SiteUpdateArgs},
};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn site_create(db: State<DbState>, args: SiteCreateArgs) -> CommandResult<Site> {
    let name = args.name.trim();
    let base_url = args.base_url.trim();

    if name.is_empty() {
        return Err(CommandError::new("VALIDATION_ERROR", "サイト名は必須です"));
    }
    if base_url.is_empty() {
        return Err(CommandError::new("VALIDATION_ERROR", "base_url は必須です"));
    }

    let conn = db.connection()?;
    let now = now_utc();
    conn.execute(
        "INSERT INTO sites (name, base_url, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, base_url, now, now],
    )?;

    Ok(Site {
        id: conn.last_insert_rowid(),
        name: name.to_string(),
        base_url: base_url.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn site_list(db: State<DbState>) -> CommandResult<Vec<Site>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, base_url, created_at, updated_at
         FROM sites
         ORDER BY name COLLATE NOCASE, id",
    )?;

    let sites = stmt
        .query_map([], |row| {
            Ok(Site {
                id: row.get(0)?,
                name: row.get(1)?,
                base_url: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sites)
}

#[tauri::command]
pub fn site_update(db: State<DbState>, args: SiteUpdateArgs) -> CommandResult<Site> {
    let name = args.name.trim();
    let base_url = args.base_url.trim();

    if name.is_empty() {
        return Err(CommandError::new("VALIDATION_ERROR", "サイト名は必須です"));
    }
    if base_url.is_empty() {
        return Err(CommandError::new("VALIDATION_ERROR", "base_url は必須です"));
    }

    let conn = db.connection()?;
    let now = now_utc();
    let changed = conn.execute(
        "UPDATE sites SET name=?1, base_url=?2, updated_at=?3 WHERE id=?4",
        params![name, base_url, now, args.id],
    )?;
    if changed == 0 {
        return Err(CommandError::new("NOT_FOUND", "サイトが見つかりません"));
    }

    Ok(Site {
        id: args.id,
        name: name.to_string(),
        base_url: base_url.to_string(),
        created_at: conn.query_row(
            "SELECT created_at FROM sites WHERE id=?1",
            params![args.id],
            |row| row.get(0),
        )?,
        updated_at: now,
    })
}

#[tauri::command]
pub fn site_delete(db: State<DbState>, args: IdArgs) -> CommandResult<()> {
    let conn = db.connection()?;
    conn.execute("DELETE FROM sites WHERE id=?1", params![args.id])?;
    Ok(())
}
