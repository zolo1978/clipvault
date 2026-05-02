// error.rs — ClipVault unified error type

use serde::Serialize;

/// Application-wide error type.
/// All Tauri commands return `Result<T, AppError>`.
/// thiserror derives Display + Error; manual Serialize impl sanitizes for IPC.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("clipboard error: {0}")]
    #[allow(dead_code)]
    Clipboard(String),

    #[error("internal error: {0}")]
    Internal(String),
}

// Serialize as plain string for Tauri IPC — frontend sees only the message.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Helper: wrap any error as Internal
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
