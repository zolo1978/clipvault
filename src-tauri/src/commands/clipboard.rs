use crate::error::AppError;
use crate::models::*;
use crate::services::clip_service;
use crate::state::AppState;
use std::sync::atomic::Ordering;
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
    window: tauri::WebviewWindow,
) -> Result<(), AppError> {
    if state.is_pasting.load(Ordering::SeqCst) {
        return Err(AppError::Validation("paste already in progress".into()));
    }
    state.is_pasting.store(true, Ordering::SeqCst);

    let result = do_paste(id, &state, &window).await;

    state.is_pasting.store(false, Ordering::SeqCst);
    let _ = window.show();
    result
}

async fn do_paste(
    id: String,
    state: &AppState,
    window: &tauri::WebviewWindow,
) -> Result<(), AppError> {
    // 1. Read clip from DB
    let db = state.db.clone();
    let clip = tokio::task::spawn_blocking(move || {
        clip_service::get_clip(&db, &id)?
            .ok_or_else(|| AppError::NotFound(format!("clip not found: {id}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if clip.content_type != ContentType::Text {
        return Err(AppError::Validation("only text paste is supported".into()));
    }

    let text = String::from_utf8(clip.content)
        .map_err(|e| AppError::Internal(format!("invalid utf8: {e}")))?;

    // 2. Hide window so Cmd+V goes to the user's previous app
    window
        .hide()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 3. Wait for window hide to take effect
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // 4. Save current clipboard, set new text
    let text_clone = text.clone();
    let saved = tokio::task::spawn_blocking(move || {
        let prev = read_clipboard_text();
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(&text_clone);
        }
        prev
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // 5. Wait for clipboard to be read by system
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 6. Simulate Cmd+V
    simulate_paste()?;

    // 7. Wait for target app to read the clipboard
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 8. Restore previous clipboard content
    if let Some(prev) = saved {
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(&prev);
            }
        })
        .await;
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
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if clip.content_type != ContentType::Image {
        return Err(AppError::Validation("not an image clip".into()));
    }

    if !uuid::Uuid::parse_str(&clip.id).is_ok() {
        return Err(AppError::Validation("invalid clip id".into()));
    }

    let tmp = std::env::temp_dir().join(format!("clipvault-preview-{}.png", clip.id));

    // Write with restricted permissions
    let tmp_clone = tmp.clone();
    let content = clip.content;
    tokio::task::spawn_blocking(move || {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&tmp_clone)
            .and_then(|mut f| std::io::Write::write_all(&mut f, &content))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    tokio::process::Command::new("open")
        .arg(&tmp)
        .spawn()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Schedule cleanup after 30s
    let cleanup_path = tmp.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        let _ = tokio::fs::remove_file(&cleanup_path).await;
    });

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
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if clip.content_type != ContentType::FilePath {
        return Err(AppError::Validation("not a file path clip".into()));
    }

    let path = String::from_utf8(clip.content)
        .map_err(|e| AppError::Internal(format!("invalid utf8: {e}")))?;

    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(format!("path not found: {path}")));
    }

    let canonical = p
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("cannot resolve path: {e}")))?;

    // Reject system paths
    let path_str = canonical.to_string_lossy();
    if path_str.starts_with("/System/") || path_str.starts_with("/private/var/") {
        return Err(AppError::Validation("system path not allowed".into()));
    }

    tokio::process::Command::new("open")
        .arg("-R")
        .arg(&canonical)
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
            enigo
                .key(Key::Meta, Direction::Press)
                .map_err(|e| AppError::Internal(format!("meta press: {e}")))?;
            enigo
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| AppError::Internal(format!("v click: {e}")))?;
            enigo
                .key(Key::Meta, Direction::Release)
                .map_err(|e| AppError::Internal(format!("meta release: {e}")))?;
        }

        #[cfg(target_os = "windows")]
        {
            use enigo::Direction;
            enigo
                .key(Key::Control, Direction::Press)
                .map_err(|e| AppError::Internal(format!("ctrl press: {e}")))?;
            enigo
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| AppError::Internal(format!("v click: {e}")))?;
            enigo
                .key(Key::Control, Direction::Release)
                .map_err(|e| AppError::Internal(format!("ctrl release: {e}")))?;
        }

        Ok(())
    })
}
