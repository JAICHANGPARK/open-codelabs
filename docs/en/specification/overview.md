# Project Overview

A high-level overview of Open Codelabs and its core concepts.

## Project goals

Open Codelabs is built to achieve the following goals:

### 1. Accessibility
- **Easy setup**: run the full system with a single Docker command
- **Low barrier**: author content with Markdown only
- **Cross-platform**: runs on all major OSes

### 2. Usability
- **Intuitive UI**: familiar Google Codelab style
- **Real-time interaction**: WebSocket chat and help requests
- **Progress tracking**: monitor attendee progress

### 3. Flexibility
- **Local-first**: works without internet
- **SaaS-ready**: cloud deployment
- **Import/Export**: content portability

### 4. Scalability
- **Lightweight start**: SQLite for quick setup
- **Horizontal growth**: migrate to PostgreSQL when needed
- **Modular**: isolated features by domain

## Core concepts

### Codelab
A step-by-step learning guide.

- **Metadata**: title, description, author
- **Steps**: ordered learning steps
- **Attendees**: registered learners

### Step
An individual learning step in a codelab.

- **Order**: sorted by `step_number`
- **Title**: step title
- **Content**: Markdown content
- **Progress**: per-attendee completion

### Facilitator
A person who creates and manages codelabs.

**Permissions**:
- Codelab CRUD
- Step editing
- Attendee monitoring
- Help request management
- Chat participation

### Attendee
A person who learns a codelab.

**Permissions**:
- View codelabs
- Navigate steps
- Request help
- Join chat
- Submit feedback

### Progress
Learning status per attendee.

- **current_step**: current step
- **Real-time sync**: updated via WebSocket
- **Dashboard**: monitored by facilitator

### Help request
Feature for attendees to ask for help.

**States**:
- `pending`: waiting
- `resolved`: resolved

**Info**:
- step at request time
- requester
- timestamp

### Chat
Real-time communication.

**Types**:
- `chat`: public chat
- `dm`: 1:1 messages

## System structure

### Layered architecture

```
+-------------------------------+
| Frontend (SvelteKit)          |
|  +----------+  +-----------+ |
|  | Admin    |  | Attendee  | |
|  | Dashboard|  | View      | |
|  +----------+  +-----------+ |
+---------------+---------------+
                | HTTP / WebSocket
+---------------+---------------+
| Backend (Axum)                |
|  +----------+  +-----------+ |
|  | API      |  | WebSocket | |
|  | Handlers |  | Handler   | |
|  +----------+  +-----------+ |
+---------------+---------------+
                | SQLx
+---------------+---------------+
| Database (SQLite)             |
| codelabs, steps, attendees... |
+-------------------------------+
```

### Data flow

#### Codelab creation flow

```
Facilitator -> POST /api/codelabs
               -> Axum handler
               -> SQLx query
               -> SQLite DB
               -> Response (codelab JSON)
```

#### Real-time chat flow

```
User A -> WebSocket -> Backend -> WebSocket -> User B
                     -> DashMap (in-memory)
                     -> DB (persistence)
```

## Technical decisions

### Why Rust (backend)?

**Pros**:
- **Performance**: native speed
- **Safety**: memory safety
- **Concurrency**: Tokio async runtime
- **Type safety**: compile-time checks

**Cons**:
- Steeper learning curve
- Longer compile times

**Reason**:
Workshops require stable performance under tens to hundreds of concurrent users.

### Why SvelteKit (frontend)?

**Pros**:
- **Lightweight**: small bundle
- **Fast**: compiled, no virtual DOM
- **Simple**: minimal boilerplate
- **SSR/SSG**: SEO and performance

**Reason**:
Fast load times and smooth UX are critical to learning.

### Why SQLite?

**Pros**:
- **Zero-config**: no setup
- **Portable**: single file
- **Fast**: embedded DB
- **Enough**: supports thousands of users

**Limits**:
- Concurrent writes
- No network access

**Scaling plan**:
Migrate to PostgreSQL/MySQL as needed (via SQLx).

### Why WebSocket?

**Alternatives**:
- HTTP Long Polling
- Server-Sent Events (SSE)
- WebRTC

**Reason**:
- Bi-directional real-time communication
- Low latency
- Standard and stable

## Non-functional requirements

### Performance

| Metric | Target | Current |
|------|------|------|
| API response time | < 100ms | ~50ms |
| Page load | < 2s | ~1s |
| Concurrent users | 100+ | tested |
| Memory usage | < 512MB | ~200MB |

### Availability

- **Uptime**: 99.9% during events
- **Auto restart**: Docker/systemd
- **Health check**: API endpoint

### Security

- **Auth**: admin login only
- **Authorization**: role-based (Facilitator/Attendee)
- **Input validation**: all API input
- **XSS prevention**: DOMPurify
- **SQL injection**: SQLx prepared statements

### Accessibility

- **Responsive**: mobile/tablet support
- **Keyboard navigation**: tab key support
- **Screen reader**: ARIA labels
- **Localization**: i18n ready (Korean currently)

## Constraints

### Current constraints

1. **Single admin**: only one facilitator
2. **Simple auth**: password only
3. **SQLite limits**: concurrent writes
4. **WebSocket scale**: single server

### Future improvements

1. **Multi-admin**: multiple facilitators
2. **OAuth**: social login
3. **Database**: PostgreSQL support
4. **Redis**: WebSocket message broker
5. **S3**: image storage

## License

Apache License 2.0 - free to use, modify, and distribute.

## Next steps

- [Feature specs](features.md) - detailed features
- [Database schema](database-schema.md) - DB structure
- [API reference](api-reference.md) - REST API
