CREATE TABLE IF NOT EXISTS cli_auth_requests (
    id TEXT PRIMARY KEY,
    poll_token_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at_epoch BIGINT NOT NULL,
    expires_at_epoch BIGINT NOT NULL,
    approved_at_epoch BIGINT,
    approved_by TEXT,
    consumed_at_epoch BIGINT
);

CREATE INDEX IF NOT EXISTS idx_cli_auth_requests_expires_at
    ON cli_auth_requests (expires_at_epoch);
