use super::{
    ensure_exists, map_site_profile, now_utc, site_profile_get_by_id, validate_profile_json,
};
use crate::{
    db::DbState,
    error::{CommandError, CommandResult},
    types::{
        IdArgs, SiteProfile, SiteProfileCreateArgs, SiteProfileListArgs, SiteProfileUpdateArgs,
    },
};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn site_profile_create(
    db: State<DbState>,
    args: SiteProfileCreateArgs,
) -> CommandResult<SiteProfile> {
    let name = args.name.trim();
    if name.is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "プロファイル名は必須です",
        ));
    }
    validate_profile_json(&args.profile_json)?;

    let conn = db.connection()?;
    ensure_exists(&conn, "sites", args.site_id, "サイト")?;
    let now = now_utc();
    conn.execute(
        "INSERT INTO site_profiles (site_id, name, profile_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![args.site_id, name, args.profile_json, now, now],
    )?;
    site_profile_get_by_id(&conn, conn.last_insert_rowid())
}

#[tauri::command]
pub fn site_profile_list(
    db: State<DbState>,
    args: SiteProfileListArgs,
) -> CommandResult<Vec<SiteProfile>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, site_id, name, profile_json, created_at, updated_at
         FROM site_profiles
         WHERE site_id=?1
         ORDER BY name COLLATE NOCASE, id",
    )?;
    let profiles = stmt
        .query_map(params![args.site_id], map_site_profile)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(profiles)
}

#[tauri::command]
pub fn site_profile_update(
    db: State<DbState>,
    args: SiteProfileUpdateArgs,
) -> CommandResult<SiteProfile> {
    let name = args.name.trim();
    if name.is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "プロファイル名は必須です",
        ));
    }
    validate_profile_json(&args.profile_json)?;

    let conn = db.connection()?;
    let now = now_utc();
    let changed = conn.execute(
        "UPDATE site_profiles SET name=?1, profile_json=?2, updated_at=?3 WHERE id=?4",
        params![name, args.profile_json, now, args.id],
    )?;
    if changed == 0 {
        return Err(CommandError::new(
            "NOT_FOUND",
            "プロファイルが見つかりません",
        ));
    }
    site_profile_get_by_id(&conn, args.id)
}

#[tauri::command]
pub fn site_profile_delete(db: State<DbState>, args: IdArgs) -> CommandResult<()> {
    let conn = db.connection()?;
    conn.execute("DELETE FROM site_profiles WHERE id=?1", params![args.id])?;
    Ok(())
}
