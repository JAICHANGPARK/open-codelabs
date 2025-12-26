# 실시간 통신 (WebSocket)

WebSocket 기반 실시간 통신 아키텍처를 설명합니다.

## WebSocket 연결

```
Client                    Server
  │                         │
  │──── WS Connect ────────>│
  │<──── Connected ─────────│
  │                         │
  │──── Join Message ──────>│
  │<──── Welcome ───────────│
  │                         │
  │──── Chat Message ──────>│
  │<──── Broadcast ─────────│
  │                         │
```

## 메시지 타입

### 클라이언트 → 서버
- `join`: 연결 등록
- `chat`: 전체 채팅
- `dm`: 1:1 메시지
- `progress`: 진행 상황

### 서버 → 클라이언트
- `chat`: 채팅 메시지
- `dm`: DM 메시지
- `progress_update`: 진행 상황
- `help_request`: 도움 요청

## 구현

Backend: Axum WebSocket
Frontend: Native WebSocket API

## 다음 단계

- [Backend 구조](backend.md)
- [Frontend 구조](frontend.md)
