-- Create attendees table
CREATE TABLE IF NOT EXISTS attendees (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);

-- Create help_requests table
CREATE TABLE IF NOT EXISTS help_requests (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    attendee_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    status TEXT DEFAULT 'pending', -- pending, resolved
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE,
    FOREIGN KEY (attendee_id) REFERENCES attendees(id) ON DELETE CASCADE
);

-- Create chat_messages table (optional for persistence)
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);
