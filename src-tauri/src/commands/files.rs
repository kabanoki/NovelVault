use super::now_utc;
use crate::{
    db::DbState,
    error::CommandResult,
    types::{FileOutputResult, IdArgs},
};
use rusqlite::params;
use std::fs;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn export_page_text(
    app: AppHandle,
    db: State<DbState>,
    args: IdArgs,
) -> CommandResult<FileOutputResult> {
    let (site_name, work_title, page_title, content_text): (
        String,
        String,
        Option<String>,
        String,
    ) = {
        let conn = db.connection()?;
        conn.query_row(
            "SELECT s.name, w.title, p.title, COALESCE(p.content_text, '')
             FROM pages p
             JOIN works w ON w.id = p.work_id
             JOIN sites s ON s.id = w.site_id
             WHERE p.id = ?1",
            params![args.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?
    };

    let export_dir = app.path().app_data_dir()?.join("exports");
    fs::create_dir_all(&export_dir)?;
    let title = page_title.unwrap_or_else(|| "untitled".to_string());
    let filename = format!(
        "{}-{}-{}-page-{}.txt",
        sanitize_filename(&site_name),
        sanitize_filename(&work_title),
        sanitize_filename(&title),
        args.id
    );
    let path = export_dir.join(filename);
    fs::write(&path, content_text)?;

    Ok(FileOutputResult {
        path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn backup_database(app: AppHandle, db: State<DbState>) -> CommandResult<FileOutputResult> {
    let backup_dir = app.path().app_data_dir()?.join("backups");
    fs::create_dir_all(&backup_dir)?;
    let timestamp = now_utc()
        .replace([':', '-'], "")
        .replace('T', "_")
        .trim_end_matches('Z')
        .to_string();
    let path = backup_dir.join(format!("novel-vault-{timestamp}.db"));

    {
        let conn = db.connection()?;
        conn.execute("VACUUM INTO ?1", params![path.to_string_lossy().as_ref()])?;
    }

    Ok(FileOutputResult {
        path: path.to_string_lossy().to_string(),
    })
}

pub(super) fn sanitize_filename(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if cleaned.is_empty() {
        "untitled".to_string()
    } else {
        cleaned.chars().take(80).collect()
    }
}
