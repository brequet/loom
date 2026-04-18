# PRD-0001: Project Scaffolding

## Problem Statement

Loom needs a working project skeleton before any features can be built. The Rust backend, Svelte frontend, shared type generation pipeline, and dev tooling must all be set up, connected, and runnable with a single command.

## Solution

Scaffold a monorepo with a Rust/Axum backend and a Svelte 5/Vite SPA frontend. The backend serves the frontend assets in production (via rust-embed) and proxies to the Vite dev server in development. SQLite database is initialized via SQLx migrations. ts-rs generates TypeScript types from Rust structs into a shared directory the frontend imports. shadcn-svelte is installed for UI components.

## User Stories

1. As a developer, I want to run a single command to start both backend and frontend in dev mode, so that I can iterate quickly.
2. As a developer, I want the Rust backend to compile and start with `cargo run`, so that I can verify the backend works.
3. As a developer, I want the Svelte frontend to start with `pnpm dev`, so that I can verify the frontend works.
4. As a developer, I want the frontend dev server to be proxied through the backend, so that I access everything on one port.
5. As a developer, I want SQLite set up with SQLx and an initial migration, so that I can start defining tables.
6. As a developer, I want ts-rs configured so that `cargo test` generates TypeScript types into `shared/`, so that the frontend can import them.
7. As a developer, I want shadcn-svelte installed with at least one component (e.g., Button), so that I can verify the component pipeline works.
8. As a developer, I want `cargo build --release` to embed the frontend's built assets, so that the production binary is self-contained.
9. As a developer, I want oxlint configured for the frontend, so that linting is fast.
10. As a developer, I want a basic health-check API endpoint (`GET /api/health`), so that I can verify the backend is responding.
11. As a developer, I want the project to have a `.gitignore` covering Rust, Node, and SQLite artifacts.

## Implementation Decisions

### Modules

- **backend/** — Cargo project with Axum. Modules: `main.rs` (startup, server config), `routes/` (API route handlers), `db/` (SQLx pool setup), `state.rs` (AppState for DI via Axum's State extractor).
- **frontend/** — Vite + Svelte 5 SPA. Standard Vite scaffold with Tailwind CSS v4 and shadcn-svelte.
- **shared/** — Output directory for ts-rs generated `.ts` files. Not a package itself, imported by frontend via relative path or TypeScript path alias.

### Technical decisions

- **Axum State extractor** for dependency injection — AppState holds the SQLx pool (and later, service handles).
- **Dev mode**: Backend detects `DEV` env var and reverse-proxies non-API routes to `http://localhost:5173` (Vite).
- **Prod mode**: Backend serves embedded assets from `frontend/dist/` via rust-embed for any non-API route.
- **SQLx offline mode**: We'll use `sqlx prepare` so the project compiles without a live database in CI.
- **pnpm** as frontend package manager.
- **Oxlint** for frontend linting (fast, drop-in ESLint replacement). Oxfmt if available.
- **Tailwind CSS v4** (required by latest shadcn-svelte).

### What this PRD does NOT cover

- No real features (sessions, Jira integration, GitLab integration, git worktree management).
- No real database schema beyond the initial empty migration.
- No authentication or authorization.
- No OpenCode process management.

## Testing Decisions

- At this stage, testing is minimal — just verify the stack compiles and connects:
  - `cargo build` succeeds
  - `cargo test` succeeds (and generates ts-rs types)
  - `pnpm build` succeeds in frontend/
  - The health endpoint returns 200
- Deeper testing strategy will be defined in the feature PRDs.

## Out of Scope

- Session management, lifecycle, or any business logic
- Jira or GitLab API integration
- Git worktree management
- OpenCode process spawning
- Dashboard UI beyond a placeholder page
- User configuration or settings

## Further Notes

This scaffolding must be completed before the next domain-model session, which will define the actual app behavior: pages, features, session management, integration workflows, dependency injection patterns, and OpenCode process orchestration.
