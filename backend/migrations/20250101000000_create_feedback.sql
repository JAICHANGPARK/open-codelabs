-- Create feedback table
CREATE TABLE IF NOT EXISTS feedback (
    id TEXT PRIMARY KEY,
    codelab_id TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    satisfaction TEXT NOT NULL,
    comment TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
