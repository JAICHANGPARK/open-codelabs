# 프로젝트 개요

Open Codelabs의 전체적인 개요와 핵심 개념을 설명합니다.

## 프로젝트 목표

Open Codelabs는 다음 목표를 달성하기 위해 만들어졌습니다:

### 1. 접근성
- **쉬운 설치**: Docker 한 줄로 전체 시스템 실행
- **낮은 진입장벽**: Markdown만 알면 콘텐츠 제작 가능
- **크로스 플랫폼**: 모든 OS에서 동작

### 2. 사용성
- **직관적인 UI**: Google Codelab 스타일의 익숙한 인터페이스
- **실시간 상호작용**: WebSocket 기반 채팅 및 도움 요청
- **진행 상황 추적**: 참가자별 학습 진도 모니터링

### 3. 유연성
- **Local-First**: 인터넷 없이도 사용 가능
- **SaaS-Ready**: 클라우드 배포 가능
- **Import/Export**: 콘텐츠 이식성

### 4. 확장성
- **가벼운 시작**: SQLite로 빠른 시작
- **수평 확장**: 필요시 PostgreSQL 등으로 확장 가능
- **모듈화**: 기능별로 독립적인 구조

## 핵심 개념

### Codelab
단계별 학습 가이드

- **메타데이터**: 제목, 설명, 작성자
- **Step 목록**: 순서가 있는 학습 단계들
- **참가자**: 등록된 학습자 목록

### Step
Codelab의 개별 학습 단계

- **순서**: step_number로 정렬
- **제목**: 단계의 제목
- **콘텐츠**: Markdown 형식의 내용
- **진행 추적**: 참가자별 완료 여부

### Facilitator (진행자)
Codelab을 만들고 관리하는 사람

**권한**:
- Codelab CRUD
- Step 편집
- 참가자 모니터링
- 도움 요청 관리
- 채팅 참여

### Attendee (참가자)
Codelab을 학습하는 사람

**권한**:
- Codelab 조회
- Step 이동
- 도움 요청
- 채팅 참여
- 피드백 제출

### 진행 상황 (Progress)
각 참가자의 학습 진도

- **current_step**: 현재 위치한 Step
- **실시간 동기화**: WebSocket으로 업데이트
- **대시보드 표시**: Facilitator가 모니터링

### 도움 요청 (Help Request)
참가자가 막혔을 때 도움을 요청하는 기능

**상태**:
- `pending`: 대기 중
- `resolved`: 해결됨

**정보**:
- 어떤 Step에서 요청했는지
- 누가 요청했는지
- 언제 요청했는지

### 채팅 (Chat)
실시간 커뮤니케이션

**타입**:
- `chat`: 전체 채팅
- `dm`: 1:1 메시지

## 시스템 구조

### 계층 구조

```
┌─────────────────────────────────────┐
│         Frontend (SvelteKit)        │
│  ┌─────────┐  ┌─────────┐          │
│  │ Admin   │  │Attendee │          │
│  │ Dashboard│  │  View   │          │
│  └─────────┘  └─────────┘          │
└─────────────┬───────────────────────┘
              │ HTTP / WebSocket
┌─────────────┴───────────────────────┐
│         Backend (Axum)              │
│  ┌─────────┐  ┌─────────┐          │
│  │   API   │  │WebSocket│          │
│  │ Handlers│  │ Handler │          │
│  └─────────┘  └─────────┘          │
└─────────────┬───────────────────────┘
              │ SQLx
┌─────────────┴───────────────────────┐
│         Database (SQLite)           │
│  codelabs, steps, attendees, ...   │
└─────────────────────────────────────┘
```

### 데이터 흐름

#### Codelab 생성 흐름

```
Facilitator → POST /api/codelabs
              ↓
         Axum Handler
              ↓
         SQLx Query
              ↓
         SQLite DB
              ↓
         Response ← Codelab JSON
```

#### 실시간 채팅 흐름

```
User A → WebSocket → Backend → WebSocket → User B
                        ↓
                  DashMap (메모리)
                        ↓
                  DB (영속화)
```

## 기술적 결정

### 왜 Rust (Backend)?

**장점**:
- **성능**: 네이티브 속도
- **안전성**: 메모리 안전 보장
- **동시성**: Tokio async runtime
- **타입 안전**: 컴파일 타임 검증

**단점**:
- 학습 곡선
- 컴파일 시간

**결정 이유**:
워크샵/행사에서 수십~수백 명의 동시 접속을 안정적으로 처리하기 위해 성능과 안전성이 중요

### 왜 SvelteKit (Frontend)?

**장점**:
- **경량**: 작은 번들 사이즈
- **빠름**: Virtual DOM 없이 컴파일
- **간단함**: 적은 boilerplate
- **SSR/SSG**: SEO 및 성능

**결정 이유**:
빠른 로딩과 부드러운 UX가 학습 경험에 중요

### 왜 SQLite?

**장점**:
- **Zero-config**: 설치 불필요
- **Portable**: 단일 파일
- **빠름**: 임베디드 데이터베이스
- **충분함**: 수천 명까지 처리 가능

**한계**:
- 동시 쓰기 제한
- 네트워크 접근 불가

**확장 계획**:
필요시 PostgreSQL/MySQL로 마이그레이션 가능 (SQLx 덕분)

### 왜 WebSocket?

**대안**:
- HTTP Long Polling
- Server-Sent Events (SSE)
- WebRTC

**선택 이유**:
- 양방향 실시간 통신 필요
- 낮은 레이턴시
- 표준화되고 안정적

## 비기능적 요구사항

### 성능

| 지표 | 목표 | 현재 |
|------|------|------|
| API 응답 시간 | < 100ms | ~50ms |
| 페이지 로드 | < 2s | ~1s |
| 동시 사용자 | 100+ | 테스트됨 |
| 메모리 사용량 | < 512MB | ~200MB |

### 가용성

- **Uptime**: 99.9% (행사 중)
- **자동 재시작**: Docker/systemd
- **Health Check**: API endpoint

### 보안

- **인증**: 관리자만 로그인
- **권한**: 역할 기반 (Facilitator/Attendee)
- **입력 검증**: 모든 API 입력
- **XSS 방지**: DOMPurify
- **SQL Injection 방지**: SQLx prepared statements

### 접근성

- **반응형**: 모바일/태블릿 지원
- **키보드 네비게이션**: 탭 키 지원
- **스크린 리더**: ARIA 레이블
- **다국어**: i18n 준비 (현재 한국어)

## 제약사항

### 현재 제약

1. **단일 관리자**: 한 명의 Facilitator만 지원
2. **단순 인증**: 비밀번호만 사용
3. **SQLite 한계**: 대규모 동시 쓰기 제한
4. **WebSocket 확장**: 단일 서버만 지원

### 향후 개선 계획

1. **다중 관리자**: 여러 Facilitator 지원
2. **OAuth**: 소셜 로그인
3. **Database**: PostgreSQL 지원
4. **Redis**: WebSocket 메시지 브로커
5. **S3**: 이미지 스토리지

## 라이선스

MIT License - 자유롭게 사용, 수정, 배포 가능

## 다음 단계

- [기능 명세](features.md) - 상세 기능 설명
- [데이터베이스 스키마](database-schema.md) - DB 구조
- [API 레퍼런스](api-reference.md) - REST API 문서
