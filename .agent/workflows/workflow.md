---
description: Building AntiGravity Hands-on System
---

# Workflow: Building AntiGravity Hands-on System

## Phase 1: Project Scaffolding
1. **Backend Initialization**:
    - Create a Rust project using Cargo.
    - Set up `Axum`, `SQLx`, `Tokio`.
    - Create SQLite schema: `codelabs`, `steps`, `attendees`, `progress`.
2. **Frontend Initialization**:
    - Create SvelteKit project using `bun create svelte@latest`.
    - Install Tailwind CSS and Typography plugin for Markdown styling.
3. **Docker Setup**:
    - Create `Dockerfile` for both services.
    - Orchestrate with `docker-compose.yml`.

## Phase 2: Database & Backend API
1. **Models**: Define Rust structs for Codelab (ID, Title, Description, Author).
2. **CRUD API**:
    - `GET /api/codelabs`: List all codelabs.
    - `GET /api/codelabs/:id`: Get full steps.
    - `POST /api/codelabs`: Create new content (Facilitator only).
3. **Static File Serving**: Serve the compiled SvelteKit frontend via Rust or separately in Docker.

## Phase 3: Facilitator UI (Admin)
1. **Codelab Editor**:
    - Interface to add/remove steps.
    - Markdown editor with live preview.
2. **Dashboard**:
    - List of active codelabs.
    - Display QR Code for each codelab (using `svelte-qrcode` or similar).

## Phase 4: Attendee UI (Client)
1. **Step-by-Step Viewer**:
    - Implement the "Google Antigravity" layout:
        - Header with title and estimated time.
        - Left sidebar for step navigation.
        - Center Markdown content.
        - Bottom navigation bar.
2. **Progress Tracking**: Local storage or simple session-based progress save.

## Phase 5: Local SaaS & Networking
1. **ngrok Integration**:
    - Add a script `run-public.sh` that starts docker-compose and ngrok.
    - Output the public ngrok URL to the console for the facilitator to share.
2. **Dynamic URL/QR**:
    - Ensure the frontend generates QR codes based on the CURRENT_HOST (even if it's an ngrok URL).

## Phase 6: Refinement
1. **Styling**: Match Google's blue/grey color palette.
2. **Testing**: Mock a full codelab session with multiple browser tabs.