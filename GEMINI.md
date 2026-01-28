# Open Codelabs Project Context

## Project Overview
**Open Codelabs** is a full-stack, open-source platform for creating and hosting hands-on coding workshops (Codelabs), inspired by Google Codelabs. It features a dual-role system (Facilitator vs. Attendee), AI-assisted content generation, and flexible deployment options (Self-hosted via Rust/SQLite or Serverless via Firebase).

## Tech Stack & Architecture

### Frontend (`/frontend`)
*   **Framework:** SvelteKit 5 (Vite + TypeScript)
*   **Runtime:** Bun
*   **Styling:** Tailwind CSS 4.0
*   **State Management:** Svelte Runes
*   **Communication:** HTTP REST + WebSocket
*   **Internationalization:** `svelte-i18n`

### Backend (`/backend`)
*   **Language:** Rust (Edition 2021)
*   **Framework:** Axum (Tokio stack)
*   **Database:** SQLite (managed via SQLx)
*   **Key Libraries:** `sqlx`, `tokio`, `serde`, `tower-http`, `tracing`

### Infrastructure
*   **Containerization:** Docker & Docker Compose
*   **Cloud (Optional):** Firebase (Hosting, Firestore, Storage)
*   **Documentation:** MkDocs

## Key Commands

### Docker (Recommended for Running)
```bash
# Start the full stack (Frontend + Backend + DB)
docker compose up --build
```
*   Frontend: `http://localhost:5173`
*   Backend: `http://localhost:8080`

### Local Development

#### Backend
```bash
cd backend
# Setup environment variables
echo "DATABASE_URL=sqlite:data/sqlite.db?mode=rwc" > .env
# Run the server
cargo run
```

#### Frontend
```bash
cd frontend
# Install dependencies
bun install
# Setup environment variables
echo "VITE_API_URL=http://localhost:8080" > .env
# Run dev server
bun run dev
```

### Public Exposure
To expose the local server for a workshop:
```bash
./run-public.sh --ngrok
# OR
./run-public.sh --bore
# OR
./run-public.sh --cloudflare
```

## Project Structure
*   `backend/`: Rust source code, migrations, and Cargo configuration.
*   `frontend/`: SvelteKit application, components, and static assets.
*   `docs/`: Project documentation source files (MkDocs).
*   `run-public.sh`: Script to tunnel localhost to the public internet.
*   `docker-compose.yml`: Orchestration for local deployment.

## Development Conventions
*   **Code Style:** Follows standard Rust (`cargo fmt`) and TypeScript/Svelte conventions (`prettier`).
*   **Database:** Database schema changes are handled via `sqlx` migrations (`backend/migrations`).
*   **I18n:** All user-facing text should be internationalized using `svelte-i18n`.
*   **Architecture:**
    *   **Frontend:** Reactive UI with Svelte Runes, communicating with backend via REST and WebSocket.
    *   **Backend:** RESTful API with specific handlers for admin/attendee logic.
