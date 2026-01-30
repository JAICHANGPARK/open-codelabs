# Real-time Communication (WebSocket)

This document describes the WebSocket-based real-time communication architecture.

## WebSocket connection

```
Client                    Server
  |                         |
  |---- WS Connect -------->|
  |<--- Connected ----------|
  |                         |
  |---- Join Message ------>|
  |<--- Welcome ------------|
  |                         |
  |---- Chat Message ------>|
  |<--- Broadcast ----------|
  |                         |
```

## Message types

### Client -> server
- `join`: register connection
- `chat`: public chat
- `dm`: 1:1 message
- `progress`: progress update

### Server -> client
- `chat`: chat message
- `dm`: DM message
- `progress_update`: progress update
- `help_request`: help request

## Implementation

Backend: Axum WebSocket
Frontend: Native WebSocket API

## Next steps

- [Backend architecture](backend.md)
- [Frontend architecture](frontend.md)
