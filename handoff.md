# Loom Implementation Handoff

## What was done

### Backend (Rust/Axum)
Fully restructured from flat files into domain modules:

```
backend/src/
  main.rs              -- entry point, config, startup recovery
  config.rs            -- Config from env vars
  db.rs                -- SQLite pool
  error.rs             -- AppError with HTTP status mapping
  state.rs             -- AppState holding all services + config
  models.rs            -- domain models + request/response types + ts-rs exports
  services/
    mod.rs
    session.rs         -- CRUD, lifecycle, port allocation, startup recovery
    opencode.rs        -- process spawn/stop, OpenCode REST API, retries
    jira.rs            -- issue picker search + get issue (API v3)
    gitlab.rs          -- MR search via GitLab API v4
  routes/
    mod.rs             -- combined routes, dev_proxy, static_handler
    health.rs          -- GET /api/health
    sessions.rs        -- CRUD + resume/stop/terminate
    jira.rs            -- GET /api/jira/search, /api/jira/issues/:key
    gitlab.rs          -- GET /api/gitlab/search
```

### Frontend (Svelte 5 SPA)
Full dashboard UI with routing:

```
frontend/src/
  App.svelte           -- router setup (/, /sessions/:id, /settings)
  lib/
    api/
      client.ts        -- fetch wrapper (get/post)
      sessions.ts      -- session API calls
      jira.ts          -- Jira search API
      gitlab.ts        -- GitLab search API
    components/
      Layout.svelte          -- app shell with nav
      SessionCard.svelte     -- clickable session card
      NewSessionDialog.svelte -- tabbed dialog (Jira/GitLab/Scratch)
      SourceSearch.svelte    -- debounced autocomplete
      StateBadge.svelte      -- colored state badge
    pages/
      Dashboard.svelte       -- session list, auto-refresh
      SessionDetail.svelte   -- session detail + actions
      Settings.svelte        -- env var status display
```

### shadcn components installed
card, input, badge, tabs, dialog, separator, button

### Vite config
- `$shared` alias for shared types
- Dev proxy: `/api` -> `http://localhost:3000`

## What needs testing

### Jira integration
- Uses `GET /rest/api/3/issue/picker?query=...&currentJQL=...` for autocomplete
- Uses `GET /rest/api/3/issue/{key}` for direct fetch
- Auth: Basic Auth (email:api_token)
- **Status**: Auth was returning 401. Likely env vars weren't loaded in the test terminal. Need to verify with fresh terminal.

Test command:
```powershell
$cred = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("$($env:JIRA_EMAIL):$($env:JIRA_API_TOKEN)"))
Invoke-RestMethod -Uri "$($env:JIRA_BASE_URL)/rest/api/3/myself" -Headers @{Authorization="Basic $cred"; Accept="application/json"}
```

If that works, search test:
```powershell
Invoke-RestMethod -Uri "$($env:JIRA_BASE_URL)/rest/api/3/issue/picker?query=SAM-398" -Headers @{Authorization="Basic $cred"; Accept="application/json"}
```

### GitLab integration
- Uses `GET /api/v4/merge_requests?search=...&scope=all&in=title`
- Auth: PRIVATE-TOKEN header
- **Status**: `scope=all` on gitlab.com times out (searches all public MRs). Need to scope to group/project.
- **Decision pending**: Switch to URL-only input for GitLab (user pastes MR URL), defer MR discovery.

Test command:
```powershell
Invoke-RestMethod -Uri "$($env:GITLAB_BASE_URL)/api/v4/personal_access_tokens/self" -Headers @{"PRIVATE-TOKEN"=$env:GITLAB_PRIVATE_TOKEN}
```

## Required env vars
```
JIRA_BASE_URL=https://lateamrock.atlassian.net
JIRA_EMAIL=your-email@domain.com
JIRA_API_TOKEN=your-api-token
GITLAB_BASE_URL=https://gitlab.com
GITLAB_PRIVATE_TOKEN=your-token
```

## Running
```powershell
# Backend (terminal 1)
cd backend && cargo run

# Frontend (terminal 2)
cd frontend && pnpm dev
```

Open http://localhost:5173 — API calls proxy to backend on :3000.

## Pending decisions
1. GitLab: switch to URL-only input (paste MR URL) instead of search
2. GitLab: if search is wanted later, add `GITLAB_GROUP_ID` env var for group-scoped search
3. OpenCode model defaults to `github-copilot/gpt-5-mini`
