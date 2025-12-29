-- Add quiz settings to codelabs table
ALTER TABLE codelabs ADD COLUMN quiz_enabled INTEGER DEFAULT 0;
ALTER TABLE codelabs ADD COLUMN require_quiz INTEGER DEFAULT 0;
ALTER TABLE codelabs ADD COLUMN require_feedback INTEGER DEFAULT 0;

-- Create quizzes table
CREATE TABLE IF NOT EXISTS quizzes (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    codelab_id VARCHAR(255) NOT NULL,
    question TEXT NOT NULL,
    options TEXT NOT NULL, -- JSON array of strings
    correct_answer INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);
