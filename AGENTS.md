# Repository Guidelines

## Project Structure & Module Organization
- `backend/`: Rust Axum server, state, handlers, migrations.
- `frontend/`: SvelteKit (Bun) app; key libs in `src/lib`, routes under `src/routes`.
- `assets/`, `landing/`: static and landing assets; `docs/`: MkDocs docs.
- Root configs: `docker-compose.yml`, `.env.sample`, Firebase rules, README variants.

## Build, Test, and Development Commands
- Docker stack: `docker compose up --build` (frontend at `:5173`, backend at `:8080`).
- Backend local: `cd backend && cargo run` (needs `DATABASE_URL`, `ADMIN_ID`, `ADMIN_PW`, optional `GEMINI_API_KEY`).
- Frontend local: `cd frontend && bun install && bun run dev` (needs `VITE_API_URL`, optional `VITE_ADMIN_ENCRYPTION_PASSWORD`).
- Backend tests: `cd backend && cargo test`.
- Frontend format/lint (if configured): `cd frontend && bun run check` / `bun run lint` (add scripts if missing).

## Coding Style & Naming Conventions
- Rust: follow `cargo fmt` defaults; snake_case for files/modules; struct/enum PascalCase.
- Svelte/TS: 2-space indent; PascalCase components in `src/lib/components`, kebab-case routes; prefer module-scoped helpers over globals.
- Keep user-facing text i18n-friendly (`svelte-i18n`).
- Env keys: `VITE_*` for frontend build-time, uppercase snake for backend.

## Testing Guidelines
- Backend: write `#[tokio::test]`/`#[test]` in module-adjacent files; prefer integration-style coverage for handlers/state; run `cargo test` before PRs.
- Frontend: add component/page tests if harness present; keep fixtures in `src/lib/__tests__` or colocated.
- Name tests descriptively (behavior-focused), not just “works”.

## Commit & Pull Request Guidelines
- Messages: short imperative summary (e.g., “Add Gemini env wiring”), optional body for context.
- PRs: describe scope and risk, link issues, include screenshots for UI changes, and list test commands run.
- Keep changes minimal per PR; avoid unrelated formatting churn.

## Security & Configuration Tips
- Do not commit real API keys; use `.env` or Docker secrets. Frontend needs `VITE_ADMIN_ENCRYPTION_PASSWORD` baked at build to avoid prompts.
- Backend decrypts Gemini keys with `ADMIN_PW`; ensure frontend uses the same password when encrypting.
- For public exposure, prefer `./run-public.sh --ngrok|--bore` instead of opening raw ports.
