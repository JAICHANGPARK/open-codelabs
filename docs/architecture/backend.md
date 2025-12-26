# 백엔드 구조

Rust Axum 기반 Backend 아키텍처를 설명합니다.

## 디렉토리 구조

```
backend/
├── src/
│   ├── main.rs              # 엔트리 포인트
│   ├── models.rs            # 데이터 모델
│   ├── state.rs             # 애플리케이션 상태
│   └── handlers/            # API 핸들러
│       ├── mod.rs
│       ├── admin.rs
│       ├── codelabs.rs
│       ├── attendees.rs
│       ├── feedback.rs
│       ├── upload.rs
│       └── websocket.rs
├── migrations/              # DB 마이그레이션
└── Cargo.toml              # 의존성
```

## 핵심 컴포넌트

### 1. Router
HTTP 라우팅 및 미들웨어

### 2. Handlers
비즈니스 로직 처리

### 3. State
공유 애플리케이션 상태

### 4. WebSocket
실시간 통신

## 다음 단계

- [API 레퍼런스](../specification/api-reference.md)
- [코드 예제](../code-guide/backend-examples.md)
