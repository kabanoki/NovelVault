use super::{ensure_exists, now_utc};
use crate::{
    db::DbState,
    error::CommandResult,
    types::{
        Favorite, FavoriteCheckResult, FavoriteListItem, FavoritePageArgs, FavoriteSiteGroup,
        FavoriteWorkGroup, FavoritesGrouped,
    },
};
use rusqlite::{params, OptionalExtension};
use tauri::State;

#[tauri::command]
pub fn favorite_add(db: State<DbState>, args: FavoritePageArgs) -> CommandResult<Favorite> {
    let conn = db.connection()?;
    ensure_exists(&conn, "pages", args.page_id, "ページ")?;
    let now = now_utc();
    conn.execute(
        "INSERT INTO favorites (page_id, created_at) VALUES (?1, ?2)
         ON CONFLICT(page_id) DO NOTHING",
        params![args.page_id, now],
    )?;
    let favorite = conn.query_row(
        "SELECT id, page_id, created_at FROM favorites WHERE page_id=?1",
        params![args.page_id],
        |row| {
            Ok(Favorite {
                id: row.get(0)?,
                page_id: row.get(1)?,
                created_at: row.get(2)?,
            })
        },
    )?;
    Ok(favorite)
}

#[tauri::command]
pub fn favorite_remove(db: State<DbState>, args: FavoritePageArgs) -> CommandResult<()> {
    let conn = db.connection()?;
    conn.execute(
        "DELETE FROM favorites WHERE page_id=?1",
        params![args.page_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn favorite_check(
    db: State<DbState>,
    args: FavoritePageArgs,
) -> CommandResult<FavoriteCheckResult> {
    let conn = db.connection()?;
    let favorite_id = conn
        .query_row(
            "SELECT id FROM favorites WHERE page_id=?1",
            params![args.page_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    Ok(FavoriteCheckResult {
        is_favorite: favorite_id.is_some(),
        favorite_id,
    })
}

#[tauri::command]
pub fn favorite_list(db: State<DbState>) -> CommandResult<FavoritesGrouped> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT
           f.id, f.created_at,
           p.id, p.title, p.page_number,
           w.id, w.title,
           s.id, s.name
         FROM favorites f
         JOIN pages p ON f.page_id = p.id
         JOIN works w ON p.work_id = w.id
         JOIN sites s ON w.site_id = s.id
         ORDER BY s.name COLLATE NOCASE, s.id, w.sort_order, w.id, p.sort_order, p.id",
    )?;
    let items = stmt
        .query_map([], |row| {
            Ok(FavoriteListItem {
                favorite_id: row.get(0)?,
                favorited_at: row.get(1)?,
                page_id: row.get(2)?,
                page_title: row.get(3)?,
                page_number: row.get(4)?,
                work_id: row.get(5)?,
                work_title: row.get(6)?,
                site_id: row.get(7)?,
                site_name: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(group_favorites(items))
}

pub(super) fn group_favorites(items: Vec<FavoriteListItem>) -> FavoritesGrouped {
    let mut groups: Vec<FavoriteSiteGroup> = Vec::new();

    for item in items {
        let site_group = match groups.last_mut() {
            Some(group) if group.site_id == item.site_id => group,
            _ => {
                groups.push(FavoriteSiteGroup {
                    site_id: item.site_id,
                    site_name: item.site_name.clone(),
                    works: Vec::new(),
                });
                groups.last_mut().expect("group exists")
            }
        };

        let work_group = match site_group.works.last_mut() {
            Some(group) if group.work_id == item.work_id => group,
            _ => {
                site_group.works.push(FavoriteWorkGroup {
                    work_id: item.work_id,
                    work_title: item.work_title.clone(),
                    pages: Vec::new(),
                });
                site_group.works.last_mut().expect("work group exists")
            }
        };

        work_group.pages.push(item);
    }

    FavoritesGrouped { groups }
}
