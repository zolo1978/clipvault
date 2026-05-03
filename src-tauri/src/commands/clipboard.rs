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

    let _ = window.show();

    // Delay resetting is_pasting to prevent monitor from re-capturing pasted content
    let flag = state.is_pasting.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        flag.store(false, Ordering::SeqCst);
    });

    result
}

async fn do_paste(
    id: String,
    state: &AppState,
    window: &tauri::WebviewWindow,
) -> Result<(), AppError> {
    // 1. Read clip from DB
    let db = state.db.clone();
    let id_for_store = id.clone();
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

    let text = if clip.is_sensitive {
        let mut store = state.sensitive_store.lock().await;
        match store.get(&id_for_store) {
            Some(crate::services::sensitive_store::EntryState::Available(content)) => {
                String::from_utf8(content)
                    .map_err(|e| AppError::Internal(format!("invalid utf8: {e}")))?
            }
            Some(crate::services::sensitive_store::EntryState::Expired) => {
                return Err(AppError::Validation(
                    "sensitive content has expired".into(),
                ));
            }
            None => {
                return Err(AppError::Validation(
                    "sensitive content not available".into(),
                ));
            }
        }
    } else {
        String::from_utf8(clip.content)
            .map_err(|e| AppError::Internal(format!("invalid utf8: {e}")))?
    };

    eprintln!("[paste] clip_id={} content='{}' ({} bytes)", &id_for_store[..8], &text[..text.len().min(40)], text.len());

    // 2. Set clipboard FIRST (while window still visible, avoids focus-switch race)
    let text_for_cb = text.clone();
    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let mut cb = arboard::Clipboard::new()
            .map_err(|e| AppError::Internal(format!("clipboard init: {e}")))?;
        cb.set_text(&text_for_cb)
            .map_err(|e| AppError::Internal(format!("clipboard set: {e}")))?;
        // Verify
        std::thread::sleep(std::time::Duration::from_millis(20));
        match cb.get_text() {
            Ok(read_back) if read_back == text_for_cb => {
                eprintln!("[paste] clipboard verified OK ({} bytes)", text_for_cb.len());
            }
            Ok(other) => {
                eprintln!(
                    "[paste] WARNING: clipboard mismatch! expected {} bytes, got {} bytes",
                    text_for_cb.len(),
                    other.len(),
                );
            }
            Err(e) => {
                eprintln!("[paste] WARNING: clipboard read-back failed: {e}");
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // 3. NOW hide window — clipboard already has correct content
    window
        .hide()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 4. Wait for app focus switch
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    // 5. Final clipboard check before simulating paste
    let text_for_check = text.clone();
    let check_result = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
        let mut cb = arboard::Clipboard::new()
            .map_err(|e| AppError::Internal(format!("clipboard init: {e}")))?;
        cb.get_text()
            .map_err(|e| AppError::Internal(format!("clipboard read: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    if check_result != text_for_check {
        eprintln!(
            "[paste] CLIPBOARD CHANGED AFTER HIDE! expected '{}', got '{}' — re-setting",
            &text_for_check[..text_for_check.len().min(30)],
            &check_result[..check_result.len().min(30)],
        );
        // Re-set clipboard
        let text_reset = text.clone();
        tokio::task::spawn_blocking(move || -> Result<(), AppError> {
            let mut cb = arboard::Clipboard::new()
                .map_err(|e| AppError::Internal(format!("clipboard init: {e}")))?;
            cb.set_text(&text_reset)
                .map_err(|e| AppError::Internal(format!("clipboard re-set: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // 6. Simulate Cmd+V
    simulate_paste().await?;

    // 7. Wait for target app to consume the paste
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    Ok(())
}

#[tauri::command]
pub async fn get_sensitive_clip_content(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let mut store = state.sensitive_store.lock().await;
    match store.get(&id) {
        Some(crate::services::sensitive_store::EntryState::Available(content)) => {
            String::from_utf8(content)
                .map_err(|e| AppError::Internal(format!("invalid utf8: {e}")))
        }
        Some(crate::services::sensitive_store::EntryState::Expired) => {
            Err(AppError::Validation("sensitive content has expired".into()))
        }
        None => Err(AppError::NotFound(format!(
            "sensitive content not found: {id}"
        ))),
    }
}

#[tauri::command]
pub async fn check_sensitive_expired(
    id: String,
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let store = state.sensitive_store.lock().await;
    Ok(store.is_expired(&id))
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

async fn simulate_paste() -> Result<(), AppError> {
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "macos")]
        {
            use core_graphics::event::{CGEvent, CGEventTapLocation};
            use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

            let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                .map_err(|_| AppError::Internal("CGEventSource init failed".into()))?;

            // kVK_Command = 0x37 (55), kVK_ANSI_V = 0x09 (9)
            let event = CGEvent::new_keyboard_event(source.clone(), 55, true)
                .map_err(|_| AppError::Internal("cmd down event failed".into()))?;
            event.post(CGEventTapLocation::HID);

            let event = CGEvent::new_keyboard_event(source.clone(), 9, true)
                .map_err(|_| AppError::Internal("v down event failed".into()))?;
            event.post(CGEventTapLocation::HID);

            let event = CGEvent::new_keyboard_event(source.clone(), 9, false)
                .map_err(|_| AppError::Internal("v up event failed".into()))?;
            event.post(CGEventTapLocation::HID);

            let event = CGEvent::new_keyboard_event(source, 55, false)
                .map_err(|_| AppError::Internal("cmd up event failed".into()))?;
            event.post(CGEventTapLocation::HID);
        }

        #[cfg(target_os = "windows")]
        {
            use enigo::{Direction, Key, Keyboard, Settings};
            let mut enigo = enigo::Enigo::new(&Settings::default())
                .map_err(|e| AppError::Internal(format!("enigo init: {e}")))?;
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
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}
