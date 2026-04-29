use crate::{
    error::{CommandError, CommandResult},
    types::{Page, SiteProfile, Work},
};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};

pub mod diagnostics;
pub mod favorites;
pub mod fetch;
pub mod files;
pub mod pages;
pub mod search;
pub mod site_profiles;
pub mod sites;
pub mod works;

pub(super) fn now_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub(super) fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(super) fn ensure_unique_page_source_url(
    conn: &rusqlite::Connection,
    work_id: i64,
    excluded_page_id: Option<i64>,
    source_url: Option<&str>,
    source_type: &str,
) -> CommandResult<()> {
    let Some(source_url) = source_url else {
        return Ok(());
    };
    let duplicate_id = conn
        .query_row(
            "SELECT id
             FROM pages
             WHERE work_id=?1
               AND source_type=?2
               AND source_url=?3
               AND (?4 IS NULL OR id<>?4)
             LIMIT 1",
            params![work_id, source_type, source_url, excluded_page_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    if duplicate_id.is_some() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "同じ作品に同じ取得元URLのページが既にあります",
        ));
    }
    Ok(())
}

pub(super) fn ensure_exists(
    conn: &rusqlite::Connection,
    table: &str,
    id: i64,
    label: &str,
) -> CommandResult<()> {
    let sql = format!("SELECT 1 FROM {table} WHERE id=?1");
    let exists = conn
        .query_row(&sql, params![id], |_| Ok(()))
        .optional()?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err(CommandError::new(
            "NOT_FOUND",
            format!("{label}が見つかりません"),
        ))
    }
}

pub(super) fn next_sort_order(
    conn: &rusqlite::Connection,
    table: &str,
    parent_column: &str,
    parent_id: i64,
) -> rusqlite::Result<i64> {
    let sql =
        format!("SELECT COALESCE(MAX(sort_order), 0) + 10 FROM {table} WHERE {parent_column}=?1");
    conn.query_row(&sql, params![parent_id], |row| row.get(0))
}

pub(super) fn validate_source_type(source_type: &str) -> CommandResult<()> {
    match source_type {
        "normal" | "wayback" | "local" => Ok(()),
        _ => Err(CommandError::new(
            "VALIDATION_ERROR",
            "URL種別は normal / wayback / local のいずれかです",
        )),
    }
}

#[derive(Debug)]
pub(super) struct WaybackMetadata {
    pub(super) canonical_url: Option<String>,
    pub(super) archived_at: Option<String>,
}

pub(super) fn wayback_metadata(
    url: Option<&str>,
    source_type: &str,
) -> CommandResult<WaybackMetadata> {
    if source_type != "wayback" {
        return Ok(WaybackMetadata {
            canonical_url: None,
            archived_at: None,
        });
    }
    let url = url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| CommandError::new("INVALID_WAYBACK_URL", "Wayback URLが空です"))?;
    parse_wayback_url(url)
}

fn parse_wayback_url(url: &str) -> CommandResult<WaybackMetadata> {
    let re = regex::Regex::new(r"web\.archive\.org/web/(\d{14})(?:[a-z_]+)?/(.+)$")
        .map_err(|e| CommandError::new("REGEX_ERROR", e.to_string()))?;
    let captures = re.captures(url).ok_or_else(|| {
        CommandError::new(
            "INVALID_WAYBACK_URL",
            "Wayback URLは /web/{timestamp}/{original_url} 形式である必要があります",
        )
    })?;
    Ok(WaybackMetadata {
        archived_at: captures.get(1).map(|value| value.as_str().to_string()),
        canonical_url: captures.get(2).map(|value| value.as_str().to_string()),
    })
}

pub(super) fn validate_profile_json(profile_json: &str) -> CommandResult<()> {
    let value: serde_json::Value = serde_json::from_str(profile_json)
        .map_err(|e| CommandError::new("VALIDATION_ERROR", format!("JSONが不正です: {e}")))?;
    if !value.is_object() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "プロファイルJSONはオブジェクトである必要があります",
        ));
    }
    Ok(())
}

pub(super) fn work_get_by_id(conn: &rusqlite::Connection, id: i64) -> CommandResult<Work> {
    conn.query_row(
        "SELECT id, site_id, site_profile_id, title, author_name, description, source_url,
                sort_order, created_at, updated_at
         FROM works
         WHERE id=?1",
        params![id],
        map_work,
    )
    .optional()?
    .ok_or_else(|| CommandError::new("NOT_FOUND", "作品が見つかりません"))
}

pub(super) fn site_profile_get_by_id(
    conn: &rusqlite::Connection,
    id: i64,
) -> CommandResult<SiteProfile> {
    conn.query_row(
        "SELECT id, site_id, name, profile_json, created_at, updated_at
         FROM site_profiles
         WHERE id=?1",
        params![id],
        map_site_profile,
    )
    .optional()?
    .ok_or_else(|| CommandError::new("NOT_FOUND", "プロファイルが見つかりません"))
}

pub(super) fn page_get_by_id(conn: &rusqlite::Connection, id: i64) -> CommandResult<Page> {
    conn.query_row(
        "SELECT id, work_id, page_number, sort_order, title, source_url, source_type,
                canonical_url, archived_at, requested_encoding, detected_encoding, content_text, content_html_path,
                fetch_status, fetch_error, fetched_at, created_at, updated_at
         FROM pages
         WHERE id=?1",
        params![id],
        map_page,
    )
    .optional()?
    .ok_or_else(|| CommandError::new("NOT_FOUND", "ページが見つかりません"))
}

pub(super) fn map_work(row: &rusqlite::Row<'_>) -> rusqlite::Result<Work> {
    Ok(Work {
        id: row.get(0)?,
        site_id: row.get(1)?,
        site_profile_id: row.get(2)?,
        title: row.get(3)?,
        author_name: row.get(4)?,
        description: row.get(5)?,
        source_url: row.get(6)?,
        sort_order: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub(super) fn map_site_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<SiteProfile> {
    Ok(SiteProfile {
        id: row.get(0)?,
        site_id: row.get(1)?,
        name: row.get(2)?,
        profile_json: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub(super) fn map_page(row: &rusqlite::Row<'_>) -> rusqlite::Result<Page> {
    Ok(Page {
        id: row.get(0)?,
        work_id: row.get(1)?,
        page_number: row.get(2)?,
        sort_order: row.get(3)?,
        title: row.get(4)?,
        source_url: row.get(5)?,
        source_type: row.get(6)?,
        canonical_url: row.get(7)?,
        archived_at: row.get(8)?,
        requested_encoding: row.get(9)?,
        detected_encoding: row.get(10)?,
        content_text: row.get(11)?,
        content_html_path: row.get(12)?,
        fetch_status: row.get(13)?,
        fetch_error: row.get(14)?,
        fetched_at: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

#[cfg(test)]
mod tests;
