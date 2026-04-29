use crate::{
    db::DbState,
    error::CommandResult,
    types::{
        FullTextSearchItem, PageSearchItem, SearchFullTextArgs, SearchTitlesArgs, Site,
        TitleSearchResults, WorkSearchItem,
    },
};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn search_titles(
    db: State<DbState>,
    args: SearchTitlesArgs,
) -> CommandResult<TitleSearchResults> {
    let query = args.query.trim();
    if query.is_empty() {
        return Ok(TitleSearchResults {
            sites: Vec::new(),
            works: Vec::new(),
            pages: Vec::new(),
        });
    }
    let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
    let conn = db.connection()?;

    let mut site_stmt = conn.prepare(
        "SELECT id, name, base_url, created_at, updated_at
         FROM sites
         WHERE name LIKE ?1 ESCAPE '\\'
         ORDER BY name COLLATE NOCASE, id
         LIMIT 50",
    )?;
    let sites = site_stmt
        .query_map(params![pattern], |row| {
            Ok(Site {
                id: row.get(0)?,
                name: row.get(1)?,
                base_url: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut work_stmt = conn.prepare(
        "SELECT w.id, w.site_id, s.name, w.title, w.author_name
         FROM works w
         JOIN sites s ON w.site_id = s.id
         WHERE w.title LIKE ?1 ESCAPE '\\'
         ORDER BY s.name COLLATE NOCASE, w.sort_order, w.id
         LIMIT 50",
    )?;
    let works = work_stmt
        .query_map(params![pattern], |row| {
            Ok(WorkSearchItem {
                id: row.get(0)?,
                site_id: row.get(1)?,
                site_name: row.get(2)?,
                title: row.get(3)?,
                author_name: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut page_stmt = conn.prepare(
        "SELECT p.id, p.work_id, w.title, s.id, s.name, p.title, p.page_number
         FROM pages p
         JOIN works w ON p.work_id = w.id
         JOIN sites s ON w.site_id = s.id
         WHERE p.title LIKE ?1 ESCAPE '\\'
         ORDER BY s.name COLLATE NOCASE, w.sort_order, p.sort_order, p.id
         LIMIT 50",
    )?;
    let pages = page_stmt
        .query_map(params![pattern], |row| {
            Ok(PageSearchItem {
                id: row.get(0)?,
                work_id: row.get(1)?,
                work_title: row.get(2)?,
                site_id: row.get(3)?,
                site_name: row.get(4)?,
                title: row.get(5)?,
                page_number: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(TitleSearchResults {
        sites,
        works,
        pages,
    })
}

#[tauri::command]
pub fn search_full_text(
    db: State<DbState>,
    args: SearchFullTextArgs,
) -> CommandResult<Vec<FullTextSearchItem>> {
    let query = args.query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let conn = db.connection()?;
    if search_terms(query)
        .iter()
        .any(|term| term.chars().count() < 3)
    {
        search_full_text_like(&conn, query)
    } else {
        search_full_text_fts(&conn, query)
    }
}

#[tauri::command]
pub fn rebuild_search_index(db: State<DbState>) -> CommandResult<()> {
    let conn = db.connection()?;
    conn.execute("INSERT INTO pages_fts(pages_fts) VALUES ('rebuild')", [])?;
    Ok(())
}

pub(super) fn build_fts_query(query: &str) -> String {
    search_terms(query)
        .into_iter()
        .filter_map(|term| {
            let escaped = term.replace('"', "\"\"");
            if escaped.is_empty() {
                None
            } else {
                Some(format!("\"{escaped}\""))
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn search_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(str::to_string)
        .collect()
}

pub(super) fn search_full_text_fts(
    conn: &rusqlite::Connection,
    query: &str,
) -> CommandResult<Vec<FullTextSearchItem>> {
    let fts_query = build_fts_query(query);
    let mut stmt = conn.prepare(
        "SELECT
           p.id,
           p.title,
           p.page_number,
           w.id,
           w.title,
           s.id,
           s.name,
           snippet(pages_fts, 1, '<mark>', '</mark>', '...', 24)
         FROM pages_fts
         JOIN pages p ON pages_fts.rowid = p.id
         JOIN works w ON p.work_id = w.id
         JOIN sites s ON w.site_id = s.id
         WHERE pages_fts MATCH ?1
         ORDER BY rank
         LIMIT 50",
    )?;

    let results = map_full_text_rows(stmt.query_map(params![fts_query], map_full_text_item)?);
    results
}

pub(super) fn search_full_text_like(
    conn: &rusqlite::Connection,
    query: &str,
) -> CommandResult<Vec<FullTextSearchItem>> {
    let terms = search_terms(query);
    let where_clause = terms
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                "(COALESCE(p.title, '') LIKE ?{0} ESCAPE '\\' OR COALESCE(p.content_text, '') LIKE ?{0} ESCAPE '\\')",
                index + 1
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    let sql = format!(
        "SELECT
           p.id,
           p.title,
           p.page_number,
           w.id,
           w.title,
           s.id,
           s.name,
           substr(COALESCE(p.content_text, ''), 1, 180)
         FROM pages p
         JOIN works w ON p.work_id = w.id
         JOIN sites s ON w.site_id = s.id
         WHERE {where_clause}
         ORDER BY s.name COLLATE NOCASE, w.sort_order, p.sort_order
         LIMIT 50"
    );
    let patterns = terms
        .iter()
        .map(|term| like_pattern(term))
        .collect::<Vec<_>>();
    let params = rusqlite::params_from_iter(patterns.iter().map(String::as_str));
    let mut stmt = conn.prepare(&sql)?;

    let results = map_full_text_rows(stmt.query_map(params, map_full_text_item)?);
    results
}

fn map_full_text_rows<T>(
    rows: rusqlite::MappedRows<'_, T>,
) -> CommandResult<Vec<FullTextSearchItem>>
where
    T: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<FullTextSearchItem>,
{
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn map_full_text_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<FullTextSearchItem> {
    Ok(FullTextSearchItem {
        page_id: row.get(0)?,
        page_title: row.get(1)?,
        page_number: row.get(2)?,
        work_id: row.get(3)?,
        work_title: row.get(4)?,
        site_id: row.get(5)?,
        site_name: row.get(6)?,
        snippet: row.get(7)?,
    })
}

fn like_pattern(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}
