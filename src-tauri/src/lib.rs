mod commands;
mod db;
mod error;
mod types;

use db::init_db;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let state = init_db(&app_data_dir)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::sites::site_create,
            commands::sites::site_list,
            commands::sites::site_update,
            commands::sites::site_delete,
            commands::works::work_create,
            commands::works::work_list_by_site,
            commands::works::work_update,
            commands::works::work_delete,
            commands::site_profiles::site_profile_create,
            commands::site_profiles::site_profile_list,
            commands::site_profiles::site_profile_update,
            commands::site_profiles::site_profile_delete,
            commands::pages::page_create,
            commands::pages::page_list_by_work,
            commands::pages::page_get,
            commands::pages::page_update,
            commands::pages::page_delete,
            commands::search::search_titles,
            commands::search::search_full_text,
            commands::search::rebuild_search_index,
            commands::fetch::fetch_page_by_url,
            commands::favorites::favorite_add,
            commands::favorites::favorite_remove,
            commands::favorites::favorite_check,
            commands::favorites::favorite_list,
            commands::fetch::bulk_fetch_by_profile,
            commands::files::export_page_text,
            commands::files::backup_database,
            commands::diagnostics::duplicate_source_url_list,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run NovelVault");
}
