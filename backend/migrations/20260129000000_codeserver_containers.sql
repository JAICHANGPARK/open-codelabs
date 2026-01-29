-- Migration: Add codeserver_workspaces table
-- SQLite
CREATE TABLE IF NOT EXISTS codeserver_workspaces (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    codelab_id TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);

CREATE INDEX idx_codeserver_workspace_codelab ON codeserver_workspaces(codelab_id);

-- Postgres
-- CREATE TABLE IF NOT EXISTS codeserver_workspaces (
--     id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
--     codelab_id TEXT NOT NULL UNIQUE,
--     url TEXT NOT NULL,
--     created_at TIMESTAMP NOT NULL DEFAULT NOW(),
--     FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
-- );
--
-- CREATE INDEX idx_codeserver_workspace_codelab ON codeserver_workspaces(codelab_id);
