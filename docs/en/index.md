# Open Codelabs (Hands-on System)

[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/svelte-v5-ff3e00.svg)](https://svelte.dev/)
[![Bun](https://img.shields.io/badge/bun-v1.0+-black.svg)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/docker-blue.svg)](https://www.docker.com/)
[![Firebase](https://img.shields.io/badge/firebase-yellow.svg)](https://firebase.google.com/)
[![Supabase](https://img.shields.io/badge/supabase-3FCF8E.svg)](https://supabase.com/)

**Open-Codelabs: Hands-on System** is an open-source platform designed to run Google Codelab-style hands-on sessions. It supports both facilitator and attendee roles, lets you author content with Markdown or AI, and provides self-hosted and serverless deployment options.

## Overview

This project is an interactive hands-on platform where educators (Facilitators) create step-by-step guides and attendees learn at their own pace. It is built on a SaaS-friendly architecture and manages content through Markdown and AI workflows. Built-in i18n support is provided.

## Key Features

### Role-based workflow
- **Facilitator (admin)**: create and manage codelabs, edit content, manage attendees
- **Attendee**: follow steps, track progress, request help

### AI-assisted content creation
- Gemini-based codelab generator (source code or docs input)
- Prep guide generation and centralized materials management

### Codelab workspace (optional)
- code-server based workspace
- Step snapshots (branch/folder modes) and downloads

### Completion and verification
- Require quizzes and feedback for completion
- Auto-issued certificates with verification URLs

### Google Codelab-style UI
- Familiar interface
- Responsive layout for mobile/tablet
- Dark mode support

### Real-time interaction
- WebSocket-based chat
- 1:1 DM
- Help request queue and management
- Live attendee progress monitoring
- Submission panel and certificate raffle

### Markdown-based content
- Simple authoring
- Code highlighting
- Image upload and management
- Import/Export support

### Deployment flexibility
- One-command Docker deployment
- Public exposure via ngrok/bore/cloudflared
- QR code invites
- Rust + SQLite self-hosting or Firebase/Supabase serverless mode
- Prebuilt images via `docker-compose.images.yml`

## Tech Stack

### Frontend
- **Framework**: SvelteKit 5 (Vite + TypeScript)
- **Runtime**: Bun
- **Styling**: Tailwind CSS 4.0
- **State**: Svelte Runes
- **i18n**: svelte-i18n
- **Markdown**: marked & dompurify
- **Icons**: Lucide Svelte
- **QR Code**: svelte-qrcode

### Backend
- **Language**: Rust
- **Framework**: Axum (Tokio async runtime)
- **Database**: SQLite with SQLx
- **WebSocket**: Axum WebSocket support
- **Serialization**: Serde (JSON)

### Cloud (serverless options)
- **Firebase**: Hosting, Firestore, Storage
- **Supabase**: Postgres, Auth, Storage, Realtime

### DevOps
- Docker & Docker Compose
- Multi-stage builds
- ngrok/bore/cloudflared for tunneling

## Quickstart

The simplest way to run the full stack is via Docker:

```bash
docker compose up --build
```

- **Frontend**: [http://localhost:5173](http://localhost:5173)
- **Backend API**: [http://localhost:8080](http://localhost:8080)

To run with prebuilt images:

```bash
cp .env.sample .env
docker compose -f docker-compose.images.yml up
```

For more details, see the [Installation Guide](getting-started/installation.md).

## Documentation Map

- **[Getting Started](getting-started/quickstart.md)**: Quickstart and setup
- **[Self-Hosting](self-hosting/docker.md)**: Docker, local dev, public deployment
- **[Serverless Deployment](self-hosting/firebase.md)**: Firebase guide
- **[Serverless Deployment](self-hosting/supabase.md)**: Supabase guide
- **[Specifications](specification/overview.md)**: Features and API reference
- **[Architecture](architecture/system-architecture.md)**: System design docs
- **[Code Guide](code-guide/backend-examples.md)**: Examples and usage
- **[User Guide](user-guide/facilitator.md)**: Facilitator and attendee features
- **[Contributing](contributing/guide.md)**: How to contribute
- **[FAQ](faq.md)**: Common questions

## Contributing

Want to contribute? See the [Contribution Guide](contributing/guide.md).

## License

This project is licensed under the [Apache License 2.0](license.md).

## Links

- [GitHub Repository](https://github.com/JAICHANGPARK/open-codelabs)
- [Issue Tracker](https://github.com/JAICHANGPARK/open-codelabs/issues)
- [Pull Requests](https://github.com/JAICHANGPARK/open-codelabs/pulls)
