-- Migration: Add structure_type to codeserver_workspaces
-- SQLite
ALTER TABLE codeserver_workspaces ADD COLUMN structure_type TEXT NOT NULL DEFAULT 'branch';

-- Postgres
-- ALTER TABLE codeserver_workspaces ADD COLUMN structure_type TEXT NOT NULL DEFAULT 'branch';
