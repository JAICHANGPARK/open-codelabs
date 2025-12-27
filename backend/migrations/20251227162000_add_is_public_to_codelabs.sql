-- Add is_public column to codelabs table
ALTER TABLE codelabs ADD COLUMN is_public INTEGER NOT NULL DEFAULT 1;
