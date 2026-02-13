# API Reference

Open Codelabs REST API reference.

## Base URL

```
http://localhost:8080/api
```

Production:
```
https://your-domain.com/api
```

## Authentication and sessions

- On login/registration, session cookie (`oc_session`) and CSRF cookie (`oc_csrf`) are issued.
- **With a session cookie**, `POST/PUT/DELETE` requests must include the `oc_csrf` value in the `X-CSRF-Token` header.

### Admin login

`POST /login`

**Request Body**:
```json
{
  "admin_id": "admin",
  "admin_pw": "admin123"
}
```

**Response** (200 OK):
```json
{ "status": "ok" }
```

### Session check

`GET /session`

**Response** (200 OK):
```json
{
  "sub": "admin",
  "role": "admin",
  "codelab_id": null,
  "exp": 1730000000
}
```

### Logout

`POST /logout`

**Response** (204 No Content)

### Admin settings (store Gemini key)

`POST /admin/settings`

**Request Body**:
```json
{
  "gemini_api_key": "ENCRYPTED_KEY"
}
```

**Response** (200 OK)

!!! note
    `gemini_api_key` must be encrypted with `ADMIN_PW`. Plaintext keys are rejected.

## Codelabs

### List all

`GET /codelabs`

**Response** (200 OK):
```json
[
  {
    "id": "codelab_xxx",
    "title": "Build a REST API with Rust",
    "description": "Build a RESTful API server with Axum",
    "author": "Jane Doe",
    "is_public": true,
    "quiz_enabled": false,
    "require_quiz": false,
    "require_feedback": false,
    "guide_markdown": null,
    "created_at": "2024-12-27T10:00:00"
  }
]
```

### Get a codelab

`GET /codelabs/:id`

**Response** (200 OK):
```json
[
  {
    "id": "codelab_xxx",
    "title": "Build a REST API with Rust",
    "description": "Build a RESTful API server with Axum",
    "author": "Jane Doe",
    "is_public": true,
    "quiz_enabled": false,
    "require_quiz": false,
    "require_feedback": false,
    "guide_markdown": null,
    "created_at": "2024-12-27T10:00:00"
  },
  [
    {
      "id": "step_xxx",
      "codelab_id": "codelab_xxx",
      "step_number": 1,
      "title": "Project setup",
      "content_markdown": "# Project Setup\n\n..."
    }
  ]
]
```

### Create a codelab

`POST /codelabs`

**Request Body**:
```json
{
  "title": "New Codelab",
  "description": "Codelab description",
  "author": "Author name",
  "is_public": true,
  "quiz_enabled": false,
  "require_quiz": false,
  "require_feedback": false,
  "guide_markdown": "# Prep Guide\n..."
}
```

**Response** (200 OK): codelab object

### Update codelab

`PUT /codelabs/:id`

**Request Body**: same as create

### Delete codelab

`DELETE /codelabs/:id`

**Response** (204 No Content)

### Copy codelab

`POST /codelabs/:id/copy`

**Response** (200 OK): copied codelab object

### Update steps

`PUT /codelabs/:id/steps`

**Request Body**:
```json
{
  "steps": [
    { "title": "Step 1 title", "content_markdown": "# Step 1\n\n..." }
  ]
}
```

**Response** (200 OK):
```json
{ "status": "ok" }
```

### Export

`GET /codelabs/:id/export`

**Response**: ZIP download (`application/zip`)

### Import

`POST /codelabs/import`

**Request**: `multipart/form-data` (`file` field)

**Response** (200 OK): codelab object

### Get chat history

`GET /codelabs/:id/chat`

**Response** (200 OK):
```json
[
  {
    "id": "chat_xxx",
    "codelab_id": "codelab_xxx",
    "sender_name": "Facilitator",
    "message": "Hello",
    "msg_type": "chat",
    "target_id": null,
    "created_at": "2024-12-27T10:20:00"
  }
]
```

### Get inline comments

`GET /codelabs/:id/inline-comments`

Optional query params:

- `target_type`: `step` or `guide`
- `target_step_id`: step ID when `target_type=step`

**Response** (200 OK):
```json
[
  {
    "id": "thread_xxx",
    "codelab_id": "codelab_xxx",
    "anchor_key": "step|step_xxx|hash|12|34",
    "target_type": "step",
    "target_step_id": "step_xxx",
    "start_offset": 12,
    "end_offset": 34,
    "selected_text": "Selected text",
    "content_hash": "abc123",
    "created_by_attendee_id": "attendee_xxx",
    "created_at": "2026-02-13T10:20:00Z",
    "messages": [
      {
        "id": "comment_xxx",
        "thread_id": "thread_xxx",
        "codelab_id": "codelab_xxx",
        "author_role": "attendee",
        "author_id": "attendee_xxx",
        "author_name": "Jane Doe",
        "message": "This part is confusing.",
        "created_at": "2026-02-13T10:20:00Z"
      }
    ]
  }
]
```

### Create inline comment

`POST /codelabs/:id/inline-comments`

**Request Body**:
```json
{
  "anchor_key": "step|step_xxx|hash|12|34",
  "target_type": "step",
  "target_step_id": "step_xxx",
  "start_offset": 12,
  "end_offset": 34,
  "selected_text": "Selected text",
  "content_hash": "abc123",
  "message": "This part is confusing."
}
```

If the same `anchor_key` already exists, the server appends a message to the existing thread. Overlapping different ranges return `400`.

### Reply to inline comment

`POST /codelabs/:id/inline-comments/:thread_id/comments`

**Request Body**:
```json
{
  "message": "You can read this concept first.",
  "content_hash": "abc123"
}
```

If `content_hash` differs from the current content, the thread is treated as stale and the server returns `400`.

### Delete inline comment

`DELETE /codelabs/:id/inline-comments/:thread_id/comments/:comment_id`

- Backend mode: author or admin can delete
- Serverless mode (Supabase/Firebase): author only

When the last message in a thread is deleted, the thread is deleted too.

## Attendees

### Register attendee

`POST /codelabs/:id/register`

**Request Body**:
```json
{
  "name": "Jane Doe",
  "code": "ATTEND2024",
  "email": "test@example.com"
}
```

**Response** (200 OK):
```json
{
  "id": "attendee_xxx",
  "codelab_id": "codelab_xxx",
  "name": "Jane Doe",
  "email": "test@example.com",
  "current_step": 1,
  "is_completed": false,
  "completed_at": null,
  "created_at": "2024-12-27T10:10:00"
}
```

### List attendees

`GET /codelabs/:id/attendees`

**Response** (200 OK): attendee array

### Mark completion

`POST /codelabs/:id/complete`

**Response** (200 OK):
```json
{ "status": "ok" }
```

### Get certificate

`GET /certificates/:id`

**Response** (200 OK):
```json
{
  "attendee_name": "Jane Doe",
  "codelab_title": "Build a REST API with Rust",
  "codelab_id": "codelab_xxx",
  "author": "Jane Doe",
  "completed_at": "2024-12-27T12:00:00",
  "verification_url": "/verify/attendee_xxx"
}
```

## Help requests

### Create help request

`POST /codelabs/:id/help`

**Request Body**:
```json
{ "step_number": 3 }
```

**Response** (200 OK):
```json
{ "status": "ok" }
```

### List help requests

`GET /codelabs/:id/help`

**Response** (200 OK):
```json
[
  {
    "id": "help_xxx",
    "codelab_id": "codelab_xxx",
    "attendee_id": "attendee_xxx",
    "attendee_name": "Jane Doe",
    "step_number": 3,
    "status": "pending",
    "created_at": "2024-12-27T10:15:00"
  }
]
```

### Resolve help request

`POST /codelabs/:id/help/:help_id/resolve`

**Response** (200 OK):
```json
{ "status": "ok" }
```

## Feedback

### Submit feedback

`POST /codelabs/:id/feedback`

**Request Body**:
```json
{
  "difficulty": "3",
  "satisfaction": "5",
  "comment": "Very helpful"
}
```

### List feedback

`GET /codelabs/:id/feedback`

**Response** (200 OK): feedback array

## Materials

### List materials

`GET /codelabs/:id/materials`

### Add material

`POST /codelabs/:id/materials`

**Request Body**:
```json
{
  "title": "Material link",
  "material_type": "link",
  "link_url": "https://example.com",
  "file_path": null
}
```

### Delete material

`DELETE /codelabs/:id/materials/:material_id`

### Upload material file

`POST /upload/material`

**Request**: `multipart/form-data` (`file`)

**Response** (200 OK):
```json
{
  "url": "/uploads/materials/xxxx.ext",
  "original_name": "guide.pdf"
}
```

## Quizzes

### Get quizzes

`GET /codelabs/:id/quizzes`

### Update quizzes

`PUT /codelabs/:id/quizzes`

**Request Body**:
```json
[
  {
    "question": "Question",
    "quiz_type": "multiple_choice",
    "options": ["A", "B", "C"],
    "correct_answer": 1
  }
]
```

### Submit quizzes

`POST /codelabs/:id/quizzes/submit`

**Request Body**:
```json
{
  "submissions": [
    { "quiz_id": "quiz_xxx", "answer": "B", "is_correct": true }
  ]
}
```

### List quiz submissions (admin)

`GET /codelabs/:id/quizzes/submissions`

## Submissions

### Submit assignment

`POST /codelabs/:id/attendees/:attendee_id/submissions`

**Request**: `multipart/form-data` (`file`)

### List submissions

`GET /codelabs/:id/submissions`

### Delete submission

`DELETE /codelabs/:id/attendees/:attendee_id/submissions/:submission_id`

## Upload

### Upload image

`POST /upload/image`

**Request**: `multipart/form-data` (`file`)

**Response** (200 OK):
```json
{ "url": "/uploads/xxxx.webp" }
```

## AI

### Gemini streaming proxy

`POST /ai/stream`

**Request Body**:
```json
{
  "prompt": "Question",
  "system_instruction": "Optional system",
  "api_key": "ENCRYPTED_KEY",
  "model": "gemini-3-flash-preview",
  "generation_config": {},
  "tools": {},
  "codelab_id": "codelab_xxx",
  "step_number": 2
}
```

**Response**: SSE stream (`text/event-stream`)

### Save AI conversation

`POST /ai/conversations`

**Request Body**:
```json
{
  "codelab_id": "codelab_xxx",
  "step_number": 2,
  "question": "Question",
  "answer": "Answer",
  "model": "gemini-3-flash-preview"
}
```

**Response** (200 OK):
```json
{ "id": "conversation_xxx" }
```

### List AI conversations (admin)

`GET /codelabs/:id/ai/conversations`

## Audit logs

`GET /admin/audit-logs`

**Query Params**: `limit`, `offset`, `codelab_id`, `action`

## Code Server

### Create workspace

`POST /codeserver`

**Request Body**:
```json
{
  "codelab_id": "codelab_xxx",
  "structure_type": "branch",
  "workspace_files": [
    { "path": "README.md", "content": "# Hello" }
  ]
}
```

### Get/delete workspace

- `GET /codeserver/:codelab_id`
- `DELETE /codeserver/:codelab_id`

### Create branch/folder

- `POST /codeserver/:codelab_id/branch`
- `POST /codeserver/:codelab_id/folder`

### Download workspace

`GET /codeserver/:codelab_id/download`

### Branch-based files

- `GET /codeserver/:codelab_id/branches`
- `GET /codeserver/:codelab_id/branches/:branch/files`
- `GET /codeserver/:codelab_id/branches/:branch/file?file=path/to/file`
- `POST /codeserver/:codelab_id/branches/:branch/files`

### Folder-based files

- `GET /codeserver/:codelab_id/folders`
- `GET /codeserver/:codelab_id/folders/:folder/files`
- `GET /codeserver/:codelab_id/folders/:folder/file?file=path/to/file`
- `POST /codeserver/:codelab_id/folders/:folder/files`

## WebSocket

### Connect

`WS /api/ws/:id`

### Client -> server

```json
{ "type": "chat", "message": "Hello" }
```

```json
{ "type": "dm", "target_id": "attendee_xxx", "message": "Private message" }
```

```json
{ "type": "step_progress", "step_number": 3 }
```

### Server -> client

```json
{ "type": "chat", "sender": "Jane Doe", "message": "Hello" }
```

```json
{ "type": "dm", "sender": "Facilitator", "message": "Private message", "target_id": "attendee_xxx" }
```

```json
{ "type": "step_progress", "attendee_id": "attendee_xxx", "step_number": 3 }
```

```json
{ "type": "help_request", "attendee_id": "attendee_xxx", "step_number": 3 }
```

```json
{ "type": "inline_comment_changed", "target_type": "step", "target_step_id": "step_xxx" }
```

## Error responses

All errors follow this format:

```json
{ "error": "Error message" }
```

### HTTP status codes

| Code | Meaning | Example |
|------|------|------|
| `200` | Success | Read/update success |
| `201` | Created | Resource created |
| `204` | No Content | Delete success |
| `400` | Bad Request | Missing required field |
| `401` | Unauthorized | Login required |
| `403` | Forbidden | Access denied |
| `404` | Not Found | Resource missing |
| `409` | Conflict | Duplicate registration |
