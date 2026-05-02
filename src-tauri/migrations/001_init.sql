-- ClipVault migration 001: initial schema
-- Do NOT include PRAGMA here — they are set in state.rs before migration runs.

-----------------------------------------------
-- clips: core clipboard record table
-----------------------------------------------
CREATE TABLE IF NOT EXISTS clips (
    id            TEXT PRIMARY KEY,
    content_type  TEXT NOT NULL,
    content       BLOB NOT NULL,
    preview       TEXT NOT NULL DEFAULT '',
    content_hash  TEXT NOT NULL,
    is_favorite   INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_clips_hash ON clips (content_hash);
CREATE INDEX IF NOT EXISTS idx_clips_created ON clips (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clips_type ON clips (content_type);
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
