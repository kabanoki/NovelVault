use super::{clean_optional, ensure_exists, map_work, next_sort_order, now_utc, work_get_by_id};
use crate::{
    db::DbState,
    error::{CommandError, CommandResult},
    types::{IdArgs, Work, WorkCreateArgs, WorkListBySiteArgs, WorkUpdateArgs},
};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn work_create(db: State<DbState>, args: WorkCreateArgs) -> CommandResult<Work> {
    let title = args.title.trim();
    if title.is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "作品タイトルは必須です",
        ));
    }

    let conn = db.connection()?;
    ensure_exists(&conn, "sites", args.site_id, "サイト")?;

    let now = now_utc();
    let sort_order = next_sort_order(&conn, "works", "site_id", args.site_id)?;
    conn.execute(
        "INSERT INTO works (
            site_id, site_profile_id, title, author_name, description, source_url, sort_order, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            args.site_id,
            args.site_profile_id,
            title,
            clean_optional(args.author_name),
            clean_optional(args.description),
            clean_optional(args.source_url),
            sort_order,
            now,
            now,
        ],
    )?;

    work_get_by_id(&conn, conn.last_insert_rowid())
}

#[tauri::command]
pub fn work_list_by_site(db: State<DbState>, args: WorkListBySiteArgs) -> CommandResult<Vec<Work>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, site_id, site_profile_id, title, author_name, description, source_url,
                sort_order, created_at, updated_at
         FROM works
         WHERE site_id=?1
         ORDER BY sort_order, id",
    )?;

    let works = stmt
        .query_map(params![args.site_id], map_work)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(works)
}

#[tauri::command]
pub fn work_update(db: State<DbState>, args: WorkUpdateArgs) -> CommandResult<Work> {
    let title = args.title.trim();
    if title.is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "作品タイトルは必須です",
        ));
    }

    let conn = db.connection()?;
    let now = now_utc();
    let changed = conn.execute(
        "UPDATE works
         SET site_profile_id=?1, title=?2, author_name=?3, description=?4, source_url=?5, updated_at=?6
         WHERE id=?7",
        params![
            args.site_profile_id,
            title,
            clean_optional(args.author_name),
            clean_optional(args.description),
            clean_optional(args.source_url),
            now,
            args.id,
        ],
    )?;
    if changed == 0 {
        return Err(CommandError::new("NOT_FOUND", "作品が見つかりません"));
    }

    work_get_by_id(&conn, args.id)
}

#[tauri::command]
pub fn work_delete(db: State<DbState>, args: IdArgs) -> CommandResult<()> {
    let conn = db.connection()?;
    conn.execute("DELETE FROM works WHERE id=?1", params![args.id])?;
    Ok(())
}
