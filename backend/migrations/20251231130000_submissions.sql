CREATE TABLE submissions (
    id TEXT PRIMARY KEY,
    codelab_id TEXT NOT NULL,
    attendee_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs (id),
    FOREIGN KEY (attendee_id) REFERENCES attendees (id)
);
