CREATE TABLE IF NOT EXISTS inline_comment_threads (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    codelab_id VARCHAR(255) NOT NULL,
    anchor_key VARCHAR(512) NOT NULL,
    target_type VARCHAR(16) NOT NULL,
    target_step_id VARCHAR(255),
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    selected_text TEXT NOT NULL,
    content_hash VARCHAR(128) NOT NULL,
    created_by_attendee_id VARCHAR(255) NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE,
    UNIQUE (codelab_id, anchor_key)
);

CREATE INDEX IF NOT EXISTS idx_inline_comment_threads_target
    ON inline_comment_threads (codelab_id, target_type, target_step_id);

CREATE INDEX IF NOT EXISTS idx_inline_comment_threads_hash_offsets
    ON inline_comment_threads (codelab_id, target_type, target_step_id, content_hash, start_offset, end_offset);

CREATE TABLE IF NOT EXISTS inline_comment_messages (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    thread_id VARCHAR(255) NOT NULL,
    codelab_id VARCHAR(255) NOT NULL,
    author_role VARCHAR(16) NOT NULL,
    author_id VARCHAR(255) NOT NULL,
    author_name VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (thread_id) REFERENCES inline_comment_threads(id) ON DELETE CASCADE,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_inline_comment_messages_thread
    ON inline_comment_messages (thread_id, created_at);
