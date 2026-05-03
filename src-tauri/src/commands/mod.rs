// commands/mod.rs — Tauri IPC command handlers (core CRUD)

pub mod clipboard;
pub mod monitor;
pub mod screenshot;
pub mod window;

use crate::error::AppError;
use crate::models::*;
use crate::services::clip_service;
use crate::state::{AppConfig, AppState};
use tauri::State;

/// List clips with pagination, newest first. Optional content_type filter.
#[tauri::command]
pub async fn list_clips(
    limit: u32,
    offset: u32,
    content_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ClipSummary>, AppError> {
    let db = state.db.clone();
    let ct = content_type.map(|s| s.to_string());
    tokio::task::spawn_blocking(move || {
        let ct_ref = ct.as_deref();
        clip_service::list_recent(&db, limit, offset, ct_ref)
    })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Search clips via FTS5 full-text search.
#[tauri::command]
pub async fn search_clips(
    query: String,
    content_type: Option<String>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<ClipSummary>, AppError> {
    let db = state.db.clone();
    let ct = content_type.map(|s| s.to_string());
    let limit = limit.unwrap_or(50);
    tokio::task::spawn_blocking(move || {
        let ct_ref = ct.as_deref();
        clip_service::search(&db, &query, ct_ref, limit)
    })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Get a single clip by ID (full content).
#[tauri::command]
pub async fn get_clip(
    id: String,
    state: State<'_, AppState>,
) -> Result<Clip, AppError> {
    let db = state.db.clone();
    let id_for_db = id.clone();
    let mut clip = tokio::task::spawn_blocking(move || {
        clip_service::get_clip(&db, &id_for_db)?
            .ok_or_else(|| AppError::NotFound(format!("clip not found: {id_for_db}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // Populate sensitive content from memory store
    if clip.is_sensitive {
        let mut store = state.sensitive_store.lock().await;
        if let Some(
            crate::services::sensitive_store::EntryState::Available(content),
        ) = store.get(&id)
        {
            clip.content = content;
        }
    }

    Ok(clip)
}

/// Delete a single clip.
#[tauri::command]
pub async fn delete_clip(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || clip_service::delete_clip(&db, &id))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Batch delete clips by IDs.
#[tauri::command]
pub async fn delete_clips(
    ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || clip_service::delete_clips(&db, &ids))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Toggle favorite status. Returns updated summary.
#[tauri::command]
pub async fn toggle_favorite(
    id: String,
    state: State<'_, AppState>,
) -> Result<ClipSummary, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || clip_service::toggle_favorite(&db, &id))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Purge old clips based on retention policy.
#[tauri::command]
pub async fn purge_clips(
    keep_days: Option<u32>,
    keep_count: Option<u32>,
    state: State<'_, AppState>,
) -> Result<u64, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || clip_service::purge_clips(&db, keep_days, keep_count))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Get current application config.
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, AppError> {
    let config = state.config.read().await;
    Ok(config.clone())
}

/// Update application config with validation.
#[tauri::command]
pub async fn update_config(
    config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    if config.max_clips < 1 {
        return Err(AppError::Validation("max_clips must be at least 1".into()));
    }
    if config.monitor_interval_ms < 50 {
        return Err(AppError::Validation(
            "monitor_interval_ms must be at least 50".into(),
        ));
    }
    let mut current = state.config.write().await;
    *current = config;
    Ok(())
}
