// =============================================================================
// favorites.rs
// お気に入り機能の型定義とコマンド実装（Rust側）
// 対応要件定義: v4.3 / スキーマ: 002_add_favorites.sql
// =============================================================================

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::types::{CommandError, CommandResult};
use crate::db::DbState;

// 現在時刻を UTC ISO8601 文字列で返す（commands.rs と同じ実装）
fn now_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}


// =============================================================================
// エンティティ型
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
    pub id:         i64,
    pub page_id:    i64,
    pub created_at: String,
}

/// お気に入り一覧の1行（JOIN済み）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteListItem {
    pub favorite_id:  i64,
    pub favorited_at: String,
    pub page_id:      i64,
    pub page_title:   Option<String>,
    pub page_number:  Option<i64>,
    pub work_id:      i64,
    pub work_title:   String,
    pub site_id:      i64,
    pub site_name:    String,
}

/// サイト・作品でグループ化された一覧
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoritesGrouped {
    pub groups: Vec<FavoriteSiteGroup>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteSiteGroup {
    pub site_id:   i64,
    pub site_name: String,
    pub works:     Vec<FavoriteWorkGroup>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteWorkGroup {
    pub work_id:    i64,
    pub work_title: String,
    pub pages:      Vec<FavoriteListItem>,
}


// =============================================================================
// コマンド引数型
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteAddArgs {
    pub page_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteRemoveArgs {
    pub page_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteCheckArgs {
    pub page_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteCheckResult {
    pub is_favorite: bool,
    pub favorite_id: Option<i64>,
}


// =============================================================================
// コマンド実装
// =============================================================================

/// お気に入りに追加
///
/// 既に登録済みのページは重複登録せず、既存レコードを返す。
#[tauri::command]
pub fn favorite_add(
    db: State<DbState>,
    args: FavoriteAddArgs,
) -> CommandResult<Favorite> {
    let conn = db.connection()?;
    let now = now_utc();
    conn.execute(
        "INSERT INTO favorites (page_id, created_at) VALUES (?1, ?2)
         ON CONFLICT(page_id) DO NOTHING",
        rusqlite::params![args.page_id, now],
    )?;
    conn.query_row(
        "SELECT id, page_id, created_at FROM favorites WHERE page_id=?1",
        rusqlite::params![args.page_id],
        |row| Ok(Favorite {
            id: row.get(0)?,
            page_id: row.get(1)?,
            created_at: row.get(2)?,
        }),
    ).map_err(CommandError::from)
}

/// お気に入りから削除
#[tauri::command]
pub fn favorite_remove(
    db: State<DbState>,
    args: FavoriteRemoveArgs,
) -> CommandResult<()> {
    let conn = db.connection()?;
    conn.execute(
        "DELETE FROM favorites WHERE page_id=?1",
        rusqlite::params![args.page_id],
    )?;
    Ok(())
}

/// 特定ページがお気に入り済みか確認
#[tauri::command]
pub fn favorite_check(
    db: State<DbState>,
    args: FavoriteCheckArgs,
) -> CommandResult<FavoriteCheckResult> {
    let conn = db.connection()?;
    let result = conn.query_row(
        "SELECT id FROM favorites WHERE page_id=?1",
        rusqlite::params![args.page_id],
        |row| row.get::<_, i64>(0),
    );
    match result {
        Ok(id) => Ok(FavoriteCheckResult {
            is_favorite: true,
            favorite_id: Some(id),
        }),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(FavoriteCheckResult {
            is_favorite: false,
            favorite_id: None,
        }),
        Err(e) => Err(CommandError::from(e)),
    }
}

/// お気に入り一覧をサイト・作品でグループ化して返す
///
/// SQL では平坦なJOIN結果を取得し、Rust側でグループ化する。
/// 並び順は sites.name → works.sort_order → pages.sort_order。
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
         JOIN pages p ON f.page_id  = p.id
         JOIN works w ON p.work_id  = w.id
         JOIN sites s ON w.site_id  = s.id
         ORDER BY s.name COLLATE NOCASE, w.sort_order, p.sort_order, p.id"
    )?;

    let items = stmt.query_map([], |row| {
        Ok(FavoriteListItem {
            favorite_id:  row.get(0)?,
            favorited_at: row.get(1)?,
            page_id:      row.get(2)?,
            page_title:   row.get(3)?,
            page_number:  row.get(4)?,
            work_id:      row.get(5)?,
            work_title:   row.get(6)?,
            site_id:      row.get(7)?,
            site_name:    row.get(8)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    // サイト・作品でグループ化
    Ok(group_favorites(items))
}


// =============================================================================
// 内部ユーティリティ
// =============================================================================

/// 平坦なリストをサイト・作品でグループ化
/// 入力は site → work の順にソート済みである前提
fn group_favorites(items: Vec<FavoriteListItem>) -> FavoritesGrouped {
    let mut groups: Vec<FavoriteSiteGroup> = Vec::new();

    for item in items {
        // サイトグループ取得 or 新規作成
        let site_group = match groups.last_mut() {
            Some(g) if g.site_id == item.site_id => g,
            _ => {
                groups.push(FavoriteSiteGroup {
                    site_id:   item.site_id,
                    site_name: item.site_name.clone(),
                    works:     Vec::new(),
                });
                groups.last_mut().unwrap()
            }
        };

        // 作品グループ取得 or 新規作成
        let work_group = match site_group.works.last_mut() {
            Some(w) if w.work_id == item.work_id => w,
            _ => {
                site_group.works.push(FavoriteWorkGroup {
                    work_id:    item.work_id,
                    work_title: item.work_title.clone(),
                    pages:      Vec::new(),
                });
                site_group.works.last_mut().unwrap()
            }
        };

        work_group.pages.push(item);
    }

    FavoritesGrouped { groups }
}


// =============================================================================
// main.rs での登録例
// =============================================================================
//
// tauri::Builder::default()
//   .invoke_handler(tauri::generate_handler![
//     // ... 既存のコマンド ...
//     favorite_add,
//     favorite_remove,
//     favorite_check,
//     favorite_list,
//   ])
