// repositories/clip_repo.rs — Clip data access layer
// All rusqlite operations live here. Callers wrap in spawn_blocking.

use crate::error::AppError;
use crate::models::{Clip, ClipSummary, ContentType};
use rusqlite::{params, Connection};

// -- Row mappers --

fn row_to_clip(row: &rusqlite::Row<'_>) -> rusqlite::Result<Clip> {
    Ok(Clip {
        id: row.get(0)?,
        content_type: ContentType::from_str(&row.get::<_, String>(1)?)
            .unwrap_or(ContentType::Text),
        content: row.get(2)?,
        preview: row.get(3)?,
        content_hash: row.get(4)?,
        is_favorite: row.get::<_, i32>(5)? != 0,
        created_at: row.get(6)?,
    })
}

fn row_to_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipSummary> {
    Ok(ClipSummary {
        id: row.get(0)?,
        content_type: ContentType::from_str(&row.get::<_, String>(1)?)
            .unwrap_or(ContentType::Text),
        preview: row.get(2)?,
        is_favorite: row.get::<_, i32>(3)? != 0,
        created_at: row.get(4)?,
    })
}

// -- Public API --

/// Insert a new clip. If content_hash already exists (dedup), return the existing record.
pub fn insert_clip(
    conn: &Connection,
    id: &str,
    content_type: &str,
    content: Vec<u8>,
    preview: &str,
    content_hash: &str,
    created_at: i64,
) -> Result<Clip, AppError> {
    let changed = conn.execute(
        "INSERT OR IGNORE INTO clips \
         (id, content_type, content, preview, content_hash, is_favorite, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
        params![id, content_type, content, preview, content_hash, created_at],
    )?;

    if changed == 0 {
        // Duplicate — return existing record
        return conn
            .query_row(
                "SELECT id, content_type, content, preview, content_hash, is_favorite, created_at \
                 FROM clips WHERE content_hash = ?1",
                params![content_hash],
                row_to_clip,
            )
            .map_err(AppError::Database);
    }

    Ok(Clip {
        id: id.to_string(),
        content_type: ContentType::from_str(content_type).unwrap_or(ContentType::Text),
        content,
        preview: preview.to_string(),
        content_hash: content_hash.to_string(),
        is_favorite: false,
        created_at,
    })
}

/// List clips with pagination, newest first. Optional content_type filter.
pub fn list_clips(
    conn: &Connection,
    limit: u32,
    offset: u32,
    content_type: Option<&str>,
) -> Result<Vec<ClipSummary>, AppError> {
    let mut sql = String::from(
        "SELECT id, content_type, preview, is_favorite, created_at FROM clips",
    );
    let mut param_boxed: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ct) = content_type {
        sql.push_str(" WHERE content_type = ?");
        param_boxed.push(Box::new(ct.to_string()));
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    param_boxed.push(Box::new(limit));
    param_boxed.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_boxed.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_summary)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// FTS5 full-text search with BM25 ranking.
pub fn search_clips(
    conn: &Connection,
    query: &str,
    content_type: Option<&str>,
    limit: u32,
) -> Result<Vec<ClipSummary>, AppError> {
    // Strip FTS5 operators, keep only literal chars for safe phrase matching
    let sanitized: String = query
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let fts_query = format!("\"{}\"*", trimmed.replace('"', "\"\""));

    let mut sql = String::from(
        "SELECT c.id, c.content_type, c.preview, c.is_favorite, c.created_at \
         FROM clips c \
         JOIN clips_fts f ON f.rowid = c.rowid \
         WHERE clips_fts MATCH ?",
    );
    let mut param_boxed: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(fts_query)];

    if let Some(ct) = content_type {
        sql.push_str(" AND c.content_type = ?");
        param_boxed.push(Box::new(ct.to_string()));
    }

    sql.push_str(" ORDER BY f.rank LIMIT ?");
    param_boxed.push(Box::new(limit));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_boxed.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_summary)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Get a single clip by ID. Returns None if not found.
pub fn get_clip(conn: &Connection, id: &str) -> Result<Option<Clip>, AppError> {
    let result = conn.query_row(
        "SELECT id, content_type, content, preview, content_hash, is_favorite, created_at \
         FROM clips WHERE id = ?1",
        params![id],
        row_to_clip,
    );
    match result {
        Ok(clip) => Ok(Some(clip)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// Delete a clip by ID. Returns true if a row was deleted.
pub fn delete_clip(conn: &Connection, id: &str) -> Result<bool, AppError> {
    let rows = conn.execute("DELETE FROM clips WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

/// Toggle the is_favorite flag. Returns the updated summary, or None if not found.
pub fn toggle_favorite(
    conn: &Connection,
    id: &str,
) -> Result<Option<ClipSummary>, AppError> {
    let affected = conn.execute(
        "UPDATE clips SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END \
         WHERE id = ?1",
        params![id],
    )?;
    if affected == 0 {
        return Ok(None);
    }
    let summary = conn.query_row(
        "SELECT id, content_type, preview, is_favorite, created_at FROM clips WHERE id = ?1",
        params![id],
        row_to_summary,
    )?;
    Ok(Some(summary))
}

/// Purge old non-favorite clips by age and/or count. Returns total deleted.
pub fn purge_old(
    conn: &Connection,
    keep_days: Option<u32>,
    keep_count: Option<u32>,
) -> Result<u64, AppError> {
    let mut deleted: u64 = 0;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    if let Some(days) = keep_days {
        if days == 0 {
            // keep_days: 0 means no time-based purge
        } else {
            let cutoff_ms = now_ms - (days as i64 * 24 * 3600 * 1000);
            let rows = conn.execute(
                "DELETE FROM clips WHERE is_favorite = 0 AND created_at < ?1",
                params![cutoff_ms],
            )?;
            deleted += rows as u64;
        }
    }

    if let Some(count) = keep_count {
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM clips WHERE is_favorite = 0",
            [],
            |row| row.get(0),
        )?;
        if total > count as i64 {
            let excess = total - count as i64;
            let rows = conn.execute(
                "DELETE FROM clips WHERE is_favorite = 0 AND id IN \
                 (SELECT id FROM clips WHERE is_favorite = 0 \
                  ORDER BY created_at ASC LIMIT ?1)",
                params![excess],
            )?;
            deleted += rows as u64;
        }
    }

    Ok(deleted)
}
