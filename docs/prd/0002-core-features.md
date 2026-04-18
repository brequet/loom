# PRD-0002: Core Features

## Problem Statement

A developer works on multiple Jira issues and GitLab MRs simultaneously using OpenCode. Currently, they work in a single repository clone, requiring manual branch switching and stashing. They need isolated, parallel work environments that are easy to create, manage, and resume.

## Solution

Loom provides a web Dashboard to create and manage isolated OpenCode sessions. Each session is tied to a Source (Jira issue, GitLab MR, or scratch), gets its own Workspace (git worktree), and runs its own OpenCode web instance. The user interacts with the Dashboard to manage sessions and with OpenCode's web UI to do the actual work.

## User Stories

1. As a developer, I want to see all my active sessions on a dashboard, so that I can quickly navigate between tasks.
2. As a developer, I want to create a session from a Jira issue by typing an issue key (e.g., SAM-234), so that OpenCode starts working on it immediately.
3. As a developer, I want to paste a Jira issue URL to create a session, so that I don't have to extract the issue key manually.
4. As a developer, I want to search Jira issues with predictive autocomplete, so that I can find and select issues without remembering exact keys.
5. As a developer, I want to create a session from a GitLab MR by typing !42 or pasting an MR URL, so that I can review or continue work on it.
6. As a developer, I want to search open GitLab MRs with autocomplete, so that I can find MRs without remembering numbers.
7. As a developer, I want to create a scratch session (no Jira/MR source), so that I can do ad-hoc work in an isolated environment.
8. As a developer, I want each session to automatically have its own git worktree, so that sessions are fully isolated from each other.
9. As a developer, I want OpenCode to receive the Jira issue description or MR details as its initial prompt, so that it understands the task context immediately.
10. As a developer, I want to click a session on the dashboard to open its OpenCode web UI in a new tab, so that I can work on it.
11. As a developer, I want to see the state of each session (provisioning, running, stopped, terminated), so that I know which sessions are active.
12. As a developer, I want to terminate a session from the dashboard, so that its resources (worktree, OpenCode process) are cleaned up.
13. As a developer, I want stopped sessions (crashed/rebooted) to be resumable, so that I don't lose work.
14. As a developer, I want to resume a stopped session from the dashboard with one click, so that recovery is fast.
15. As a developer, I want the dashboard to survive a system reboot, with all non-terminated sessions showing as stopped and resumable.
16. As a developer, I want to configure my Jira and GitLab API credentials via environment variables, so that setup is simple.
17. As a developer, I want to write a base system prompt for OpenCode (repos I work on, preferences), so that every session starts with my context.
18. As a developer, I want the dashboard to show session metadata (source type, issue key, creation time, port), so that I can identify sessions at a glance.

## Implementation Decisions

### Backend Architecture (Rust/Axum)

- **Service layer DI**: AppState holds services (SessionService, JiraService, GitLabService, OpenCodeService), injected via Axum's State extractor. Handlers are thin dispatchers.
- **OpenCodeService**: Manages spawning/stopping `opencode web` processes. Calls OpenCode's REST API via reqwest to create sessions and send initial prompts.
- **JiraService**: Calls Jira REST API to search issues and fetch issue details. Credentials from env vars.
- **GitLabService**: Calls GitLab REST API to search MRs and fetch MR details. Credentials from env vars.
- **SessionService**: CRUD for sessions in SQLite. Orchestrates session lifecycle (create workspace → spawn OpenCode → update state).

### Database (SQLite via SQLx)

- **sessions**: id, title, source_type (jira/gitlab/scratch), source_ref, state (provisioning/running/stopped/terminated), opencode_port, workspace_path, project_id (nullable), created_at, updated_at
- **projects**: id, name, repo_url, bare_clone_path, created_at — populated lazily as sessions use repos

### OpenCode Integration

- One `opencode web --port <port> --hostname 0.0.0.0` process per session, launched in the session's workspace directory.
- Loom allocates ports from a configurable range (e.g., 10000-10999).
- After spawning, Loom calls `POST /session` then `POST /session/:id/prompt_async` to inject the initial task context.
- Git repo setup is handled by OpenCode itself via system prompt instructions (bare clone cache path + worktree commands).

### Frontend (Svelte 5 SPA)

- **Dashboard page**: Lists sessions grouped by state. "New Session" button.
- **New Session page**: Three tabs (Jira, GitLab, Scratch). Smart unified input per tab with autocomplete.
- **Session Detail page**: Metadata, state, actions (open, resume, stop, terminate). Link to OpenCode web UI.
- **Settings page**: Display env var status, base prompt file path, cache directory.
- Uses shadcn-svelte components (Button, Input, Card, Dialog, Badge, Tabs).

### System Prompt Strategy

- User maintains a base prompt file (e.g., `~/.loom/opencode-prompt.md`) listing their repos, preferences, conventions.
- Per-session, Loom appends task-specific context: Jira issue title/description, or MR title/description/branch/repo URL.
- Combined prompt is sent as the first message in the OpenCode session.

## Testing Decisions

- Good tests verify external behavior, not implementation details.
- **SessionService**: Test session lifecycle transitions (provisioning → running → stopped → terminated). Mock OpenCode process spawning.
- **JiraService / GitLabService**: Test API response parsing. Mock HTTP responses.
- **API handlers**: Integration tests hitting the Axum router with a test SQLite database.
- Frontend: Component tests for the smart input (key detection, URL parsing, search triggering).

## Out of Scope

- Multi-user support or authentication for Loom itself
- Automated cleanup of old terminated sessions (manual for now)
- Monitoring or alerting for session health
- Git conflict resolution between sessions
- CI/CD integration
- OpenCode plugin/extension development

## Further Notes

- On startup, Loom should scan for sessions in "running" state and mark them as "stopped" (since their OpenCode processes died with the previous shutdown).
- Port allocation should check for actual port availability, not just database records.
- The bare clone cache directory should be configurable (default: `~/.loom/repos/`).
- Session workspace directory should be configurable (default: `~/.loom/sessions/<session-id>/`).
