-- Add require_submission to codelabs
ALTER TABLE codelabs ADD COLUMN require_submission INTEGER DEFAULT 0;
