# Backend Architecture

This document describes the Rust Axum backend architecture.

## Directory structure

```
backend/
|-- src/
|   |-- main.rs              # entry point
|   |-- lib.rs               # shared modules
|   |-- api/                 # routing and handlers
|   |   |-- mod.rs
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
|   |-- domain/              # domain models/services
|   |   |-- mod.rs
|   |   |-- models.rs
|   |   `-- services/
|   |-- infrastructure/      # DB, audit logs
|   |   |-- mod.rs
|   |   |-- database.rs
|   |   `-- audit.rs
|   |-- middleware/          # auth/security/rate limit
|   |   |-- mod.rs
|   |   |-- auth.rs
|   |   |-- request_info.rs
|   |   |-- rate_limit.rs
|   |   `-- security.rs
|   `-- utils/               # utilities
|       |-- mod.rs
|       |-- crypto.rs
|       |-- error.rs
|       `-- validation.rs
|-- migrations/              # DB migrations
|-- tests/                   # integration tests
`-- Cargo.toml               # dependencies
```

## Core components

### 1. Router
HTTP routing and middleware.

### 2. Handlers
Business logic.

### 3. Domain
Core models and domain services.

### 4. Infrastructure
DB connection, audit logs, and other infra.

### 5. Middleware
Session auth, CSRF, security headers, rate limiting.

### 6. WebSocket
Real-time communication.

## Next steps

- [API reference](../specification/api-reference.md)
- [Backend code examples](../code-guide/backend-examples.md)
