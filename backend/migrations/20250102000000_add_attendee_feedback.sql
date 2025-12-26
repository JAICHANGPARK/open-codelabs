-- Add attendee_id to feedback table
-- SQLite doesn't support adding a column with a UNIQUE constraint in one go easily/cleanly if preserving data is tricky or dealing with complex constraints.
-- But we can add the column first.
ALTER TABLE feedback ADD COLUMN attendee_id TEXT;

-- Then add a unique index (which acts as a constraint).
-- Note: Existing rows will have NULL attendee_id. This might violate uniqueness if multiple NULLs are treated as duplicates (depends on DB, usually NULL != NULL). 
-- However, for future rows, we want (codelab_id, attendee_id) to be unique.
CREATE UNIQUE INDEX idx_feedback_codelab_attendee ON feedback(codelab_id, attendee_id);
