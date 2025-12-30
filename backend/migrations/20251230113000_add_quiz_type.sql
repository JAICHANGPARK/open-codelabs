-- Add quiz_type to quizzes table
ALTER TABLE quizzes ADD COLUMN quiz_type TEXT DEFAULT 'multiple_choice';
