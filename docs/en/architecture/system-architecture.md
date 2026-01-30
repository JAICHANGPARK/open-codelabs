# System Architecture

This document describes the overall system architecture of Open Codelabs.

## Architecture overview

```
+---------------------------------------------------+
| Users                                             |
|  +-------------+                +--------------+ |
|  | Facilitator |                | Attendee     | |
|  | (admin)     |                | (participant)| |
|  +------+------+                +------+-------+ |
+---------+------------------------------+---------+
          |                              |
          |           Browser            |
          +---------------+--------------+
                          | HTTPS
          +---------------+--------------+
          | Frontend Layer               |
          | (SvelteKit + Vite)           |
          |  +--------+    +----------+  |
          |  | Pages  |    |Components|  |
          |  +--------+    +----------+  |
          |  +--------+    +----------+  |
          |  | Stores |    |   API    |  |
          |  +--------+    +----------+  |
          +---------------+--------------+
                          | HTTP/WebSocket
          +---------------+--------------+
          | Backend Layer                |
          | (Rust + Axum)                |
          |  +--------+    +----------+  |
          |  | Router |    | Handlers |  |
          |  +--------+    +----------+  |
          |  +--------+    +----------+  |
          |  |  Auth  |    | WebSocket|  |
          |  +--------+    +----------+  |
          +---------------+--------------+
                          | SQLx
          +---------------+--------------+
          | Database Layer               |
          | (SQLite)                     |
          |  +------------------------+  |
          |  | Tables and Relations   |  |
          |  +------------------------+  |
          +-------------------------------+
```

## Layered architecture

### Presentation layer (Frontend)
- **SvelteKit 5**: SSR/SPA hybrid
- **Vite**: bundler and dev server
- **Tailwind CSS**: styling
- **WebSocket client**: real-time communication

### Application layer (Backend)
- **Axum**: web framework
- **Tokio**: async runtime
- **Tower**: middleware
- **WebSocket**: real-time server

### Data layer (Database)
- **SQLite**: relational database
- **SQLx**: type-safe queries
- **Migrations**: schema versioning

## Communication protocols

### HTTP REST API
- CRUD operations
- State management
- File upload/download

### WebSocket
- Real-time chat
- Progress updates
- Help request alerts

## Next steps

- [Backend architecture](backend.md) - backend details
- [Frontend architecture](frontend.md) - frontend details
- [WebSocket](websocket.md) - real-time communication
