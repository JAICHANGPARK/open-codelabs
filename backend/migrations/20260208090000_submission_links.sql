ALTER TABLE submissions ADD COLUMN submission_type TEXT NOT NULL DEFAULT 'file';
ALTER TABLE submissions ADD COLUMN link_url TEXT;
