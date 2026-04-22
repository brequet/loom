set shell := ["powershell.exe", "-c"]

# Show available recipes
default:
    @just --list

# Generate TypeScript types from Rust models into shared/
gen-dto:
    cargo test --manifest-path backend/Cargo.toml

# Build backend (release) - embeds frontend/dist, run build-frontend first
build-backend:
    cargo build --manifest-path backend/Cargo.toml --release

# Run backend in dev mode
dev-backend:
    cargo run --manifest-path backend/Cargo.toml

# Run backend tests
test-backend:
    cargo test --manifest-path backend/Cargo.toml

# Check backend (fast, no codegen)
check-backend:
    cargo check --manifest-path backend/Cargo.toml

# Install frontend deps
install-frontend:
    pnpm --dir frontend install

# Build frontend (production) - output goes to frontend/dist/
build-frontend: install-frontend
    pnpm --dir frontend run build

# Run frontend dev server
dev-frontend: install-frontend
    pnpm --dir frontend run dev

# Type-check frontend
check-frontend: install-frontend
    pnpm --dir frontend run check

# Regenerate DTOs then type-check frontend (use after changing Rust models)
sync: gen-dto check-frontend

# Build everything in correct order: DTOs -> frontend -> backend (embeds frontend/dist)
build: gen-dto build-frontend build-backend

# Full check: backend + frontend
check: check-backend check-frontend
