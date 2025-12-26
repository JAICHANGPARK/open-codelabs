---
trigger: always_on
---

# Project Rules: Open-Codelabs: Hands-on System

## Technology Stack
- **Frontend**: SvelteKit (using Bun as runtime/package manager)
- **Backend**: Rust (Axum or Actix-web)
- **Database**: SQLite (via SQLx)
- **Containerization**: Docker, Docker Compose
- **Styling**: Tailwind CSS (Google Codelab look & feel)

## General Principles
1. **SaaS Architecture**: The system should support multiple codelabs. Local execution must be exposed via ngrok easily.
2. **Role-Based Access**:
    - `Facilitator`: Can create/edit codelabs, view participant status, and manage steps.
    - `Attendee`: Can view codelabs, follow steps, and submit progress.
3. **Data Integrity**: Use SQLite for lightweight, local-first storage that can be easily backed up as a single file.
4. **Content Format**: Codelab content should be stored/rendered in Markdown to follow the "Google Codelab" (Antigravity) style.

## Backend (Rust) Guidelines
- Use `sqlx` for asynchronous SQLite interaction.
- Implementation of RESTful API for:
    - CRUD for Codelabs and Steps.
    - Session management for attendees.
- Error handling should use `anyhow` or `thiserror`.

## Frontend (SvelteKit) Guidelines
- Use `Bun` for all scripts and dependency management.
- Implement responsive design mimicking `claat` (Google's codelab tool).
- Components:
    - `Sidebar`: Navigation between steps.
    - `MainContent`: Markdown renderer.
    - `Controls`: "Next", "Back", "Finish" buttons.
- Use QR code library to generate access links for attendees.

## Deployment Rules
- `Dockerfile` must use multi-stage builds to keep images small.
- `docker-compose.yml` should include:
    - `backend` (Rust)
    - `frontend` (SvelteKit)
    - `db` (Volume for SQLite)
    - `ngrok` (Optional sidecar for easy tunneling)