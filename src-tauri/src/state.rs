use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{
    atomic::AtomicBool,
    Arc, Mutex,
};

/// Application configuration persisted via tauri-plugin-store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub max_clips: u32,
    pub keep_days: u32,
    pub monitor_interval_ms: u64,
    pub exclude_sources: Vec<String>,
    pub shortcut: String,
    pub sensitive_detection_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_clips: 10000,
            keep_days: 0,
            monitor_interval_ms: 250,
            exclude_sources: Vec::new(),
            shortcut: "Cmd+Shift+V".to_string(),
            sensitive_detection_enabled: true,
        }
    }
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub config: Arc<tokio::sync::RwLock<AppConfig>>,
    pub monitor: Arc<tokio::sync::Mutex<Option<crate::services::monitor_service::MonitorService>>>,
    pub is_pasting: Arc<AtomicBool>,
    pub sensitive_store: Arc<tokio::sync::Mutex<crate::services::sensitive_store::SensitiveStore>>,
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self, crate::error::AppError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = ON;",
        )?;
        Self::run_migrations(&conn)?;
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            config: Arc::new(tokio::sync::RwLock::new(AppConfig::default())),
            monitor: Arc::new(tokio::sync::Mutex::new(None)),
            is_pasting: Arc::new(AtomicBool::new(false)),
            sensitive_store: Arc::new(tokio::sync::Mutex::new(
                crate::services::sensitive_store::SensitiveStore::new(300_000),
            )),
        })
    }

    #[allow(dead_code)]
    pub fn new_test() -> Self {
        let conn = Connection::open_in_memory().expect("in-memory db must not fail");
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
            .expect("pragma must not fail");
        Self::run_migrations(&conn).expect("migrations must not fail");
        Self {
            db: Arc::new(Mutex::new(conn)),
            config: Arc::new(tokio::sync::RwLock::new(AppConfig::default())),
            monitor: Arc::new(tokio::sync::Mutex::new(None)),
            is_pasting: Arc::new(AtomicBool::new(false)),
            sensitive_store: Arc::new(tokio::sync::Mutex::new(
                crate::services::sensitive_store::SensitiveStore::new(300_000),
            )),
        }
    }

    pub fn default_db_path() -> PathBuf {
        let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("clipvault").join("clipvault.db")
    }

    fn run_migrations(conn: &Connection) -> Result<(), crate::error::AppError> {
        conn.execute_batch(include_str!("../migrations/001_init.sql"))?;
        // 002 may fail with "duplicate column" if already applied
        match conn.execute_batch(include_str!("../migrations/002_sensitive.sql")) {
            Ok(()) => {}
            Err(e) if e.to_string().contains("duplicate column") => {}
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContentType;
    use crate::services::clip_service;

    #[test]
    fn test_clip_crud_with_is_sensitive() {
        let state = AppState::new_test();

        // Insert two clips
        let clip1 = clip_service::create_clip(&state.db, ContentType::Text, b"text_1".to_vec()).unwrap();
        let clip2 = clip_service::create_clip(&state.db, ContentType::Text, b"text_3".to_vec()).unwrap();

        assert_ne!(clip1.id, clip2.id);
        assert_eq!(clip1.content, b"text_1");
        assert_eq!(clip2.content, b"text_3");
        assert!(!clip1.is_sensitive);
        assert!(!clip2.is_sensitive);

        // Read back clip2 (the "3" one)
        let read2 = clip_service::get_clip(&state.db, &clip2.id).unwrap().unwrap();
        assert_eq!(read2.content, b"text_3");
        assert_eq!(read2.id, clip2.id);
        assert!(!read2.is_sensitive);

        // Read back clip1 (the "1" one)  
        let read1 = clip_service::get_clip(&state.db, &clip1.id).unwrap().unwrap();
        assert_eq!(read1.content, b"text_1");
        assert_eq!(read1.id, clip1.id);

        // List clips - should have both
        let summaries = clip_service::list_recent(&state.db, 10, 0, None).unwrap();
        assert_eq!(summaries.len(), 2);
        let previews: Vec<&str> = summaries.iter().map(|s| s.preview.as_str()).collect();
        assert!(previews.contains(&"text_1"), "missing text_1: {previews:?}");
        assert!(previews.contains(&"text_3"), "missing text_3: {previews:?}");

        eprintln!("[TEST] ALL PASSED - DB read/write correct with is_sensitive column");
    }
}
