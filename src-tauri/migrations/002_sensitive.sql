-- V1.1: sensitive data support
ALTER TABLE clips ADD COLUMN is_sensitive INTEGER NOT NULL DEFAULT 0;
