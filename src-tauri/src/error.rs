use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

impl From<rusqlite::Error> for CommandError {
    fn from(value: rusqlite::Error) -> Self {
        Self::new("DB_ERROR", value.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(value: std::io::Error) -> Self {
        Self::new("IO_ERROR", value.to_string())
    }
}

impl From<tauri::Error> for CommandError {
    fn from(value: tauri::Error) -> Self {
        Self::new("TAURI_ERROR", value.to_string())
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
