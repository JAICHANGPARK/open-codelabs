# API 레퍼런스

Open Codelabs REST API 레퍼런스입니다.

## Base URL

```
http://localhost:8080/api
```

프로덕션:
```
https://your-domain.com/api
```

## 인증 및 세션

- 로그인/등록 시 세션 쿠키(`oc_session`)와 CSRF 쿠키(`oc_csrf`)가 발급됩니다.
- **세션 쿠키가 있는 상태에서** `POST/PUT/DELETE` 요청을 보낼 때는 `X-CSRF-Token` 헤더에 `oc_csrf` 값을 전달해야 합니다.

### 관리자 로그인

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

### 세션 확인

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

### 로그아웃

`POST /logout`

**Response** (204 No Content)

### 관리자 설정 (Gemini 키 저장)

`POST /admin/settings`

**Request Body**:
```json
{
  "gemini_api_key": "ENCRYPTED_KEY"
}
```

**Response** (200 OK)

!!! note
    `gemini_api_key`는 `ADMIN_PW`로 암호화된 값이어야 합니다. 평문 키는 거부됩니다.

## Codelabs

### 전체 목록 조회

`GET /codelabs`

**Response** (200 OK):
```json
[
  {
    "id": "codelab_xxx",
    "title": "Rust로 REST API 만들기",
    "description": "Axum 프레임워크로 RESTful API 서버 구축",
    "author": "홍길동",
    "is_public": true,
    "quiz_enabled": false,
    "require_quiz": false,
    "require_feedback": false,
    "guide_markdown": null,
    "created_at": "2024-12-27T10:00:00"
  }
]
```

### 특정 Codelab 조회

`GET /codelabs/:id`

**Response** (200 OK):
```json
[
  {
    "id": "codelab_xxx",
    "title": "Rust로 REST API 만들기",
    "description": "Axum 프레임워크로 RESTful API 서버 구축",
    "author": "홍길동",
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
      "title": "프로젝트 설정",
      "content_markdown": "# 프로젝트 설정\n\n..."
    }
  ]
]
```

### Codelab 생성

`POST /codelabs`

**Request Body**:
```json
{
  "title": "새 Codelab",
  "description": "Codelab 설명",
  "author": "작성자 이름",
  "is_public": true,
  "quiz_enabled": false,
  "require_quiz": false,
  "require_feedback": false,
  "guide_markdown": "# 준비 가이드\n..."
}
```

**Response** (200 OK): Codelab 객체

### Codelab 정보 수정

`PUT /codelabs/:id`

**Request Body**: 생성과 동일

### Codelab 삭제

`DELETE /codelabs/:id`

**Response** (204 No Content)

### Codelab 복사

`POST /codelabs/:id/copy`

**Response** (200 OK): 복사된 Codelab 객체

### Steps 업데이트

`PUT /codelabs/:id/steps`

**Request Body**:
```json
{
  "steps": [
    { "title": "Step 1 제목", "content_markdown": "# Step 1 내용\n\n..." }
  ]
}
```

**Response** (200 OK):
```json
{ "status": "ok" }
```

### Export

`GET /codelabs/:id/export`

**Response**: ZIP 다운로드 (`application/zip`)

### Import

`POST /codelabs/import`

**Request**: `multipart/form-data` (`file` 필드)

**Response** (200 OK): Codelab 객체

### 채팅 기록 조회

`GET /codelabs/:id/chat`

**Response** (200 OK):
```json
[
  {
    "id": "chat_xxx",
    "codelab_id": "codelab_xxx",
    "sender_name": "진행자",
    "message": "안녕하세요!",
    "msg_type": "chat",
    "target_id": null,
    "created_at": "2024-12-27T10:20:00"
  }
]
```

## Attendees

### 참가자 등록

`POST /codelabs/:id/register`

**Request Body**:
```json
{
  "name": "홍길동",
  "code": "ATTEND2024",
  "email": "test@example.com"
}
```

**Response** (200 OK):
```json
{
  "id": "attendee_xxx",
  "codelab_id": "codelab_xxx",
  "name": "홍길동",
  "email": "test@example.com",
  "current_step": 1,
  "is_completed": false,
  "completed_at": null,
  "created_at": "2024-12-27T10:10:00"
}
```

### 참가자 목록 조회

`GET /codelabs/:id/attendees`

**Response** (200 OK): 참가자 배열

### 수료 처리

`POST /codelabs/:id/complete`

**Response** (200 OK):
```json
{ "status": "ok" }
```

### 수료 인증서 조회

`GET /certificates/:id`

**Response** (200 OK):
```json
{
  "attendee_name": "홍길동",
  "codelab_title": "Rust로 REST API 만들기",
  "codelab_id": "codelab_xxx",
  "author": "홍길동",
  "completed_at": "2024-12-27T12:00:00",
  "verification_url": "/verify/attendee_xxx"
}
```

## Help Requests

### 도움 요청 생성

`POST /codelabs/:id/help`

**Request Body**:
```json
{ "step_number": 3 }
```

**Response** (200 OK):
```json
{ "status": "ok" }
```

### 도움 요청 목록 조회

`GET /codelabs/:id/help`

**Response** (200 OK):
```json
[
  {
    "id": "help_xxx",
    "codelab_id": "codelab_xxx",
    "attendee_id": "attendee_xxx",
    "attendee_name": "홍길동",
    "step_number": 3,
    "status": "pending",
    "created_at": "2024-12-27T10:15:00"
  }
]
```

### 도움 요청 해결

`POST /codelabs/:id/help/:help_id/resolve`

**Response** (200 OK):
```json
{ "status": "ok" }
```

## Feedback

### 피드백 제출

`POST /codelabs/:id/feedback`

**Request Body**:
```json
{
  "difficulty": "3",
  "satisfaction": "5",
  "comment": "매우 유익했습니다!"
}
```

### 피드백 조회

`GET /codelabs/:id/feedback`

**Response** (200 OK): 피드백 배열

## Materials

### 자료 목록

`GET /codelabs/:id/materials`

### 자료 추가

`POST /codelabs/:id/materials`

**Request Body**:
```json
{
  "title": "자료 링크",
  "material_type": "link",
  "link_url": "https://example.com",
  "file_path": null
}
```

### 자료 삭제

`DELETE /codelabs/:id/materials/:material_id`

### 자료 파일 업로드

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

### 퀴즈 조회

`GET /codelabs/:id/quizzes`

### 퀴즈 업데이트

`PUT /codelabs/:id/quizzes`

**Request Body**:
```json
[
  {
    "question": "질문",
    "quiz_type": "multiple_choice",
    "options": ["A", "B", "C"],
    "correct_answer": 1
  }
]
```

### 퀴즈 제출

`POST /codelabs/:id/quizzes/submit`

**Request Body**:
```json
{
  "submissions": [
    { "quiz_id": "quiz_xxx", "answer": "B", "is_correct": true }
  ]
}
```

### 퀴즈 제출 결과 조회 (관리자)

`GET /codelabs/:id/quizzes/submissions`

## Submissions

### 과제 제출

`POST /codelabs/:id/attendees/:attendee_id/submissions`

**Request**: `multipart/form-data` (`file`)

### 제출 목록

`GET /codelabs/:id/submissions`

### 제출 삭제

`DELETE /codelabs/:id/attendees/:attendee_id/submissions/:submission_id`

## Upload

### 이미지 업로드

`POST /upload/image`

**Request**: `multipart/form-data` (`file`)

**Response** (200 OK):
```json
{ "url": "/uploads/xxxx.webp" }
```

## AI

### Gemini 스트리밍 프록시

`POST /ai/stream`

**Request Body**:
```json
{
  "prompt": "질문",
  "system_instruction": "Optional system",
  "api_key": "ENCRYPTED_KEY",
  "model": "gemini-3-flash-preview",
  "generation_config": {},
  "tools": {},
  "codelab_id": "codelab_xxx",
  "step_number": 2
}
```

**Response**: SSE 스트림 (`text/event-stream`)

### AI 대화 저장

`POST /ai/conversations`

**Request Body**:
```json
{
  "codelab_id": "codelab_xxx",
  "step_number": 2,
  "question": "질문",
  "answer": "답변",
  "model": "gemini-3-flash-preview"
}
```

**Response** (200 OK):
```json
{ "id": "conversation_xxx" }
```

### AI 대화 조회 (관리자)

`GET /codelabs/:id/ai/conversations`

## 감사 로그

`GET /admin/audit-logs`

**Query Params**: `limit`, `offset`, `codelab_id`, `action`

## Code Server

### 워크스페이스 생성

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

### 워크스페이스 조회/삭제

- `GET /codeserver/:codelab_id`
- `DELETE /codeserver/:codelab_id`

### 브랜치/폴더 생성

- `POST /codeserver/:codelab_id/branch`
- `POST /codeserver/:codelab_id/folder`

### 워크스페이스 다운로드

`GET /codeserver/:codelab_id/download`

### 브랜치 기반 파일

- `GET /codeserver/:codelab_id/branches`
- `GET /codeserver/:codelab_id/branches/:branch/files`
- `GET /codeserver/:codelab_id/branches/:branch/file?file=path/to/file`
- `POST /codeserver/:codelab_id/branches/:branch/files`

### 폴더 기반 파일

- `GET /codeserver/:codelab_id/folders`
- `GET /codeserver/:codelab_id/folders/:folder/files`
- `GET /codeserver/:codelab_id/folders/:folder/file?file=path/to/file`
- `POST /codeserver/:codelab_id/folders/:folder/files`

## WebSocket

### 연결

`WS /api/ws/:id`

### 클라이언트 → 서버

```json
{ "type": "chat", "message": "안녕하세요!" }
```

```json
{ "type": "dm", "target_id": "attendee_xxx", "message": "개인 메시지" }
```

```json
{ "type": "step_progress", "step_number": 3 }
```

### 서버 → 클라이언트

```json
{ "type": "chat", "sender": "홍길동", "message": "안녕하세요!" }
```

```json
{ "type": "dm", "sender": "진행자", "message": "개인 메시지", "target_id": "attendee_xxx" }
```

```json
{ "type": "step_progress", "attendee_id": "attendee_xxx", "step_number": 3 }
```

```json
{ "type": "help_request", "attendee_id": "attendee_xxx", "step_number": 3 }
```

## 에러 응답

모든 에러는 다음 형식을 따릅니다:

```json
{ "error": "에러 메시지" }
```

### HTTP 상태 코드

| 코드 | 의미 | 예시 |
|------|------|------|
| `200` | 성공 | 조회/수정 성공 |
| `201` | 생성됨 | 리소스 생성 |
| `204` | 내용 없음 | 삭제 성공 |
| `400` | 잘못된 요청 | 필수 필드 누락 |
| `401` | 인증 실패 | 로그인 필요 |
| `403` | 권한 없음 | 접근 불가 |
| `404` | 없음 | 리소스 없음 |
| `409` | 충돌 | 중복 등록 |
