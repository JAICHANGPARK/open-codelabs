# API 레퍼런스

Open Codelabs REST API의 완전한 레퍼런스입니다.

## Base URL

```
http://localhost:8080/api
```

프로덕션:
```
https://your-domain.com/api
```

## 인증

### 관리자 인증

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
{
  "token": "admin_session_token"
}
```

**Errors**:
- `401 Unauthorized`: 잘못된 자격증명

!!! note "현재 구현"
    현재는 간단한 ID/PW 검증만 지원합니다. JWT는 향후 추가 예정입니다.

## Codelabs

### 전체 Codelab 목록 조회

`GET /codelabs`

**Response** (200 OK):
```json
[
  {
    "id": "codelab_01234567-89ab-cdef-0123-456789abcdef",
    "title": "Rust로 REST API 만들기",
    "description": "Axum 프레임워크로 RESTful API 서버 구축",
    "author": "AntiGravity Team",
    "created_at": "2024-12-27T10:00:00"
  }
]
```

### 특정 Codelab 조회

`GET /codelabs/:id`

**Path Parameters**:
- `id`: Codelab ID

**Response** (200 OK):
```json
[
  {
    "id": "codelab_xxx",
    "title": "Rust로 REST API 만들기",
    "description": "Axum 프레임워크로 RESTful API 서버 구축",
    "author": "AntiGravity Team",
    "created_at": "2024-12-27T10:00:00"
  },
  [
    {
      "id": "step_xxx",
      "codelab_id": "codelab_xxx",
      "step_number": 1,
      "title": "프로젝트 설정",
      "content_markdown": "# 프로젝트 설정\n\n..."
    },
    {
      "id": "step_yyy",
      "codelab_id": "codelab_xxx",
      "step_number": 2,
      "title": "데이터 모델 작성",
      "content_markdown": "# 데이터 모델...\n\n..."
    }
  ]
]
```

**Errors**:
- `404 Not Found`: Codelab 없음

### Codelab 생성

`POST /codelabs`

**Request Body**:
```json
{
  "title": "새 Codelab",
  "description": "Codelab 설명",
  "author": "작성자 이름"
}
```

**Response** (201 Created):
```json
{
  "id": "codelab_new",
  "title": "새 Codelab",
  "description": "Codelab 설명",
  "author": "작성자 이름",
  "created_at": "2024-12-27T10:05:00"
}
```

**Errors**:
- `400 Bad Request`: 필수 필드 누락

### Codelab 정보 수정

`PUT /codelabs/:id`

**Path Parameters**:
- `id`: Codelab ID

**Request Body**:
```json
{
  "title": "수정된 제목",
  "description": "수정된 설명",
  "author": "작성자"
}
```

**Response** (200 OK):
```json
{
  "id": "codelab_xxx",
  "title": "수정된 제목",
  "description": "수정된 설명",
  "author": "작성자",
  "created_at": "2024-12-27T10:00:00"
}
```

### Codelab 삭제

`DELETE /codelabs/:id`

**Response** (204 No Content)

**Errors**:
- `404 Not Found`: Codelab 없음

### Steps 업데이트

`PUT /codelabs/:id/steps`

**Request Body**:
```json
{
  "steps": [
    {
      "title": "Step 1 제목",
      "content_markdown": "# Step 1 내용\n\n..."
    },
    {
      "title": "Step 2 제목",
      "content_markdown": "# Step 2 내용\n\n..."
    }
  ]
}
```

**Response** (200 OK)

!!! note "기존 Steps 대체"
    기존 Steps는 모두 삭제되고 새로운 Steps로 대체됩니다.

## Export & Import

### Codelab Export

`GET /codelabs/:id/export`

**Response** (200 OK):
- Content-Type: `application/zip`
- ZIP 파일 다운로드

**ZIP 파일 구조**:
```
codelab_xxx.zip
├── codelab.json
├── step_1.md
├── step_2.md
└── step_3.md
```

`codelab.json`:
```json
{
  "title": "Codelab 제목",
  "description": "Codelab 설명",
  "author": "작성자"
}
```

### Codelab Import

`POST /codelabs/import`

**Request**: `multipart/form-data`
- `file`: ZIP 파일

**Response** (201 Created):
```json
{
  "id": "codelab_imported",
  "title": "Import된 Codelab",
  ...
}
```

**Errors**:
- `400 Bad Request`: 잘못된 파일 형식
- `500 Internal Server Error`: Import 실패

## Attendees

### 참가자 등록

`POST /codelabs/:id/register`

**Request Body**:
```json
{
  "name": "홍길동",
  "code": "ATTEND2024"
}
```

**Response** (201 Created):
```json
{
  "id": "attendee_xxx",
  "codelab_id": "codelab_xxx",
  "name": "홍길동",
  "code": "ATTEND2024",
  "current_step": 1,
  "created_at": "2024-12-27T10:10:00"
}
```

**Errors**:
- `409 Conflict`: 이름 중복
- `400 Bad Request`: 필수 필드 누락

### 참가자 목록 조회

`GET /codelabs/:id/attendees`

**Response** (200 OK):
```json
[
  {
    "id": "attendee_xxx",
    "codelab_id": "codelab_xxx",
    "name": "홍길동",
    "code": "ATTEND2024",
    "current_step": 2,
    "created_at": "2024-12-27T10:10:00"
  }
]
```

## Help Requests

### 도움 요청 생성

`POST /codelabs/:id/help`

**Headers**:
- `X-Attendee-ID`: Attendee ID

**Request Body**:
```json
{
  "step_number": 3
}
```

**Response** (201 Created):
```json
{
  "id": "help_xxx",
  "codelab_id": "codelab_xxx",
  "attendee_id": "attendee_xxx",
  "attendee_name": "홍길동",
  "step_number": 3,
  "status": "pending",
  "created_at": "2024-12-27T10:15:00"
}
```

### 도움 요청 목록 조회

`GET /codelabs/:id/help`

**Query Parameters**:
- `status` (optional): `pending` 또는 `resolved`

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

**Response** (200 OK)

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

**Response** (201 Created)

### 피드백 조회

`GET /codelabs/:id/feedback`

**Response** (200 OK):
```json
[
  {
    "id": "feedback_xxx",
    "codelab_id": "codelab_xxx",
    "difficulty": "3",
    "satisfaction": "5",
    "comment": "매우 유익했습니다!",
    "created_at": "2024-12-27T12:00:00"
  }
]
```

## Chat

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
    "created_at": "2024-12-27T10:20:00"
  },
  {
    "id": "dm_xxx",
    "codelab_id": "codelab_xxx",
    "sender_name": "진행자",
    "message": "잘 하고 계십니다!",
    "msg_type": "dm",
    "target_id": "attendee_xxx",
    "created_at": "2024-12-27T10:25:00"
  }
]
```

## Upload

### 이미지 업로드

`POST /upload/image`

**Request**: `multipart/form-data`
- `file`: 이미지 파일 (PNG, JPG, GIF, WebP)
- 최대 크기: 10MB

**Response** (200 OK):
```json
{
  "url": "http://localhost:8080/assets/images/abc123.png"
}
```

**Errors**:
- `400 Bad Request`: 지원하지 않는 파일 형식
- `413 Payload Too Large`: 파일 크기 초과

## WebSocket

### 연결

`WS /api/ws/:id`

**Path Parameters**:
- `id`: Codelab ID

**연결 후 메시지 전송**:

```json
{
  "type": "join",
  "attendee_id": "attendee_xxx",
  "name": "홍길동"
}
```

### 메시지 타입

#### 전체 채팅

```json
{
  "type": "chat",
  "message": "안녕하세요!"
}
```

#### 1:1 DM

```json
{
  "type": "dm",
  "target_id": "attendee_xxx",
  "message": "개인 메시지"
}
```

#### 진행 상황 업데이트

```json
{
  "type": "progress",
  "attendee_id": "attendee_xxx",
  "step": 3
}
```

### 서버에서 받는 메시지

#### 채팅 메시지

```json
{
  "type": "chat",
  "sender": "홍길동",
  "message": "안녕하세요!",
  "timestamp": "2024-12-27T10:30:00Z"
}
```

#### 진행 상황 브로드캐스트

```json
{
  "type": "progress_update",
  "attendee_id": "attendee_xxx",
  "name": "홍길동",
  "step": 3
}
```

## 에러 응답

모든 에러는 다음 형식을 따릅니다:

```json
{
  "error": "에러 메시지"
}
```

### HTTP 상태 코드

| 코드 | 의미 | 예시 |
|------|------|------|
| `200` | 성공 | 조회, 수정 성공 |
| `201` | 생성됨 | 리소스 생성 성공 |
| `204` | 내용 없음 | 삭제 성공 |
| `400` | 잘못된 요청 | 필수 필드 누락 |
| `401` | 인증 실패 | 로그인 필요 |
| `404` | 없음 | 리소스를 찾을 수 없음 |
| `409` | 충돌 | 중복된 이름 |
| `413` | 페이로드 너무 큼 | 파일 크기 초과 |
| `500` | 서버 에러 | 내부 서버 오류 |

## 사용 예제

### cURL

```bash
# Codelab 목록 조회
curl http://localhost:8080/api/codelabs

# Codelab 생성
curl -X POST http://localhost:8080/api/codelabs \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","description":"Desc","author":"Me"}'

# 참가자 등록
curl -X POST http://localhost:8080/api/codelabs/codelab_xxx/register \
  -H "Content-Type: application/json" \
  -d '{"name":"홍길동","code":"ATTEND2024"}'

# 이미지 업로드
curl -X POST http://localhost:8080/api/upload/image \
  -F "file=@image.png"
```

### JavaScript (Fetch)

```javascript
// Codelab 목록 조회
const codelabs = await fetch('http://localhost:8080/api/codelabs')
  .then(res => res.json());

// Codelab 생성
const newCodelab = await fetch('http://localhost:8080/api/codelabs', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    title: 'New Codelab',
    description: 'Description',
    author: 'Author'
  })
}).then(res => res.json());

// WebSocket 연결
const ws = new WebSocket('ws://localhost:8080/api/ws/codelab_xxx');

ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'join',
    attendee_id: 'attendee_xxx',
    name: '홍길동'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};
```

### Rust (reqwest)

```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Codelab 생성
    let res = client
        .post("http://localhost:8080/api/codelabs")
        .json(&json!({
            "title": "New Codelab",
            "description": "Description",
            "author": "Author"
        }))
        .send()
        .await?;

    let codelab: serde_json::Value = res.json().await?;
    println!("{:#?}", codelab);

    Ok(())
}
```

## 다음 단계

- [데이터베이스 스키마](database-schema.md) - DB 구조
- [아키텍처](../architecture/backend.md) - Backend 구조
- [코드 예제](../code-guide/api-usage.md) - 실제 사용 예시
