mod connection;
mod migrations;

use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub struct DbState {
    db_path: PathBuf,
}

impl DbState {
    pub fn connection(&self) -> crate::error::CommandResult<Connection> {
        Ok(connection::open(&self.db_path)?)
    }
}

pub fn init_db(app_data_dir: &Path) -> crate::error::CommandResult<DbState> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("app.db");
    let mut conn = connection::open(&db_path)?;
    migrations::run(&mut conn)?;
    Ok(DbState { db_path })
}
