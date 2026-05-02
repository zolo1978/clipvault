-- ClipVault SQLite Schema
-- Target: SQLite 3.45+ with FTS5 enabled
-- rusqlite bundled mode enables FTS5 by default

-- Enable WAL mode for concurrent reads
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

-----------------------------------------------
-- clips: core clipboard record table
-----------------------------------------------
CREATE TABLE IF NOT EXISTS clips (
    id            TEXT PRIMARY KEY,          -- UUID v7 (time-sortable)
    content_type  TEXT NOT NULL,             -- 'text' | 'image' | 'file_path'
    content       BLOB NOT NULL,             -- raw content (text as UTF-8 bytes, image as binary, path as UTF-8)
    preview       TEXT NOT NULL DEFAULT '',  -- first 200 chars or thumbnail path
    content_hash  TEXT NOT NULL,             -- SHA-256 hex, used for dedup
    is_favorite   INTEGER NOT NULL DEFAULT 0, -- 0 = false, 1 = true
    created_at    INTEGER NOT NULL           -- Unix timestamp in milliseconds
);

-- Dedup: same content_hash should not appear twice
CREATE UNIQUE INDEX IF NOT EXISTS idx_clips_hash ON clips (content_hash);

-- List by time (default view: newest first)
CREATE INDEX IF NOT EXISTS idx_clips_created ON clips (created_at DESC);

-- Filter by type
CREATE INDEX IF NOT EXISTS idx_clips_type ON clips (content_type);

-- Favorites first, then by time
CREATE INDEX IF NOT EXISTS idx_clips_fav_time ON clips (is_favorite DESC, created_at DESC);

-----------------------------------------------
-- clips_fts: full-text search virtual table
-----------------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
    content,
    content='clips',
    content_rowid='rowid',
    tokenize='unicode61'
);

-- FTS5 sync triggers: keep clips_fts in sync with clips
CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
    INSERT INTO clips_fts (rowid, content) VALUES (new.rowid, new.preview);
END;

CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
    INSERT INTO clips_fts (clips_fts, rowid, content) VALUES ('delete', old.rowid, old.preview);
END;

CREATE TRIGGER IF NOT EXISTS clips_au AFTER UPDATE ON clips BEGIN
    INSERT INTO clips_fts (clips_fts, rowid, content) VALUES ('delete', old.rowid, old.preview);
    INSERT INTO clips_fts (rowid, content) VALUES (new.rowid, new.preview);
END;

-----------------------------------------------
-- clip_tags: tag association (P2, V1.x)
-----------------------------------------------
CREATE TABLE IF NOT EXISTS clip_tags (
    clip_id  TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
    tag      TEXT NOT NULL,
    PRIMARY KEY (clip_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_tags_tag ON clip_tags (tag);

-----------------------------------------------
-- Auto-cleanup: delete clips older than N days
-- Called from Rust service layer, not via trigger
-----------------------------------------------
-- Example: DELETE FROM clips WHERE created_at < (unixepoch('now') * 1000 - :keep_ms);
