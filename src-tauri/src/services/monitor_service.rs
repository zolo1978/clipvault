use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

use rusqlite::Connection;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::{ClipSummary, ContentType, MonitorStatus, Sensitivity};
use crate::services::clip_service;
use crate::state::AppConfig;

pub struct MonitorService {
    db: Arc<Mutex<Connection>>,
    app_handle: AppHandle,
    stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
    config: Arc<RwLock<AppConfig>>,
    is_running: Arc<AtomicBool>,
    clips_captured: Arc<AtomicU64>,
    is_pasting: Arc<AtomicBool>,
    sensitive_store: Arc<tokio::sync::Mutex<crate::services::sensitive_store::SensitiveStore>>,
}

impl MonitorService {
    pub fn new(
        db: Arc<Mutex<Connection>>,
        app_handle: AppHandle,
        config: Arc<RwLock<AppConfig>>,
        is_pasting: Arc<AtomicBool>,
        sensitive_store: Arc<tokio::sync::Mutex<crate::services::sensitive_store::SensitiveStore>>,
    ) -> Self {
        Self {
            db,
            app_handle,
            stop_tx: None,
            config,
            is_running: Arc::new(AtomicBool::new(false)),
            clips_captured: Arc::new(AtomicU64::new(0)),
            is_pasting,
            sensitive_store,
        }
    }

    pub fn start(&mut self) -> Result<(), AppError> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let db = self.db.clone();
        let app_handle = self.app_handle.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        let clips_captured = self.clips_captured.clone();
        let is_pasting = self.is_pasting.clone();
        let sensitive_store = self.sensitive_store.clone();

        is_running.store(true, Ordering::Relaxed);

        tauri::async_runtime::spawn(async move {
            run_monitor_loop(db, app_handle, config, is_running, clips_captured, stop_rx, is_pasting, sensitive_store).await;
        });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), AppError> {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        self.is_running.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn status(&self) -> MonitorStatus {
        MonitorStatus {
            is_running: self.is_running.load(Ordering::Relaxed),
            clips_captured: self.clips_captured.load(Ordering::Relaxed),
        }
    }
}

async fn run_monitor_loop(
    db: Arc<Mutex<Connection>>,
    app_handle: AppHandle,
    config: Arc<RwLock<AppConfig>>,
    is_running: Arc<AtomicBool>,
    clips_captured: Arc<AtomicU64>,
    mut stop_rx: tokio::sync::oneshot::Receiver<()>,
    is_pasting: Arc<AtomicBool>,
    sensitive_store: Arc<tokio::sync::Mutex<crate::services::sensitive_store::SensitiveStore>>,
) {
    let mut last_hash: Option<String> = None;
    let mut loop_count: u64 = 0;

    loop {
        let interval_ms = config.read().await.monitor_interval_ms;
        let delay = tokio::time::Duration::from_millis(interval_ms);

        tokio::select! {
            _ = &mut stop_rx => {
                is_running.store(false, Ordering::Relaxed);
                break;
            }
            _ = tokio::time::sleep(delay) => {}
        }

        // Check is_pasting BEFORE touching NSPasteboard to avoid contention
        if is_pasting.load(Ordering::SeqCst) {
            continue;
        }

        let clipboard_content = match read_clipboard() {
            Some(content) => content,
            None => continue,
        };

        // Re-check after clipboard read — paste may have started during read
        if is_pasting.load(Ordering::SeqCst) {
            continue;
        }

        let (content_type, content_bytes, hash) = match &clipboard_content {
            ClipboardContent::Text(t) => {
                let hash = compute_hash(t.as_bytes());
                (ContentType::Text, t.clone().into_bytes(), hash)
            }
            ClipboardContent::Image(data) => {
                let hash = compute_hash(data);
                (ContentType::Image, data.clone(), hash)
            }
            ClipboardContent::FilePath(p) => {
                let hash = compute_hash(p.as_bytes());
                (ContentType::FilePath, p.clone().into_bytes(), hash)
            }
        };

        if last_hash.as_ref() == Some(&hash) {
            continue;
        }
        last_hash = Some(hash.clone());

        // Periodic cleanup of expired sensitive entries
        loop_count += 1;
        if loop_count % 100 == 0 {
            sensitive_store.lock().await.cleanup_expired();
        }

        // Sensitive data detection for text content
        let detection_enabled = config.read().await.sensitive_detection_enabled;
        if detection_enabled && matches!(content_type, ContentType::Text) {
            let text_str = String::from_utf8_lossy(&content_bytes);
            let sensitivity = crate::sensitive::detect_sensitive(&text_str);

            match sensitivity {
                Sensitivity::Transient => continue,
                Sensitivity::Sensitive(kind) => {
                    let masked_preview = kind.masked_preview();
                    let db_clone = db.clone();
                    let store_clone = sensitive_store.clone();
                    let app_handle_clone = app_handle.clone();
                    let clips_captured_clone = clips_captured.clone();
                    let content_for_store = content_bytes.clone();
                    let kind_clone = kind.clone();

                    let result = tokio::task::spawn_blocking(move || {
                        clip_service::create_sensitive_clip(
                            &db_clone,
                            ContentType::Text,
                            &content_bytes,
                            &masked_preview,
                        )
                    })
                    .await;

                    match result {
                        Ok(Ok(clip)) => {
                            store_clone
                                .lock()
                                .await
                                .insert(clip.id.clone(), content_for_store, kind_clone);
                            clips_captured_clone.fetch_add(1, Ordering::Relaxed);
                            let summary = ClipSummary {
                                id: clip.id,
                                content_type: clip.content_type,
                                preview: clip.preview,
                                is_favorite: clip.is_favorite,
                                is_sensitive: clip.is_sensitive,
                                created_at: clip.created_at,
                            };
                            let _ = app_handle_clone.emit("clip-created", &summary);
                        }
                        Ok(Err(_)) | Err(_) => {}
                    }
                    continue;
                }
                Sensitivity::Clean => {}
            }
        }

        // Normal clip creation
        let db_clone = db.clone();
        let app_handle_clone = app_handle.clone();
        let clips_captured_clone = clips_captured.clone();

        let result = tokio::task::spawn_blocking(move || match content_type {
            ContentType::Text => {
                clip_service::create_clip(&db_clone, ContentType::Text, content_bytes)
            }
            ContentType::Image => {
                clip_service::create_clip(&db_clone, ContentType::Image, content_bytes)
            }
            ContentType::FilePath => {
                let preview = String::from_utf8_lossy(&content_bytes).to_string();
                clip_service::create_clip_with_preview(
                    &db_clone,
                    ContentType::FilePath,
                    content_bytes,
                    &preview,
                )
            }
        })
        .await;

        match result {
            Ok(Ok(clip)) => {
                clips_captured_clone.fetch_add(1, Ordering::Relaxed);
                let summary = ClipSummary {
                    id: clip.id,
                    content_type: clip.content_type,
                    preview: clip.preview,
                    is_favorite: clip.is_favorite,
                    is_sensitive: clip.is_sensitive,
                    created_at: clip.created_at,
                };
                let _ = app_handle_clone.emit("clip-created", &summary);
            }
            Ok(Err(_)) | Err(_) => {}
        }
    }
}

enum ClipboardContent {
    Text(String),
    Image(Vec<u8>),
    FilePath(String),
}

fn read_clipboard() -> Option<ClipboardContent> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    // Try text first
    if let Some(text) = clipboard.get_text().ok().filter(|t| !t.is_empty()) {
        // Detect single absolute file path (no spaces, path exists)
        let trimmed = text.trim();
        let is_path = trimmed.starts_with('/')
            && !trimmed.ends_with('\n')
            && trimmed.lines().count() == 1
            && std::path::Path::new(trimmed).exists();
        if is_path {
            return Some(ClipboardContent::FilePath(trimmed.to_string()));
        }
        return Some(ClipboardContent::Text(text));
    }
    // Try image
    if let Ok(image) = clipboard.get_image() {
        let rgba = image.bytes.as_ref();
        let width = image.width;
        let height = image.height;
        let png_data = rgba_to_png(rgba, width, height)?;
        return Some(ClipboardContent::Image(png_data));
    }
    None
}

fn rgba_to_png(rgba: &[u8], width: usize, height: usize) -> Option<Vec<u8>> {
    use std::io::Cursor;
    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut png_data), width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(png_data)
}

fn compute_hash(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
