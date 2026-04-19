-- Add opencode_session_id and opencode_path_prefix to track OpenCode session details
ALTER TABLE sessions ADD COLUMN opencode_session_id TEXT;
ALTER TABLE sessions ADD COLUMN opencode_path_prefix TEXT;
