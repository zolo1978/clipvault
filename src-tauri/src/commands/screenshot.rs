use crate::error::AppError;
use crate::models::{Clip, ClipSummary, ContentType};
use crate::services::clip_service;
use crate::state::AppState;
use tauri::{Emitter, State};

#[tauri::command]
pub async fn snip_screen(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ClipSummary, AppError> {
    window
        .hide()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let tmp = std::env::temp_dir().join(format!(
        "clipvault-snip-{}.png",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let tmp_path = tmp
        .to_str()
        .ok_or_else(|| AppError::Internal("invalid temp path".into()))?;

    let output = tokio::process::Command::new("screencapture")
        .arg("-i")
        .arg(tmp_path)
        .output()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !output.status.success() || !tmp.exists() {
        let _ = window.show();
        let _ = window.set_focus();
        return Err(AppError::Validation("截图已取消".into()));
    }

    let image_data =
        tokio::fs::read(&tmp)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = std::fs::remove_file(&tmp);

    let db = state.db.clone();
    let clip = tokio::task::spawn_blocking(move || -> Result<Clip, AppError> {
        clip_service::create_clip(&db, ContentType::Image, image_data)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = window.show();
    let _ = window.set_focus();

    let summary = ClipSummary {
        id: clip.id,
        content_type: clip.content_type,
        preview: clip.preview,
        is_favorite: clip.is_favorite,
        is_sensitive: clip.is_sensitive,
        created_at: clip.created_at,
    };
    let _ = app_handle.emit("clip-created", &summary);

    Ok(summary)
}
