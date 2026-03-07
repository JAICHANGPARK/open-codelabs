-- Add completion status to attendees table
ALTER TABLE attendees ADD COLUMN is_completed INTEGER DEFAULT 0;
ALTER TABLE attendees ADD COLUMN completed_at TEXT;
