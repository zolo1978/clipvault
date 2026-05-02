use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Application configuration persisted via tauri-plugin-store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub max_clips: u32,
    pub keep_days: u32,
    pub monitor_interval_ms: u64,
    pub exclude_sources: Vec<String>,
    pub shortcut: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_clips: 10000,
            keep_days: 0,
            monitor_interval_ms: 250,
            exclude_sources: Vec::new(),
            shortcut: "Cmd+Shift+V".to_string(),
        }
    }
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub config: Arc<tokio::sync::RwLock<AppConfig>>,
    pub monitor: Arc<tokio::sync::Mutex<Option<crate::services::monitor_service::MonitorService>>>,
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
        }
    }

    pub fn default_db_path() -> PathBuf {
        let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("clipvault").join("clipvault.db")
    }

    fn run_migrations(conn: &Connection) -> Result<(), crate::error::AppError> {
        conn.execute_batch(include_str!("../migrations/001_init.sql"))?;
        Ok(())
    }
}
