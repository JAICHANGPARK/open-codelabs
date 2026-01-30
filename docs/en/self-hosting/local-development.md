# Local Development Environment

A setup guide for local development.

## Development environment setup

### Install required tools

=== "macOS"
    ```bash
    # Install with Homebrew
    brew install rust bun sqlite

    # Check versions
    rustc --version
    cargo --version
    bun --version
    ```

=== "Linux (Ubuntu/Debian)"
    ```bash
    # Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env

    # Bun
    curl -fsSL https://bun.sh/install | bash

    # SQLite
    sudo apt-get install sqlite3 libsqlite3-dev
    ```

=== "Windows (WSL2)"
    Follow the Linux installation steps inside WSL2 Ubuntu.

### IDE setup

#### VS Code (recommended)

Recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "svelte.svelte-vscode",
    "oven.bun-vscode",
    "tamasfe.even-better-toml",
    "serayuzgur.crates"
  ]
}
```

`.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

#### IntelliJ IDEA / RustRover

Plugins:

- Rust
- Svelte
- Bun
- TOML

## Backend development

### Project structure

```
backend/
|-- Cargo.toml              # dependencies
|-- .env                    # environment variables
|-- src/
|   |-- main.rs            # entry point
|   |-- lib.rs             # shared modules
|   |-- api/               # routing/handlers
|   |   |-- routes.rs
|   |   `-- handlers/
|   |       |-- admin.rs
|   |       |-- ai.rs
|   |       |-- attendees.rs
|   |       |-- audit.rs
|   |       |-- codelabs.rs
|   |       |-- codeserver.rs
|   |       |-- feedback.rs
|   |       |-- materials.rs
|   |       |-- quizzes.rs
|   |       |-- submissions.rs
|   |       |-- upload.rs
|   |       `-- websocket.rs
|   |-- domain/            # domain models/services
|   |-- infrastructure/    # DB/audit logs
|   |-- middleware/        # auth/security
|   `-- utils/             # utilities
|-- migrations/            # database migrations
|   |-- 20251226161500_init.sql
|   |-- 20251226161600_attendees.sql
|   `-- ...
`-- data/                  # SQLite database
    `-- sqlite.db
```

### Run the dev server

```bash
cd backend

# Set env vars
cat > .env << EOF
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
RUST_LOG=backend=debug,tower_http=debug,sqlx=info
EOF

# Create data directory
mkdir -p data

# Check dependencies
cargo check

# Run in dev mode
cargo run

# Or watch mode (auto restart on changes)
cargo install cargo-watch
cargo watch -x run
```

### Code quality tools

#### Clippy (lint)

```bash
# Run Clippy
cargo clippy

# Treat warnings as errors
cargo clippy -- -D warnings

# Auto-fix safe issues
cargo clippy --fix
```

#### Rustfmt (formatting)

```bash
# Check formatting
cargo fmt --check

# Auto-format
cargo fmt
```

`.rustfmt.toml`:

```toml
max_width = 100
hard_tabs = false
tab_spaces = 4
edition = "2021"
```

#### Tests

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Show output
cargo test -- --nocapture

# Run single-threaded
cargo test -- --test-threads=1
```

### Database work

#### Install SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

#### Manage migrations

```bash
# Create a new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run

# Revert migrations
sqlx migrate revert

# Show migration status
sqlx migrate info
```

#### Query validation

Validate SQL queries at compile time:

```bash
# Prepare query metadata
cargo sqlx prepare

# Validate in CI
cargo sqlx prepare --check
```

#### Reset database

```bash
# Remove and recreate the database
rm data/sqlite.db
cargo run

# Or
sqlx database reset
```

### Debugging

#### Adjust log levels

`.env`:

```bash
# Global debug
RUST_LOG=debug

# Per module
RUST_LOG=backend=debug,tower_http=info,sqlx=warn

# Specific handler only
RUST_LOG=backend::api::handlers::codelabs=trace
```

#### LLDB debugger

```bash
# Debug build
cargo build

# Run with LLDB
rust-lldb target/debug/backend

# VS Code launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Backend",
      "cargo": {
        "args": ["build", "--bin=backend"]
      },
      "args": [],
      "cwd": "${workspaceFolder}/backend"
    }
  ]
}
```

## Frontend development

### Project structure

```
frontend/
|-- package.json
|-- bun.lock
|-- vite.config.ts
|-- svelte.config.js
|-- tsconfig.json
|-- src/
|   |-- app.html          # HTML template
|   |-- app.css           # global styles
|   |-- hooks.server.ts   # server hooks
|   |-- lib/              # libraries
|   |   |-- api.ts        # API router
|   |   |-- api-backend.ts
|   |   |-- api-firebase.ts
|   |   |-- api-supabase.ts
|   |   |-- components/   # shared components
|   |   |-- i18n/         # i18n resources
|   |   `-- types.ts      # shared types
|   `-- routes/           # page routes
|       |-- +layout.svelte
|       |-- +page.svelte
|       |-- admin/
|       |   |-- +page.svelte
|       |   |-- [id]/
|       |   `-- audit-logs/
|       |-- codelabs/
|       |   |-- +page.svelte
|       |   `-- [id]/
|       |-- codelabs/[id]/entry/
|       |-- codelabs/[id]/live/
|       |-- certificate/[id]/
|       |-- verify/[id]/
|       `-- login/
`-- static/               # static assets
    `-- favicon.png
```

### Run the dev server

```bash
cd frontend

# Install dependencies
bun install

# Dev server (hot reload)
bun run dev

# Change port
bun run dev --port 3000

# Expose to network (mobile testing)
bun run dev --host
```

### Code quality tools

#### Svelte Check

```bash
# Type check
bun run check

# Watch mode
bun run check:watch
```

#### ESLint and Prettier

```bash
# Install ESLint
bun add -d eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin

# Install Prettier
bun add -d prettier prettier-plugin-svelte

# Run
bunx eslint src
bunx prettier --write src
```

`.prettierrc`:

```json
{
  "useTabs": true,
  "singleQuote": true,
  "trailingComma": "es5",
  "printWidth": 100,
  "plugins": ["prettier-plugin-svelte"],
  "overrides": [
    {
      "files": "*.svelte",
      "options": {
        "parser": "svelte"
      }
    }
  ]
}
```

### Build and preview

```bash
# Production build
bun run build

# Preview build output
bun run preview

# Build analysis
bun run build -- --mode analyze
```

### Environment variables

`.env`:

```bash
# Backend API URL
VITE_API_URL=http://localhost:8080

# Other environment
VITE_API_URL=https://api.example.com
```

Used in code:

```typescript
const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080';
```

## Integrated development workflow

### Multi-terminal with tmux

`dev.sh`:

```bash
#!/bin/bash

# Create tmux session
tmux new-session -d -s codelabs

# Backend window
tmux rename-window -t codelabs:0 'backend'
tmux send-keys -t codelabs:0 'cd backend && cargo watch -x run' C-m

# Frontend window
tmux new-window -t codelabs:1 -n 'frontend'
tmux send-keys -t codelabs:1 'cd frontend && bun run dev' C-m

# Logs window
tmux new-window -t codelabs:2 -n 'logs'

# Attach
tmux attach-session -t codelabs
```

```bash
chmod +x dev.sh
./dev.sh
```

### Automation with Make

`Makefile`:

```makefile
.PHONY: dev backend frontend check test clean

# Start full dev environment
dev:
	@echo "Starting development environment..."
	@make -j2 backend frontend

# Run backend
backend:
	cd backend && cargo watch -x run

# Run frontend
frontend:
	cd frontend && bun run dev

# Code checks
check:
	cd backend && cargo clippy && cargo fmt --check
	cd frontend && bun run check

# Tests
test:
	cd backend && cargo test
	cd frontend && bun test

# Build
build:
	cd backend && cargo build --release
	cd frontend && bun run build

# Clean
clean:
	cd backend && cargo clean
	cd frontend && rm -rf node_modules .svelte-kit build
```

Usage:

```bash
make dev      # start dev environment
make check    # code checks
make test     # run tests
make build    # production build
make clean    # cleanup
```

## Hot reload setup

### Backend auto-reload

Using `cargo-watch`:

```bash
# Install
cargo install cargo-watch

# Restart on file changes
cargo watch -x run

# Watch specific paths
cargo watch -w src -x run

# Build + test + run
cargo watch -x check -x test -x run
```

### Frontend HMR

Vite supports HMR (Hot Module Replacement) automatically.

Customize `vite.config.ts`:

```typescript
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		strictPort: false,
		hmr: {
			overlay: true
		},
		watch: {
			usePolling: true  // Required in Docker
		}
	}
});
```

## Debugging tips

### Backend API tests

#### curl

```bash
# List codelabs
curl http://localhost:8080/api/codelabs

# Create a codelab
curl -X POST http://localhost:8080/api/codelabs \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","description":"Test Desc","author":"Me"}'

# Login
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{"admin_id":"admin","admin_pw":"admin123"}'
```

#### HTTPie (recommended)

```bash
# Install
brew install httpie  # macOS
pip install httpie   # Python

# Use
http GET http://localhost:8080/api/codelabs
http POST http://localhost:8080/api/codelabs title="Test" description="Desc" author="Me"
```

### WebSocket test

Using `websocat`:

```bash
# Install
brew install websocat

# Connect
websocat ws://localhost:8080/api/ws/codelab_id

# Send message (JSON)
{"type":"chat","message":"Hello"}
```

### Browser DevTools

- **Network tab**: inspect API requests and responses
- **Console**: check error logs
- **Application**: view LocalStorage and SessionStorage

## Performance profiling

### Backend profiling

```bash
# Generate flamegraph
cargo install flamegraph
cargo flamegraph

# Output: flamegraph.svg
```

### Frontend bundle analysis

```bash
# Analyze bundle
bun run build
bunx vite-bundle-visualizer
```

## Next steps

- [Public deployment](public-deployment.md) - expose locally via ngrok/bore/cloudflare
- [Environment variables](environment.md) - detailed configuration guide
- [Contribution guide](../contributing/guide.md) - how to contribute
