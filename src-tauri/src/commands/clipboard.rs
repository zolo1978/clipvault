use crate::error::AppError;
use crate::models::*;
use crate::services::clip_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn create_clip(
    content_type: ContentType,
    content: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Clip, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || clip_service::create_clip(&db, content_type, content))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn paste_clip(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db = state.db.clone();
    let clip = tokio::task::spawn_blocking(move || {
        clip_service::get_clip(&db, &id)?
            .ok_or_else(|| AppError::NotFound(format!("clip not found: {id}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    if clip.content_type != ContentType::Text {
        return Err(AppError::Validation("only text paste is supported".into()));
    }

    let text = String::from_utf8(clip.content)
        .map_err(|e| AppError::Internal(format!("invalid utf8: {e}")))?;

    let text_clone = text.clone();
    let saved = tokio::task::spawn_blocking(move || {
        let prev = read_clipboard_text();
        {
            let mut cb = arboard::Clipboard::new().ok()?;
            cb.set_text(&text_clone).ok()?;
        }
        prev
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    simulate_paste()?;

    if let Some(prev) = saved {
        tokio::task::spawn_blocking(move || {
            let mut cb = arboard::Clipboard::new().ok()?;
            cb.set_text(&prev).ok()
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(())
}

fn read_clipboard_text() -> Option<String> {
    let mut cb = arboard::Clipboard::new().ok()?;
    cb.get_text().ok()
}

#[tauri::command]
pub async fn view_image_clip(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db = state.db.clone();
    let clip = tokio::task::spawn_blocking(move || {
        clip_service::get_clip(&db, &id)?
            .ok_or_else(|| AppError::NotFound(format!("clip not found: {id}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    if clip.content_type != ContentType::Image {
        return Err(AppError::Validation("not an image clip".into()));
    }

    if !uuid::Uuid::parse_str(&clip.id).is_ok() {
        return Err(AppError::Validation("invalid clip id".into()));
    }

    let tmp = std::env::temp_dir().join(format!("clipvault-preview-{}.png", clip.id));
    tokio::fs::write(&tmp, &clip.content)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tokio::process::Command::new("open")
        .arg(&tmp)
        .spawn()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn reveal_path(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db = state.db.clone();
    let clip = tokio::task::spawn_blocking(move || {
        clip_service::get_clip(&db, &id)?
            .ok_or_else(|| AppError::NotFound(format!("clip not found: {id}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    if clip.content_type != ContentType::FilePath {
        return Err(AppError::Validation("not a file path clip".into()));
    }

    let path = String::from_utf8(clip.content)
        .map_err(|e| AppError::Internal(format!("invalid utf8: {e}")))?;

    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(format!("path not found: {path}")));
    }

    tokio::process::Command::new("open")
        .arg("-R")
        .arg(&path)
        .spawn()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

fn simulate_paste() -> Result<(), AppError> {
    tokio::task::block_in_place(|| {
        use enigo::{Key, Keyboard, Settings};
        let mut enigo = enigo::Enigo::new(&Settings::default())
            .map_err(|e| AppError::Internal(format!("enigo init: {e}")))?;

        #[cfg(target_os = "macos")]
        {
            use enigo::Direction;
            enigo.key(Key::Meta, Direction::Press)
                .map_err(|e| AppError::Internal(format!("meta press: {e}")))?;
            enigo.key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| AppError::Internal(format!("v click: {e}")))?;
            enigo.key(Key::Meta, Direction::Release)
                .map_err(|e| AppError::Internal(format!("meta release: {e}")))?;
        }

        #[cfg(target_os = "windows")]
        {
            use enigo::Direction;
            enigo.key(Key::Control, Direction::Press)
                .map_err(|e| AppError::Internal(format!("ctrl press: {e}")))?;
            enigo.key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| AppError::Internal(format!("v click: {e}")))?;
            enigo.key(Key::Control, Direction::Release)
                .map_err(|e| AppError::Internal(format!("ctrl release: {e}")))?;
        }

        Ok(())
    })
}
