-- Add correct_answers to quizzes table
ALTER TABLE quizzes ADD COLUMN correct_answers TEXT;

-- Migrate existing correct_answer to correct_answers format
-- Since it's SQLite, we can use JSON format
UPDATE quizzes SET correct_answers = '[' || correct_answer || ']';
