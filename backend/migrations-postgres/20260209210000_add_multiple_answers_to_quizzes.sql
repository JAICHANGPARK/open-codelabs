-- Add correct_answers to quizzes table
ALTER TABLE quizzes ADD COLUMN correct_answers TEXT;

-- Migrate existing correct_answer to correct_answers format.
UPDATE quizzes SET correct_answers = '[' || CAST(correct_answer AS TEXT) || ']';
