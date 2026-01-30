# 백엔드 구조

Rust Axum 기반 Backend 아키텍처를 설명합니다.

## 디렉토리 구조

```
backend/
├── src/
│   ├── main.rs              # 엔트리 포인트
│   ├── lib.rs               # 공용 모듈
│   ├── api/                 # 라우팅 및 핸들러
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   └── handlers/
│   │       ├── admin.rs
│   │       ├── ai.rs
│   │       ├── attendees.rs
│   │       ├── audit.rs
│   │       ├── codelabs.rs
│   │       ├── codeserver.rs
│   │       ├── feedback.rs
│   │       ├── materials.rs
│   │       ├── quizzes.rs
│   │       ├── submissions.rs
│   │       ├── upload.rs
│   │       └── websocket.rs
│   ├── domain/              # 도메인 모델/서비스
│   │   ├── mod.rs
│   │   ├── models.rs
│   │   └── services/
│   ├── infrastructure/      # DB, 감사 로그 등 인프라
│   │   ├── mod.rs
│   │   ├── database.rs
│   │   └── audit.rs
│   ├── middleware/          # 인증/보안/레이트리밋
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── request_info.rs
│   │   ├── rate_limit.rs
│   │   └── security.rs
│   └── utils/               # 유틸리티
│       ├── mod.rs
│       ├── crypto.rs
│       ├── error.rs
│       └── validation.rs
├── migrations/              # DB 마이그레이션
├── tests/                   # 통합 테스트
└── Cargo.toml               # 의존성
```

## 핵심 컴포넌트

### 1. Router
HTTP 라우팅 및 미들웨어

### 2. Handlers
비즈니스 로직 처리

### 3. Domain
핵심 모델 및 도메인 서비스

### 4. Infrastructure
DB 연결, 감사 로그 등 인프라 구성

### 5. Middleware
세션 인증, CSRF, 보안 헤더, 레이트 리밋

### 6. WebSocket
실시간 통신

## 다음 단계

- [API 레퍼런스](../specification/api-reference.md)
- [코드 예제](../code-guide/backend-examples.md)
