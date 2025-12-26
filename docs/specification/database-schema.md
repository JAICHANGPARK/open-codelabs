# 데이터베이스 스키마

Open Codelabs의 SQLite 데이터베이스 스키마를 설명합니다.

## ERD (Entity Relationship Diagram)

```
┌────────────┐         ┌──────────┐         ┌────────────┐
│  codelabs  │1───────*│  steps   │         │ attendees  │
│            │         │          │         │            │
│  id (PK)   │         │ id (PK)  │         │  id (PK)   │
│  title     │         │ codelab_id│   ┌────│ codelab_id │
│  description│        │step_number│   │    │  name      │
│  author    │         │  title   │   │    │  code      │
│created_at  │         │ content  │   │    │current_step│
└────────────┘         └──────────┘   │    │created_at  │
      │                                │    └────────────┘
      │                                │          │
      │                                │          │
      │                                │          │1
      │                                │          │
      │1                               │1         │
      │                                │          │
      │                                │          │*
      │                         ┌──────┴──────────┴────┐
      │                         │   help_requests      │
      │                         │                      │
      │                         │  id (PK)             │
      └─────────────────────────│  codelab_id          │
                                │  attendee_id         │
                                │  step_number         │
                                │  status              │
                                │  created_at          │
                                └──────────────────────┘

┌────────────────────┐         ┌─────────────────┐
│  chat_messages     │         │   feedback      │
│                    │         │                 │
│  id (PK)           │         │  id (PK)        │
│  codelab_id (FK)   │         │ codelab_id (FK) │
│  sender_name       │         │  difficulty     │
│  message           │         │  satisfaction   │
│  msg_type          │         │  comment        │
│  target_id         │         │  created_at     │
│  created_at        │         └─────────────────┘
└────────────────────┘
```

## 테이블 설명

### codelabs

Codelab의 메타데이터를 저장합니다.

```sql
CREATE TABLE IF NOT EXISTS codelabs (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    author TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

| 컬럼 | 타입 | 설명 | 제약 |
|------|------|------|------|
| `id` | TEXT | UUID | PRIMARY KEY |
| `title` | TEXT | Codelab 제목 | NOT NULL |
| `description` | TEXT | Codelab 설명 | NOT NULL |
| `author` | TEXT | 작성자 이름 | NOT NULL |
| `created_at` | DATETIME | 생성 시간 | DEFAULT CURRENT_TIMESTAMP |

**인덱스**:
```sql
CREATE INDEX idx_codelabs_created_at ON codelabs(created_at DESC);
```

**예제 데이터**:
```sql
INSERT INTO codelabs (id, title, description, author)
VALUES (
    'codelab_01234567-89ab-cdef-0123-456789abcdef',
    'Rust로 REST API 만들기',
    'Axum 프레임워크로 RESTful API 서버 구축',
    'AntiGravity Team'
);
```

### steps

Codelab의 개별 단계를 저장합니다.

```sql
CREATE TABLE IF NOT EXISTS steps (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);
```

| 컬럼 | 타입 | 설명 | 제약 |
|------|------|------|------|
| `id` | TEXT | UUID | PRIMARY KEY |
| `codelab_id` | TEXT | 속한 Codelab | FOREIGN KEY |
| `step_number` | INTEGER | 단계 순서 (1부터 시작) | NOT NULL |
| `title` | TEXT | Step 제목 | NOT NULL |
| `content_markdown` | TEXT | Markdown 콘텐츠 | NOT NULL |

**인덱스**:
```sql
CREATE INDEX idx_steps_codelab ON steps(codelab_id, step_number);
```

**예제 데이터**:
```sql
INSERT INTO steps (id, codelab_id, step_number, title, content_markdown)
VALUES (
    'step_01234567-89ab-cdef-0123-456789abcdef',
    'codelab_01234567-89ab-cdef-0123-456789abcdef',
    1,
    '프로젝트 설정',
    '# 프로젝트 설정\n\n```bash\ncargo new my-api\n```'
);
```

### attendees

Codelab 참가자 정보를 저장합니다.

```sql
CREATE TABLE IF NOT EXISTS attendees (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    current_step INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);
```

| 컬럼 | 타입 | 설명 | 제약 |
|------|------|------|------|
| `id` | TEXT | UUID | PRIMARY KEY |
| `codelab_id` | TEXT | 참여 중인 Codelab | FOREIGN KEY |
| `name` | TEXT | 참가자 이름 | NOT NULL |
| `code` | TEXT | 참가 코드 | NOT NULL |
| `current_step` | INTEGER | 현재 Step 번호 | DEFAULT 1 |
| `created_at` | DATETIME | 등록 시간 | DEFAULT CURRENT_TIMESTAMP |

**인덱스**:
```sql
CREATE INDEX idx_attendees_codelab ON attendees(codelab_id);
CREATE UNIQUE INDEX idx_attendees_name_codelab ON attendees(codelab_id, name);
```

**제약 조건**:
- 같은 Codelab 내에서 `name`은 중복 불가

**예제 데이터**:
```sql
INSERT INTO attendees (id, codelab_id, name, code, current_step)
VALUES (
    'attendee_01234567-89ab-cdef-0123-456789abcdef',
    'codelab_01234567-89ab-cdef-0123-456789abcdef',
    '홍길동',
    'ATTEND2024',
    2
);
```

### help_requests

참가자의 도움 요청을 저장합니다.

```sql
CREATE TABLE IF NOT EXISTS help_requests (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    attendee_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    status TEXT DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE,
    FOREIGN KEY (attendee_id) REFERENCES attendees(id) ON DELETE CASCADE
);
```

| 컬럼 | 타입 | 설명 | 제약 |
|------|------|------|------|
| `id` | TEXT | UUID | PRIMARY KEY |
| `codelab_id` | TEXT | Codelab ID | FOREIGN KEY |
| `attendee_id` | TEXT | 요청자 ID | FOREIGN KEY |
| `step_number` | INTEGER | 막힌 Step 번호 | NOT NULL |
| `status` | TEXT | 상태 (pending/resolved) | DEFAULT 'pending' |
| `created_at` | DATETIME | 요청 시간 | DEFAULT CURRENT_TIMESTAMP |

**인덱스**:
```sql
CREATE INDEX idx_help_codelab_status ON help_requests(codelab_id, status);
CREATE INDEX idx_help_created ON help_requests(created_at DESC);
```

**예제 데이터**:
```sql
INSERT INTO help_requests (id, codelab_id, attendee_id, step_number, status)
VALUES (
    'help_01234567-89ab-cdef-0123-456789abcdef',
    'codelab_01234567-89ab-cdef-0123-456789abcdef',
    'attendee_01234567-89ab-cdef-0123-456789abcdef',
    3,
    'pending'
);
```

### chat_messages

채팅 메시지를 영속화합니다.

```sql
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    message TEXT NOT NULL,
    msg_type TEXT DEFAULT 'chat',
    target_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);
```

| 컬럼 | 타입 | 설명 | 제약 |
|------|------|------|------|
| `id` | TEXT | UUID | PRIMARY KEY |
| `codelab_id` | TEXT | Codelab ID | FOREIGN KEY |
| `sender_name` | TEXT | 발신자 이름 | NOT NULL |
| `message` | TEXT | 메시지 내용 | NOT NULL |
| `msg_type` | TEXT | 타입 (chat/dm) | DEFAULT 'chat' |
| `target_id` | TEXT | DM 대상 ID (선택) | NULL |
| `created_at` | DATETIME | 전송 시간 | DEFAULT CURRENT_TIMESTAMP |

**인덱스**:
```sql
CREATE INDEX idx_chat_codelab ON chat_messages(codelab_id, created_at DESC);
CREATE INDEX idx_chat_dm ON chat_messages(codelab_id, target_id) WHERE msg_type = 'dm';
```

**예제 데이터**:
```sql
-- 전체 채팅
INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type)
VALUES (
    'chat_01234567-89ab-cdef-0123-456789abcdef',
    'codelab_01234567-89ab-cdef-0123-456789abcdef',
    '진행자',
    '5분 후 다음 섹션으로 넘어갑니다!',
    'chat'
);

-- DM
INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type, target_id)
VALUES (
    'dm_01234567-89ab-cdef-0123-456789abcdef',
    'codelab_01234567-89ab-cdef-0123-456789abcdef',
    '진행자',
    '잘 하고 계십니다!',
    'dm',
    'attendee_01234567-89ab-cdef-0123-456789abcdef'
);
```

### feedback

참가자 피드백을 저장합니다.

```sql
CREATE TABLE IF NOT EXISTS feedback (
    id TEXT PRIMARY KEY NOT NULL,
    codelab_id TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    satisfaction TEXT NOT NULL,
    comment TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (codelab_id) REFERENCES codelabs(id) ON DELETE CASCADE
);
```

| 컬럼 | 타입 | 설명 | 제약 |
|------|------|------|------|
| `id` | TEXT | UUID | PRIMARY KEY |
| `codelab_id` | TEXT | Codelab ID | FOREIGN KEY |
| `difficulty` | TEXT | 난이도 (1-5) | NOT NULL |
| `satisfaction` | TEXT | 만족도 (1-5) | NOT NULL |
| `comment` | TEXT | 의견 (선택) | NULL |
| `created_at` | DATETIME | 제출 시간 | DEFAULT CURRENT_TIMESTAMP |

**인덱스**:
```sql
CREATE INDEX idx_feedback_codelab ON feedback(codelab_id, created_at DESC);
```

**예제 데이터**:
```sql
INSERT INTO feedback (id, codelab_id, difficulty, satisfaction, comment)
VALUES (
    'feedback_01234567-89ab-cdef-0123-456789abcdef',
    'codelab_01234567-89ab-cdef-0123-456789abcdef',
    '3',
    '5',
    '매우 유익했습니다! 감사합니다.'
);
```

## 마이그레이션

### 마이그레이션 파일 구조

```
backend/migrations/
├── 20251226161500_init.sql              # Codelabs, Steps
├── 20251226161600_attendees.sql         # Attendees, Help Requests, Chat
├── 20251226161700_chat_enhancements.sql # Chat 개선
├── 20251227001500_attendee_progress.sql # Progress 추적
└── 20250101000000_create_feedback.sql   # Feedback
```

### 마이그레이션 실행

```bash
# SQLx CLI로 실행
cd backend
sqlx migrate run

# Cargo 빌드 시 자동 실행
cargo run
```

코드에서:

```rust
// main.rs
let pool = SqlitePool::connect(&database_url).await?;

// 마이그레이션 자동 실행
sqlx::migrate!("./migrations").run(&pool).await?;
```

### 마이그레이션 되돌리기

```bash
sqlx migrate revert
```

## 쿼리 예제

### Codelab과 Steps 조회

```rust
// Codelab 조회
let codelab = sqlx::query_as::<_, Codelab>(
    "SELECT * FROM codelabs WHERE id = ?"
)
.bind(codelab_id)
.fetch_one(&pool)
.await?;

// Steps 조회 (순서대로)
let steps = sqlx::query_as::<_, Step>(
    "SELECT * FROM steps WHERE codelab_id = ? ORDER BY step_number"
)
.bind(codelab_id)
.fetch_all(&pool)
.await?;
```

### 참가자 등록 (중복 체크)

```rust
// 이름 중복 확인
let exists = sqlx::query_scalar::<_, bool>(
    "SELECT EXISTS(SELECT 1 FROM attendees WHERE codelab_id = ? AND name = ?)"
)
.bind(&codelab_id)
.bind(&name)
.fetch_one(&pool)
.await?;

if exists {
    return Err(StatusCode::CONFLICT);
}

// 등록
sqlx::query(
    "INSERT INTO attendees (id, codelab_id, name, code) VALUES (?, ?, ?, ?)"
)
.bind(uuid::Uuid::new_v4().to_string())
.bind(&codelab_id)
.bind(&name)
.bind(&code)
.execute(&pool)
.await?;
```

### 도움 요청 목록 (JOIN)

```rust
let help_requests = sqlx::query_as::<_, HelpRequest>(
    r#"
    SELECT
        h.id, h.codelab_id, h.attendee_id,
        a.name as attendee_name,
        h.step_number, h.status, h.created_at
    FROM help_requests h
    JOIN attendees a ON h.attendee_id = a.id
    WHERE h.codelab_id = ? AND h.status = 'pending'
    ORDER BY h.created_at ASC
    "#
)
.bind(codelab_id)
.fetch_all(&pool)
.await?;
```

### 통계 쿼리

```rust
// 평균 진행 상황
let avg_progress = sqlx::query_scalar::<_, f64>(
    "SELECT AVG(current_step) FROM attendees WHERE codelab_id = ?"
)
.bind(codelab_id)
.fetch_one(&pool)
.await?;

// 피드백 통계
let feedback_stats = sqlx::query!(
    r#"
    SELECT
        AVG(CAST(difficulty AS REAL)) as avg_difficulty,
        AVG(CAST(satisfaction AS REAL)) as avg_satisfaction,
        COUNT(*) as total_count
    FROM feedback
    WHERE codelab_id = ?
    "#,
    codelab_id
)
.fetch_one(&pool)
.await?;
```

## 성능 최적화

### 인덱스 전략

```sql
-- 자주 조회되는 컬럼
CREATE INDEX idx_codelabs_created_at ON codelabs(created_at DESC);
CREATE INDEX idx_steps_codelab ON steps(codelab_id, step_number);
CREATE INDEX idx_attendees_codelab ON attendees(codelab_id);

-- 복합 조건
CREATE INDEX idx_help_codelab_status ON help_requests(codelab_id, status);

-- 고유 제약
CREATE UNIQUE INDEX idx_attendees_name_codelab ON attendees(codelab_id, name);
```

### Vacuum 및 Analyze

```bash
# 정기적으로 실행
sqlite3 data/sqlite.db "VACUUM;"
sqlite3 data/sqlite.db "ANALYZE;"
```

### 연결 풀 설정

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

## 백업 및 복구

### 백업

```bash
# SQLite 백업
sqlite3 data/sqlite.db ".backup backup.db"

# 또는 단순 복사 (서버 중지 후)
cp data/sqlite.db backup/sqlite_$(date +%Y%m%d).db
```

### 복구

```bash
# 백업에서 복구
cp backup/sqlite_20241227.db data/sqlite.db

# 특정 테이블만 내보내기
sqlite3 data/sqlite.db ".dump codelabs" > codelabs.sql
```

## 다음 단계

- [API 레퍼런스](api-reference.md) - REST API 문서
- [기능 명세](features.md) - 기능 상세 설명
- [아키텍처](../architecture/system-architecture.md) - 시스템 구조
