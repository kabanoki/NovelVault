use super::{
    clean_optional, ensure_exists, ensure_unique_page_source_url, map_page, next_sort_order,
    now_utc, page_get_by_id, validate_source_type, wayback_metadata,
};
use crate::{
    db::DbState,
    error::{CommandError, CommandResult},
    types::{IdArgs, Page, PageCreateArgs, PageListByWorkArgs, PageUpdateArgs},
};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn page_create(db: State<DbState>, args: PageCreateArgs) -> CommandResult<Page> {
    validate_source_type(&args.source_type)?;
    let wayback = wayback_metadata(args.source_url.as_deref(), &args.source_type)?;
    let source_url = clean_optional(args.source_url);
    let title = clean_optional(args.title);
    let requested_encoding = clean_optional(args.requested_encoding);
    let content_text = clean_optional(args.content_text);
    let conn = db.connection()?;
    ensure_exists(&conn, "works", args.work_id, "作品")?;
    ensure_unique_page_source_url(
        &conn,
        args.work_id,
        None,
        source_url.as_deref(),
        &args.source_type,
    )?;

    let now = now_utc();
    let sort_order = next_sort_order(&conn, "pages", "work_id", args.work_id)?;
    let fetch_status = if content_text.is_some() {
        "success"
    } else {
        "pending"
    };

    conn.execute(
        "INSERT INTO pages (
            work_id, page_number, sort_order, title, source_url, source_type,
            canonical_url, archived_at, requested_encoding, content_text, fetch_status, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            args.work_id,
            args.page_number,
            sort_order,
            title,
            source_url,
            args.source_type,
            wayback.canonical_url,
            wayback.archived_at,
            requested_encoding,
            content_text,
            fetch_status,
            now,
            now,
        ],
    )?;

    page_get_by_id(&conn, conn.last_insert_rowid())
}

#[tauri::command]
pub fn page_list_by_work(db: State<DbState>, args: PageListByWorkArgs) -> CommandResult<Vec<Page>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, work_id, page_number, sort_order, title, source_url, source_type,
                canonical_url, archived_at, requested_encoding, detected_encoding, content_text, content_html_path,
                fetch_status, fetch_error, fetched_at, created_at, updated_at
         FROM pages
         WHERE work_id=?1
         ORDER BY sort_order, id",
    )?;

    let pages = stmt
        .query_map(params![args.work_id], map_page)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(pages)
}

#[tauri::command]
pub fn page_get(db: State<DbState>, args: IdArgs) -> CommandResult<Page> {
    let conn = db.connection()?;
    page_get_by_id(&conn, args.id)
}

#[tauri::command]
pub fn page_update(db: State<DbState>, args: PageUpdateArgs) -> CommandResult<Page> {
    validate_source_type(&args.source_type)?;
    let wayback = wayback_metadata(args.source_url.as_deref(), &args.source_type)?;
    let source_url = clean_optional(args.source_url);
    let title = clean_optional(args.title);
    let requested_encoding = clean_optional(args.requested_encoding);
    let content_text = clean_optional(args.content_text);
    let conn = db.connection()?;
    let page = page_get_by_id(&conn, args.id)?;
    ensure_unique_page_source_url(
        &conn,
        page.work_id,
        Some(args.id),
        source_url.as_deref(),
        &args.source_type,
    )?;
    let now = now_utc();
    let fetch_status = if content_text.is_some() {
        "success"
    } else {
        "pending"
    };
    let changed = conn.execute(
        "UPDATE pages
         SET page_number=?1, title=?2, source_url=?3, source_type=?4,
             canonical_url=?5, archived_at=?6, requested_encoding=?7, content_text=?8,
             fetch_status=?9, updated_at=?10
        WHERE id=?11",
        params![
            args.page_number,
            title,
            source_url,
            args.source_type,
            wayback.canonical_url,
            wayback.archived_at,
            requested_encoding,
            content_text,
            fetch_status,
            now,
            args.id,
        ],
    )?;
    if changed == 0 {
        return Err(CommandError::new("NOT_FOUND", "ページが見つかりません"));
    }

    page_get_by_id(&conn, args.id)
}

#[tauri::command]
pub fn page_delete(db: State<DbState>, args: IdArgs) -> CommandResult<()> {
    let conn = db.connection()?;
    conn.execute("DELETE FROM pages WHERE id=?1", params![args.id])?;
    Ok(())
}
