CREATE TABLE IF NOT EXISTS codeserver_workspaces (
    id TEXT PRIMARY KEY,
    codelab_id TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CAST(CURRENT_TIMESTAMP AS TEXT),
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);

CREATE INDEX idx_codeserver_workspace_codelab ON codeserver_workspaces(codelab_id);
