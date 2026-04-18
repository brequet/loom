# Tech stack: Axum + Svelte SPA + SQLx + ts-rs

Loom is a local web app (single user, runs on a Hyper-V VM). We chose:

- **Axum** for the Rust backend — lightweight, composable, natural DI via State extractor. Loco.rs was considered but rejected: too opinionated, imposes SeaORM, fights clean architecture goals.
- **Vite + Svelte 5 (SPA)** for the frontend — pure client-side app, no SSR needed. SvelteKit rejected: its server-side features conflict with the Rust backend owning all server logic. Frontend assets are embedded in the Rust binary for production.
- **SQLx + SQLite** for persistence — compile-time checked raw SQL. SeaORM/Diesel rejected: ORM overhead not justified for a simple domain.
- **ts-rs** for shared types — Rust structs derive TypeScript definitions. Single source of truth, no schema files to maintain.
- **Git bare clone + worktrees** for session isolation — one cached bare clone per Project, instant worktree per Session. Full-clone-per-session rejected: slow, disk-heavy.
