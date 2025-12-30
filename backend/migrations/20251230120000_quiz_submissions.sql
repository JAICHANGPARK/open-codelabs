-- Create quiz_submissions table
CREATE TABLE IF NOT EXISTS quiz_submissions (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    codelab_id VARCHAR(255) NOT NULL,
    attendee_id VARCHAR(255) NOT NULL,
    quiz_id VARCHAR(255) NOT NULL,
    answer TEXT NOT NULL,
    is_correct INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE,
    FOREIGN KEY (attendee_id) REFERENCES attendees(id) ON DELETE CASCADE,
    FOREIGN KEY (quiz_id) REFERENCES quizzes(id) ON DELETE CASCADE
);
