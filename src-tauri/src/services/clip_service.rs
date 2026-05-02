// services/clip_service.rs — Clip business logic
// Pure Rust, no Tauri dependency. Hash computation, validation, dedup orchestration.

use crate::error::AppError;
use crate::models::{Clip, ClipSummary, ContentType};
use crate::repositories::clip_repo;
use base64::Engine;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

// -- Helpers --

/// SHA-256 hex digest of raw bytes.
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Generate preview string from content.
/// For images, returns a base64 data URI of a small thumbnail.
fn make_preview(content: &[u8], ct: &ContentType) -> String {
    match ct {
        ContentType::Text => {
            let text = String::from_utf8_lossy(content);
            text.chars().take(200).collect()
        }
        ContentType::Image => {
            match generate_thumbnail(content, 200) {
                Some(thumb_bytes) => {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&thumb_bytes);
                    format!("data:image/png;base64,{b64}")
                }
                None => "图片".to_string(),
            }
        }
        ContentType::FilePath => String::from_utf8_lossy(content).to_string(),
    }
}

/// Decode PNG and resize to max_width maintaining aspect ratio, re-encode as PNG.
fn generate_thumbnail(data: &[u8], max_width: u32) -> Option<Vec<u8>> {
    use std::io::Cursor;
    let decoder = png::Decoder::new(Cursor::new(data));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let bytes = &buf[..info.buffer_size()];
    let w = info.width;
    let h = info.height;

    let rgba = to_rgba(bytes, w, h, info.color_type, info.bit_depth)?;

    if w <= max_width {
        return encode_rgba_png(&rgba, w, h);
    }

    let ratio = max_width as f64 / w as f64;
    let new_h = (h as f64 * ratio) as u32;

    let mut out = Vec::with_capacity((max_width * new_h * 4) as usize);
    for y in 0..new_h {
        let src_y = ((y as f64 / ratio) as usize).min(h as usize - 1);
        for x in 0..max_width {
            let src_x = ((x as f64 / ratio) as usize).min(w as usize - 1);
            let src_idx = (src_y * w as usize + src_x) * 4;
            out.extend_from_slice(&rgba[src_idx..src_idx + 4]);
        }
    }

    encode_rgba_png(&out, max_width, new_h)
}

/// Convert decoded PNG pixels to RGBA8 regardless of source color type.
fn to_rgba(
    bytes: &[u8],
    w: u32,
    h: u32,
    color_type: png::ColorType,
    _bit_depth: png::BitDepth,
) -> Option<Vec<u8>> {
    let pixel_count = (w * h) as usize;
    match color_type {
        png::ColorType::Rgba => Some(bytes.to_vec()),
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity(pixel_count * 4);
            for chunk in bytes.chunks(3) {
                out.extend_from_slice(chunk);
                out.push(255);
            }
            Some(out)
        }
        png::ColorType::GrayscaleAlpha => {
            let mut out = Vec::with_capacity(pixel_count * 4);
            for chunk in bytes.chunks(2) {
                out.push(chunk[0]);
                out.push(chunk[0]);
                out.push(chunk[0]);
                out.push(chunk[1]);
            }
            Some(out)
        }
        png::ColorType::Grayscale => {
            let mut out = Vec::with_capacity(pixel_count * 4);
            for &v in bytes {
                out.push(v);
                out.push(v);
                out.push(v);
                out.push(255);
            }
            Some(out)
        }
        png::ColorType::Indexed => {
            // Should be expanded by default decoder transforms, handle defensively
            None
        }
    }
}

fn encode_rgba_png(data: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    use std::io::Cursor;
    let mut result = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut result), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(data).ok()?;
    }
    Some(result)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

// -- Public API --

/// Create a clip with dedup. Returns existing record if content hash matches.
pub fn create_clip(
    db: &Arc<Mutex<Connection>>,
    content_type: ContentType,
    content: Vec<u8>,
) -> Result<Clip, AppError> {
    let preview = make_preview(&content, &content_type);
    create_clip_with_preview(db, content_type, content, &preview)
}

/// Create a clip with a custom preview string.
pub fn create_clip_with_preview(
    db: &Arc<Mutex<Connection>>,
    content_type: ContentType,
    content: Vec<u8>,
    preview: &str,
) -> Result<Clip, AppError> {
    if content.is_empty() {
        return Err(AppError::Validation("content must not be empty".into()));
    }

    let hash = compute_hash(&content);
    let id = uuid::Uuid::now_v7().to_string();
    let created_at = now_ms();

    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    clip_repo::insert_clip(
        &conn,
        &id,
        content_type.as_str(),
        content,
        preview,
        &hash,
        created_at,
    )
}

/// Search clips via FTS5.
pub fn search(
    db: &Arc<Mutex<Connection>>,
    query: &str,
    content_type: Option<&str>,
    limit: u32,
) -> Result<Vec<ClipSummary>, AppError> {
    if query.is_empty() {
        return Err(AppError::Validation("query must not be empty".into()));
    }
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    clip_repo::search_clips(&conn, query, content_type, limit)
}

/// List recent clips with pagination.
pub fn list_recent(
    db: &Arc<Mutex<Connection>>,
    limit: u32,
    offset: u32,
    content_type: Option<&str>,
) -> Result<Vec<ClipSummary>, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    clip_repo::list_clips(&conn, limit, offset, content_type)
}

/// Get a single clip by ID.
pub fn get_clip(
    db: &Arc<Mutex<Connection>>,
    id: &str,
) -> Result<Option<Clip>, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    clip_repo::get_clip(&conn, id)
}

/// Delete a single clip. Returns NotFound if missing.
pub fn delete_clip(
    db: &Arc<Mutex<Connection>>,
    id: &str,
) -> Result<(), AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let deleted = clip_repo::delete_clip(&conn, id)?;
    if !deleted {
        return Err(AppError::NotFound(format!("clip not found: {id}")));
    }
    Ok(())
}

/// Batch delete clips by IDs.
pub fn delete_clips(
    db: &Arc<Mutex<Connection>>,
    ids: &[String],
) -> Result<(), AppError> {
    if ids.is_empty() {
        return Ok(());
    }
    if ids.len() > 500 {
        return Err(AppError::Validation("batch delete limited to 500 items".into()));
    }
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let placeholders: Vec<String> = ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    let sql = format!(
        "DELETE FROM clips WHERE id IN ({})",
        placeholders.join(", ")
    );
    let params: Vec<&dyn rusqlite::types::ToSql> =
        ids.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    conn.execute(&sql, params.as_slice())?;
    Ok(())
}

/// Toggle favorite status. Returns updated summary.
pub fn toggle_favorite(
    db: &Arc<Mutex<Connection>>,
    id: &str,
) -> Result<ClipSummary, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    clip_repo::toggle_favorite(&conn, id)?.ok_or_else(|| {
        AppError::NotFound(format!("clip not found: {id}"))
    })
}

/// Purge old clips based on retention policy.
pub fn purge_clips(
    db: &Arc<Mutex<Connection>>,
    keep_days: Option<u32>,
    keep_count: Option<u32>,
) -> Result<u64, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    clip_repo::purge_old(&conn, keep_days, keep_count)
}
