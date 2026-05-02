// commands/mod.rs — ClipVault IPC command registration
// Thin adapters: validate params, delegate to service, return DTO.

pub mod clipboard;
pub mod monitor;
pub mod screenshot;
pub mod window;

use crate::error::AppError;
use crate::models::*;
use crate::services::clip_service;
use crate::state::AppState;
use tauri::State;

/// List clips with pagination and optional type filter.
#[tauri::command]
pub async fn list_clips(
    limit: u32,
    offset: u32,
    content_type: Option<ContentType>,
    state: State<'_, AppState>,
) -> Result<Vec<ClipSummary>, AppError> {
    let db = state.db.clone();
    let ct_str = content_type.as_ref().map(|ct| ct.as_str().to_string());
    tokio::task::spawn_blocking(move || {
        clip_service::list_recent(
            &db,
            limit,
            offset,
            ct_str.as_deref(),
        )
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Search clips via FTS5.
#[tauri::command]
pub async fn search_clips(
    query: String,
    content_type: Option<ContentType>,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<ClipSummary>, AppError> {
    let db = state.db.clone();
    let ct_str = content_type.as_ref().map(|ct| ct.as_str().to_string());
    tokio::task::spawn_blocking(move || {
        clip_service::search(&db, &query, ct_str.as_deref(), limit)
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
    tokio::task::spawn_blocking(move || {
        clip_service::get_clip(&db, &id)?.ok_or_else(|| {
            AppError::NotFound(format!("clip not found: {id}"))
        })
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
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

/// Batch delete clips.
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

/// Toggle favorite status.
#[tauri::command]
pub async fn toggle_favorite(
    id: String,
    state: State<'_, AppState>,
) -> Result<ClipSummary, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        clip_service::toggle_favorite(&db, &id)
    })
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
    tokio::task::spawn_blocking(move || {
        clip_service::purge_clips(&db, keep_days, keep_count)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Get current application config.
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<crate::state::AppConfig, AppError> {
    let config = state.config.read().await;
    Ok(config.clone())
}

/// Update application config with field-level validation.
#[tauri::command]
pub async fn update_config(
    config: crate::state::AppConfig,
    state: State<'_, AppState>,
) -> Result<crate::state::AppConfig, AppError> {
    if config.max_clips == 0 {
        return Err(AppError::Validation("max_clips must be >= 1".into()));
    }
    if config.monitor_interval_ms < 50 {
        return Err(AppError::Validation("monitor_interval_ms must be >= 50".into()));
    }
    let mut current = state.config.write().await;
    *current = config.clone();
    Ok(config)
}
