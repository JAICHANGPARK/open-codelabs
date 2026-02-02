-- Create ai_threads table to group messages in a conversation
CREATE TABLE IF NOT EXISTS ai_threads (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    user_id TEXT NOT NULL,
    user_type TEXT NOT NULL, -- 'admin' or 'attendee'
    codelab_id TEXT, -- Optional: link to a specific codelab
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create ai_messages table to store individual messages in a thread
CREATE TABLE IF NOT EXISTS ai_messages (
    id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL,
    role TEXT NOT NULL, -- 'user' or 'model'
    content TEXT NOT NULL,
    grounding_metadata TEXT, -- JSON string
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (thread_id) REFERENCES ai_threads(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_threads_user ON ai_threads(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_messages_thread ON ai_messages(thread_id);
