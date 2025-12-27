-- Create feedback table
CREATE TABLE IF NOT EXISTS feedback (
    id VARCHAR(255) PRIMARY KEY,
    codelab_id VARCHAR(255) NOT NULL,
    difficulty VARCHAR(50) NOT NULL,
    satisfaction VARCHAR(50) NOT NULL,
    comment TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
