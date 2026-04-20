-- Add model column to sessions table
ALTER TABLE sessions ADD COLUMN model TEXT NOT NULL DEFAULT 'github-copilot/gpt-5-mini';
