-- Create ai_conversations table to store AI questions and answers
CREATE TABLE IF NOT EXISTS ai_conversations (
    id TEXT PRIMARY KEY,
    codelab_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    user_type TEXT NOT NULL, -- 'admin' or 'attendee'
    user_name TEXT NOT NULL,
    step_number INTEGER,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    model TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_conversations_codelab ON ai_conversations(codelab_id);
CREATE INDEX IF NOT EXISTS idx_ai_conversations_user ON ai_conversations(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_conversations_created ON ai_conversations(created_at);
