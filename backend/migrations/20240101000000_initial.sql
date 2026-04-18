-- Initial schema
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    bare_clone_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('jira', 'gitlab', 'scratch')),
    source_ref TEXT,
    state TEXT NOT NULL DEFAULT 'provisioning' CHECK (state IN ('provisioning', 'running', 'stopped', 'terminated')),
    opencode_port INTEGER,
    workspace_path TEXT,
    project_id TEXT REFERENCES projects(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
