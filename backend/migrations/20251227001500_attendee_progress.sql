-- Add current_step to attendees table
ALTER TABLE attendees ADD COLUMN current_step INTEGER DEFAULT 1;
