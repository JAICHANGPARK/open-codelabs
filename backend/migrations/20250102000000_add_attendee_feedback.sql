-- Add attendee_id to feedback table
ALTER TABLE feedback ADD COLUMN attendee_id VARCHAR(255);

-- Then add a unique index (which acts as a constraint).
CREATE UNIQUE INDEX idx_feedback_codelab_attendee ON feedback(codelab_id, attendee_id);
