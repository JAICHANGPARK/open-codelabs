# Environment Variables

A detailed guide to all Open Codelabs environment variables.

!!! info "Where to put env files"
    - Docker Compose: repo root `.env` (referenced by `docker-compose.yml`)
    - Local dev: `backend/.env`, `frontend/.env`

## Backend variables

### Required variables

#### DATABASE_URL

Database connection string (SQLite, PostgreSQL, MySQL supported).

**SQLite (default)**:
```bash
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
```

**PostgreSQL**:
```bash
DATABASE_URL=postgres://user:password@localhost:5432/dbname
```

**MySQL**:
```bash
DATABASE_URL=mysql://user:password@localhost:3306/dbname
```

**Format**: `sqlite:<path>?<options>` or `<db_type>://<user>:<password>@<host>:<port>/<dbname>`

**Options**:
- `mode=rwc`: read, write, create
- `mode=ro`: read-only
- `mode=memory`: in-memory database

**Examples**:

```bash
# Local development
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc

# Docker
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc

# In-memory (testing)
DATABASE_URL=sqlite::memory:
```

#### ADMIN_ID

Admin login ID.

```bash
ADMIN_ID=admin
```

!!! warning "Security"
    Do not use defaults like `admin` in production.

**Recommendations**:
- 8+ characters
- hard to guess
- email format is allowed

```bash
ADMIN_ID=facilitator_2024
ADMIN_ID=admin@example.com
```

#### ADMIN_PW

Admin password.

```bash
ADMIN_PW=admin123
```

!!! danger "Required"
    Change to a strong password in production.

**Recommendations**:
- at least 20 characters
- mix of upper/lowercase, numbers, symbols
- use a generator

```bash
# Generate a strong password
openssl rand -base64 32
# Output: 8vYR3jkLm9nP2qTxWz6CbF4hK7dN5sVuG1aE0iJ3XyO=

ADMIN_PW=8vYR3jkLm9nP2qTxWz6CbF4hK7dN5sVuG1aE0iJ3XyO=
```

### Optional variables

#### AUTH_SECRETS

JWT signing secret list (comma-separated). The first is active; others are kept for verification to allow key rotation. If empty, `ADMIN_PW` is used as fallback.

```bash
AUTH_SECRETS=primary_secret,previous_secret
```

!!! note "Legacy"
    `AUTH_SECRET` is still read, but new configs should use `AUTH_SECRETS`.

#### AUTH_ISSUER

JWT issuer claim.

```bash
AUTH_ISSUER=open-codelabs
```

Default: `open-codelabs`

#### AUTH_AUDIENCE

JWT audience claim.

```bash
AUTH_AUDIENCE=open-codelabs
```

Default: `open-codelabs`

#### ADMIN_SESSION_TTL_SECONDS

Admin session TTL in seconds.

```bash
ADMIN_SESSION_TTL_SECONDS=28800
```

Default: `28800` (8 hours)

#### ATTENDEE_SESSION_TTL_SECONDS

Attendee session TTL in seconds.

```bash
ATTENDEE_SESSION_TTL_SECONDS=43200
```

Default: `43200` (12 hours)

#### COOKIE_SECURE

Set to `true` on HTTPS (Secure cookie + `__Host-` prefix).

```bash
COOKIE_SECURE=true
```

Default: `false`

#### COOKIE_SAMESITE

Cookie SameSite policy: `lax` (default), `strict`, `none`.

```bash
COOKIE_SAMESITE=lax
```

!!! warning "Note"
    Using `COOKIE_SAMESITE=none` requires `COOKIE_SECURE=true`.

#### TRUST_PROXY

Set to `true` when running behind a reverse proxy and trusting `X-Forwarded-*` headers.

```bash
TRUST_PROXY=true
```

Default: `false`

#### CORS_ALLOWED_ORIGINS

Allowed origins (comma-separated). If empty, local defaults are allowed.

```bash
CORS_ALLOWED_ORIGINS=http://localhost:5173,http://127.0.0.1:5173
```

#### RATE_LIMIT_GENERAL_PER_MINUTE

General API requests per minute per IP.

```bash
RATE_LIMIT_GENERAL_PER_MINUTE=120
```

Default: `120`

#### RATE_LIMIT_LOGIN_PER_5_MIN

Login requests per 5 minutes per IP.

```bash
RATE_LIMIT_LOGIN_PER_5_MIN=20
```

Default: `20`

#### RATE_LIMIT_AI_PER_MINUTE

AI proxy requests per minute per IP.

```bash
RATE_LIMIT_AI_PER_MINUTE=30
```

Default: `30`

#### RATE_LIMIT_UPLOAD_PER_MINUTE

Upload and submission POST requests per minute per IP.

```bash
RATE_LIMIT_UPLOAD_PER_MINUTE=20
```

Default: `20`

#### CSP_HEADER

Override the Content-Security-Policy header for UI responses. Uses default if empty.

```bash
CSP_HEADER=default-src 'self'; img-src 'self' data: blob:
```

#### HSTS_HEADER

Override the Strict-Transport-Security header. Applied only on HTTPS.

```bash
HSTS_HEADER=max-age=63072000; includeSubDomains; preload
```

#### ALLOWED_GEMINI_MODELS

Allowed Gemini model IDs (comma-separated). Requests not in the list are rejected.

```bash
ALLOWED_GEMINI_MODELS=gemini-3-flash-preview,gemini-1.5-pro
```

#### GEMINI_API_KEY

Default Gemini API key when no admin key is stored.

```bash
GEMINI_API_KEY=your_gemini_api_key_here
```

#### RUST_LOG

Log level settings.

```bash
RUST_LOG=backend=debug,tower_http=debug
```

**Levels**: `error`, `warn`, `info`, `debug`, `trace`

**Per-module settings**:

```bash
# Global debug
RUST_LOG=debug

# Per module
RUST_LOG=backend=debug,sqlx=info,tower_http=warn

# Specific handler only
RUST_LOG=backend::handlers::codelabs=trace

# Production
RUST_LOG=backend=info,tower_http=info
```

#### PORT (backend)

Backend API server port.

```bash
# Default: 8080
PORT=8080

# Custom port
PORT=3000
```

To use in code, update `main.rs`:

```rust
let port = std::env::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .expect("PORT must be a valid number");

let addr = SocketAddr::from(([0, 0, 0, 0], port));
```

## Frontend variables

### VITE_API_URL

Backend API URL.

```bash
VITE_API_URL=http://localhost:8080
```

**Common environments**:

```bash
# Local development
VITE_API_URL=http://localhost:8080

# Docker Compose (internal network)
VITE_API_URL=http://backend:8080

# Production
VITE_API_URL=https://api.example.com

# ngrok
VITE_API_URL=https://abc123.ngrok-free.app

# Cloudflare Tunnel
VITE_API_URL=https://abc123.trycloudflare.com
```

!!! info "Vite env vars"
    - Only variables starting with `VITE_` are exposed to the client
    - Injected at build time
    - Do not include secrets

### VITE_USE_SUPABASE

Enable Supabase serverless mode in the frontend.

```bash
VITE_USE_SUPABASE=true
```

### VITE_SUPABASE_URL

Supabase project URL.

```bash
VITE_SUPABASE_URL=https://your-project.supabase.co
```

### VITE_SUPABASE_ANON_KEY

Supabase public anon key (safe for client).

```bash
VITE_SUPABASE_ANON_KEY=your_anon_key
```

### VITE_SUPABASE_STORAGE_BUCKET

Storage bucket name for uploads (images/materials/submissions).

```bash
VITE_SUPABASE_STORAGE_BUCKET=open-codelabs
```

### VITE_ADMIN_ID / VITE_ADMIN_PW

Optional admin fallback for serverless mode (Supabase/Firebase).

```bash
VITE_ADMIN_ID=admin
VITE_ADMIN_PW=admin
```

### VITE_ADMIN_ENCRYPTION_PASSWORD

Password used to encrypt the Gemini API key in the browser. It must match backend `ADMIN_PW` for decryption. If empty, the UI prompts for it.

```bash
VITE_ADMIN_ENCRYPTION_PASSWORD=your_admin_pw
```

### FRONTEND_PORT

Frontend container port (host mapping) in Docker Compose, passed to `PORT`.

```bash
FRONTEND_PORT=5173
```

### FRONTEND_HOST

Frontend bind host in Docker Compose, passed to `HOST`.

```bash
FRONTEND_HOST=0.0.0.0
```

### PORT

Frontend server port.

```bash
PORT=5173  # default
```

In Docker Compose, `FRONTEND_PORT` is passed to `PORT`.

### HOST

Bind host.

```bash
HOST=0.0.0.0  # all interfaces
HOST=127.0.0.1  # localhost only
```

In Docker Compose, `FRONTEND_HOST` is passed to `HOST`.

## Environment-specific configs

### Local development

`backend/.env`:

```bash
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
AUTH_SECRETS=change_me_primary,change_me_old
GEMINI_API_KEY=your_gemini_api_key_here
RUST_LOG=backend=debug,tower_http=debug,sqlx=info
```

`frontend/.env`:

```bash
VITE_API_URL=http://localhost:8080
VITE_ADMIN_ENCRYPTION_PASSWORD=admin123
```

### Docker Compose

The repo root `.env` is injected into `docker-compose.yml`.

`.env` (repo root):

```bash
DATA_VOLUME_PATH=~/open-codelabs
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=YourSecurePassword123!
AUTH_SECRETS=change_me_primary,change_me_old
VITE_API_URL=http://localhost:8080
VITE_ADMIN_ENCRYPTION_PASSWORD=YourSecurePassword123!
FRONTEND_PORT=5173
FRONTEND_HOST=0.0.0.0
```

`docker-compose.yml` (excerpt):

```yaml
services:
  backend:
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - ADMIN_ID=${ADMIN_ID}
      - ADMIN_PW=${ADMIN_PW}
    volumes:
      - ${DATA_VOLUME_PATH}/data:/app/data
      - ${DATA_VOLUME_PATH}/uploads:/app/static/uploads

  frontend:
    environment:
      - VITE_API_URL=${VITE_API_URL}
      - VITE_ADMIN_ENCRYPTION_PASSWORD=${ADMIN_PW}
      - PORT=${FRONTEND_PORT}
      - HOST=${FRONTEND_HOST}
```

The default `docker-compose.yml` passes `ADMIN_PW` into `VITE_ADMIN_ENCRYPTION_PASSWORD`. If you want a different value, update `frontend.environment`.

If needed, add backend variables (AUTH_*, COOKIE_*, CORS_*, RATE_LIMIT_*, CSP_HEADER, HSTS_HEADER, ALLOWED_GEMINI_MODELS, etc.) to `backend.environment`.

You can also add `env_file` entries to inject `backend/.env` and `frontend/.env` directly:

```yaml
services:
  backend:
    env_file:
      - backend/.env

  frontend:
    env_file:
      - frontend/.env
```

### Production

`backend/.env.production`:

```bash
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
ADMIN_ID=${ADMIN_ID}  # injected externally
ADMIN_PW=${ADMIN_PW}  # injected externally
AUTH_SECRETS=${AUTH_SECRETS}
COOKIE_SECURE=true
TRUST_PROXY=true
RUST_LOG=backend=info,tower_http=warn
```

`frontend/.env.production`:

```bash
VITE_API_URL=https://api.yourdomain.com
VITE_ADMIN_ENCRYPTION_PASSWORD=${ADMIN_PW}
```

**Secret management**:

```bash
# .env.local (do not commit)
ADMIN_ID=your_real_admin_id
ADMIN_PW=your_real_secure_password

# Inject via env
export ADMIN_ID="your_real_admin_id"
export ADMIN_PW="your_real_secure_password"
docker-compose up
```

## Environment validation

### Validate on backend startup

Add to `main.rs`:

```rust
fn validate_env() -> anyhow::Result<()> {
    let required = vec!["DATABASE_URL", "ADMIN_ID", "ADMIN_PW"];

    for var in required {
        std::env::var(var)
            .map_err(|_| anyhow::anyhow!("{} must be set", var))?;
    }

    // Check password strength
    let pw = std::env::var("ADMIN_PW")?;
    if pw.len() < 12 {
        tracing::warn!("ADMIN_PW is too short! Use at least 12 characters.");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    validate_env()?;

    // ...
}
```

### Validate with a script

`check-env.sh`:

```bash
#!/bin/bash

required_backend=("DATABASE_URL" "ADMIN_ID" "ADMIN_PW")
required_frontend=("VITE_API_URL")

echo "Checking backend environment..."
for var in "${required_backend[@]}"; do
    if [ -z "${!var}" ]; then
        echo "$var is not set"
        exit 1
    else
        echo "$var is set"
    fi
done

echo "Checking frontend environment..."
for var in "${required_frontend[@]}"; do
    if [ -z "${!var}" ]; then
        echo "$var is not set"
        exit 1
    else
        echo "$var is set"
    fi
done

echo "All required environment variables are set"
```

```bash
chmod +x check-env.sh
source backend/.env && source frontend/.env && ./check-env.sh
```

## Security best practices

### 1. Protect .env files

`.gitignore`:

```gitignore
# Environment variables
.env
.env.local
.env.production
.env.*.local
backend/.env
frontend/.env

# Databases
*.db
*.db-*
```

### 2. Provide example files

`.env.example`:

```bash
# Backend Configuration
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=your_admin_id_here
ADMIN_PW=your_secure_password_here
RUST_LOG=backend=info,tower_http=info

# Instructions:
# 1. Copy this file to .env
# 2. Replace placeholder values
# 3. Never commit .env to version control
```

Usage:

```bash
cp .env.example .env
nano .env  # edit values
```

### 3. Manage secrets in CI/CD

#### GitHub Actions

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Deploy with secrets
        env:
          ADMIN_ID: ${{ secrets.ADMIN_ID }}
          ADMIN_PW: ${{ secrets.ADMIN_PW }}
        run: |
          echo "DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc" > backend/.env
          echo "ADMIN_ID=$ADMIN_ID" >> backend/.env
          echo "ADMIN_PW=$ADMIN_PW" >> backend/.env
          docker-compose up -d
```

Set these in Repository Settings -> Secrets.

### 4. Production password policy

```bash
# Minimum requirement validation
validate_password() {
    local pw=$1
    local len=${#pw}

    if [ $len -lt 20 ]; then
        echo "Password too short (minimum 20 characters)"
        return 1
    fi

    if ! [[ "$pw" =~ [A-Z] ]]; then
        echo "Password must contain uppercase letters"
        return 1
    fi

    if ! [[ "$pw" =~ [a-z] ]]; then
        echo "Password must contain lowercase letters"
        return 1
    fi

    if ! [[ "$pw" =~ [0-9] ]]; then
        echo "Password must contain numbers"
        return 1
    fi

    echo "Password meets requirements"
    return 0
}

validate_password "$ADMIN_PW"
```

## Environment debugging

### Check current settings

```bash
# Backend
cd backend
cargo run --bin print-env

# Or directly
env | grep -E '(DATABASE_URL|ADMIN_ID|RUST_LOG)'

# Frontend
cd frontend
bun run dev --mode development
```

### Log environment variables

Backend `main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing::info!("DATABASE_URL: {}", std::env::var("DATABASE_URL")?);
    tracing::info!("ADMIN_ID: {}", std::env::var("ADMIN_ID")?);
    tracing::info!("ADMIN_PW: ********");  // do not log passwords
    tracing::info!("RUST_LOG: {:?}", std::env::var("RUST_LOG").ok());

    // ...
}
```

!!! warning "Note"
    Do not print sensitive values (passwords, API keys) in logs.

## Next steps

- [Docker deployment](docker.md) - production deployment with Docker
- [Security guide](security.md) - operational security checklist
- [FAQ](../faq.md) - common questions
