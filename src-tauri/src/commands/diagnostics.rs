use crate::{
    db::DbState,
    error::CommandResult,
    types::{DuplicateSourceUrlGroup, DuplicateSourceUrlPage},
};
use tauri::State;

#[tauri::command]
pub fn duplicate_source_url_list(
    db: State<DbState>,
) -> CommandResult<Vec<DuplicateSourceUrlGroup>> {
    let conn = db.connection()?;
    find_duplicate_source_urls(&conn)
}

pub(super) fn find_duplicate_source_urls(
    conn: &rusqlite::Connection,
) -> CommandResult<Vec<DuplicateSourceUrlGroup>> {
    let mut stmt = conn.prepare(
        "SELECT
           s.id,
           s.name,
           w.id,
           w.title,
           p.source_type,
           p.source_url,
           p.id,
           p.title,
           p.page_number
         FROM pages p
         JOIN works w ON p.work_id = w.id
         JOIN sites s ON w.site_id = s.id
         JOIN (
           SELECT work_id, source_type, source_url
           FROM pages
           WHERE source_url IS NOT NULL
           GROUP BY work_id, source_type, source_url
           HAVING COUNT(*) > 1
         ) d ON d.work_id = p.work_id
            AND d.source_type = p.source_type
            AND d.source_url = p.source_url
         ORDER BY s.name COLLATE NOCASE, w.sort_order, w.id, p.source_type, p.source_url, p.sort_order, p.id",
    )?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                DuplicateSourceUrlPage {
                    page_id: row.get(6)?,
                    page_title: row.get(7)?,
                    page_number: row.get(8)?,
                },
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut groups: Vec<DuplicateSourceUrlGroup> = Vec::new();
    for (site_id, site_name, work_id, work_title, source_type, source_url, page) in rows {
        let group = match groups.last_mut() {
            Some(group)
                if group.work_id == work_id
                    && group.source_type == source_type
                    && group.source_url == source_url =>
            {
                group
            }
            _ => {
                groups.push(DuplicateSourceUrlGroup {
                    site_id,
                    site_name,
                    work_id,
                    work_title,
                    source_type,
                    source_url,
                    pages: Vec::new(),
                });
                groups.last_mut().expect("group exists")
            }
        };
        group.pages.push(page);
    }

    Ok(groups)
}
