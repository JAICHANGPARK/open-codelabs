# Installation Guide

This guide covers different ways to install Open Codelabs.

## System Requirements

### Minimum requirements

- **Memory**: 2GB RAM
- **Disk**: 1GB free space
- **OS**: Linux, macOS, Windows (WSL2)

### Software requirements

=== "Docker (recommended)"
    - Docker Engine 20.10+
    - Docker Compose v2.0+

=== "Local development"
    - Rust 1.75+
    - Bun 1.0+
    - SQLite 3.35+

=== "Podman"
    - Podman 4.0+
    - podman-compose 1.0+

## Install the CLI (`oc`)

The `oc` CLI can manage an existing Open Codelabs server and also start a local stack for you.

The easiest first-run entry point is the interactive onboarding flow:

```bash
oc init
```

`oc init` guides you through starting a local stack or connecting to an existing server, then offers to save a profile and launch `oc auth login`.

### Install from source

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs
cargo install --path backend --bin oc
```

Verify the install:

```bash
oc --help
```

### Build a local release binary

```bash
cd backend
cargo build --release --bin oc
./target/release/oc --help
```

### Start a local stack with the CLI only

```bash
oc run --open
```

`oc run` will:

- Detect `docker` or `podman` automatically.
- Print install/start guidance when the container engine is missing or not running.
- Write `~/.open-codelabs/runtime/local-stack/compose.yml`.
- Start the published frontend/backend images with SQLite by default.
- Start a guided setup flow when you run it without flags in an interactive terminal.

Common options:

```bash
# Pull the latest images first
oc run --pull

# Start with the bundled PostgreSQL container
oc run --postgres

# Force a specific engine
oc run --engine docker
oc run --engine podman
```

Lifecycle commands after startup:

```bash
oc ps
oc logs --service backend --tail 200 --no-follow
oc restart --service frontend
oc down
oc down --volumes
```

### First connection after the stack is up

```bash
oc connect add --name local --url http://localhost:8080 --runtime backend --activate
oc auth login
oc codelab list
```

Or use:

```bash
oc connect add --interactive
```

to fill in the profile values through prompts.

Sessions are stored under `~/.open-codelabs/` by default.

See the [CLI Reference](../user-guide/cli.md) for the full supported command set and current usage.

## Remove and clean up the CLI (`oc`)

When you remove the CLI, check both how it was installed and whether you used the local runtime.

### 1. Stop the local stack and remove runtime data

```bash
oc down
oc down --volumes
```

- `oc down` stops the containers and tears down the compose runtime.
- `oc down --volumes` also removes the local bind-mounted data created by `oc run`.

### 2. Remove a Cargo-installed CLI

```bash
cargo uninstall oc
```

### 3. Remove a manually copied binary

- If you built it locally with `cargo build --release --bin oc`, delete `backend/target/release/oc`.
- If you installed a downloaded release binary, remove the `oc` binary you copied into your `PATH`.

### 4. Clean up config, session, and runtime files

```bash
rm -rf ~/.open-codelabs
```

- `~/.open-codelabs/` stores profiles, sessions, browser auth state, and local runtime metadata.
- On Windows, clean up the `.open-codelabs` directory under your user profile.

### 5. Verify removal

```bash
which oc
oc --help
```

- If `which oc` prints no path, the binary has been removed from your current shell lookup.
- If your shell cached the old path, open a new terminal or run `hash -r`.

## Install with Docker

Docker is the simplest and recommended approach.

### 1. Install Docker

=== "Linux"
    ```bash
    # Ubuntu/Debian
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh

    # Install Docker Compose
    sudo apt-get install docker-compose-plugin
    ```

=== "macOS"
    Download and install [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop).

=== "Windows"
    Download and install [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop).

    WSL2 is recommended.

    Docker Desktop includes Docker Compose. On Linux, you may need `docker-compose-plugin`.
    Use the `docker compose` command for modern installations (V1's `docker-compose` is deprecated).

### 2. Clone the project

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs
```

### 3. Configure environment variables (optional)

You can run with the defaults, but if you want to change admin credentials:

```bash
# Edit docker-compose.yml
nano docker-compose.yml
```

```yaml
services:
  backend:
    environment:
      - DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
      - ADMIN_ID=your_admin_id        # change
      - ADMIN_PW=your_secure_password # change
```

### 4. Run

```bash
docker compose up -d
```

- `-d`: run in the background

### 5. Check logs

```bash
# All services
docker compose logs -f

# Backend only
docker compose logs -f backend

# Frontend only
docker compose logs -f frontend
```

### 6. Stop and start

```bash
# Stop
docker compose stop

# Start
docker compose start

# Stop and remove containers
docker compose down

# Remove volumes (data loss)
docker compose down -v
```

## Install with Podman

If you prefer Podman:

### 1. Install Podman

=== "Fedora/RHEL"
    ```bash
    sudo dnf install podman podman-compose
    ```

=== "Ubuntu"
    ```bash
    sudo apt-get install podman podman-compose
    ```

=== "macOS"
    ```bash
    brew install podman podman-compose
    podman machine init
    podman machine start
    ```

### 2. Run

```bash
podman-compose up -d
```

Use `podman-compose` instead of `docker compose` for the remaining commands.

## Local development setup

Detailed instructions for local development.

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify
rustc --version
cargo --version
```

### 2. Install Bun

```bash
curl -fsSL https://bun.sh/install | bash

# Verify
bun --version
```

### 3. Install SQLite

=== "Linux"
    ```bash
    # Ubuntu/Debian
    sudo apt-get install sqlite3 libsqlite3-dev

    # Fedora
    sudo dnf install sqlite sqlite-devel
    ```

=== "macOS"
    ```bash
    brew install sqlite
    ```

=== "Windows"
    SQLite is bundled with most Windows installations.

### 4. Clone and configure

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs
```

### 5. Backend setup

```bash
cd backend

# Create .env
cat > .env << EOF
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
RUST_LOG=backend=debug,tower_http=debug
EOF

# Create data directory
mkdir -p data

# Optional: dependency check
cargo check

# Run
cargo run
```

The backend runs at `http://localhost:8080`.

### 6. Frontend setup

In a new terminal:

```bash
cd frontend

# Install dependencies
bun install

# Optional env vars
cat > .env << EOF
VITE_API_URL=http://localhost:8080
EOF

# Run dev server
bun run dev
```

The frontend runs at `http://localhost:5173`.

## Production build

### Docker production image

```bash
# Build production images
docker compose -f docker-compose.prod.yml build

# Run
docker compose -f docker-compose.prod.yml up -d
```

### Local production build

#### Backend

```bash
cd backend

# Release build
cargo build --release

# Run
./target/release/backend
```

#### Frontend

```bash
cd frontend

# Production build
bun run build

# Preview
bun run preview
```

## Install SQLx CLI (for developers)

To manage database migrations:

```bash
cargo install sqlx-cli --no-default-features --features sqlite

# Run migrations
cd backend
sqlx migrate run

# Revert a migration
sqlx migrate revert

# Create a new migration
sqlx migrate add <migration_name>
```

## Database reset

To reset the database to a clean state:

```bash
# Docker
docker compose down -v
rm -rf backend/data/sqlite.db
docker compose up -d

# Local development
rm -rf backend/data/sqlite.db
cargo run
```

## Full environment variable list

### Backend

| Variable | Description | Default | Required |
|------|------|--------|------|
| `DATABASE_URL` | SQLite database path | `sqlite:data/sqlite.db?mode=rwc` | Yes |
| `ADMIN_ID` | Admin ID | `admin` | Yes |
| `ADMIN_PW` | Admin password | `admin123` | Yes |
| `RUST_LOG` | Log level | `backend=debug,tower_http=debug` | No |

### Frontend

| Variable | Description | Default | Required |
|------|------|--------|------|
| `VITE_API_URL` | Backend API URL | `http://localhost:8080` | No |
| `PORT` | Frontend port | `5173` | No |
| `HOST` | Bind host | `0.0.0.0` | No |

## Verification

Confirm the installation:

### 1. Health check

```bash
# Backend health check
curl http://localhost:8080/api/codelabs

# Example response
[]
```

### 2. Open the frontend

Visit [http://localhost:5173](http://localhost:5173).

### 3. Admin login

1. Go to [http://localhost:5173/login](http://localhost:5173/login)
2. Log in with your credentials

## Next steps

Installation is complete. Continue with:

- [Create Your First Codelab](first-codelab.md)
- [Public Deployment Guide](../self-hosting/public-deployment.md)
- [API Reference](../specification/api-reference.md)

## Troubleshooting

### Port conflicts

If another process is using the ports:

```bash
# Check ports (Linux/macOS)
lsof -i :8080
lsof -i :5173

# Kill the process
kill -9 <PID>
```

### Database migration errors

```bash
cd backend
rm -rf data/sqlite.db
cargo run
```

### Docker build errors

```bash
# Rebuild without cache
docker compose build --no-cache

# Cleanup Docker system
docker system prune -a
```

For more troubleshooting, see the [FAQ](../faq.md).
