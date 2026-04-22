-- Remove dead project_id column and projects table
ALTER TABLE sessions DROP COLUMN project_id;
DROP TABLE IF EXISTS projects;
