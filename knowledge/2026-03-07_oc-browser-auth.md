# 2026-03-07 oc browser auth

## 작업 범위

- backend에 CLI browser auth challenge flow를 추가했다.
- `oc auth login`, `oc auth status`, `oc auth logout` 명령군을 추가했다.
- CLI가 브라우저 승인 후 admin session cookie를 교환받아 저장하도록 연결했다.

## 구현 내용

- `backend/migrations/20260307190000_cli_auth_requests.sql`
- `backend/migrations-postgres/20260307190000_cli_auth_requests.sql`
  - browser auth challenge를 저장하는 `cli_auth_requests` 테이블을 추가했다.
- `backend/src/api/dto/cli.rs`
  - auth challenge start, poll, exchange, approve용 DTO를 확장했다.
- `backend/src/api/handlers/cli.rs`
  - `POST /api/cli/auth/start`
  - `GET /api/cli/auth/poll/{id}`
  - `POST /api/cli/auth/exchange/{id}`
  - `POST /api/cli/auth/approve/{id}`
  - `GET /cli/auth/{id}`
  를 구현했다.
  - lightweight HTML approve page와 DB-backed challenge lifecycle을 포함한다.
- `backend/src/api/handlers/admin.rs`
  - admin credential 검증과 session cookie 발급 로직을 재사용 가능 helper로 분리했다.
- `backend/src/cli/client.rs`
  - browser auth start, poll, exchange 호출을 추가했다.
  - exchange 응답에서 Set-Cookie를 읽어 CLI session으로 저장할 수 있게 했다.
- `backend/src/cli/app.rs`
  - `auth login [--no-open]`
  - `auth status`
  - `auth logout`
  를 추가했다.
  - `auth login`은 connect profile/base URL을 기준으로 challenge를 만들고, 브라우저를 열거나 URL을 출력한 뒤 polling 후 session을 저장한다.

## 동작 흐름

1. `oc auth login`
2. backend가 challenge를 만들고 `/cli/auth/{id}` URL을 반환
3. 브라우저에서 기존 admin 세션이 있으면 바로 approve, 없으면 admin id/password로 approve
4. CLI가 poll 상태를 확인
5. 승인되면 `exchange`로 admin session cookie와 csrf cookie를 받아 session file에 저장

## 확인

- `cargo test api::handlers::cli --lib -- --nocapture`
- `cargo test --bin oc --no-run`
- `cargo run --bin oc -- auth status --json`

`api::handlers::cli::tests::browser_auth_flow_round_trips` 테스트로 start -> approve -> poll -> exchange 전체 흐름을 검증했다.

## 주의

- `oc auth login`의 실제 브라우저 열기는 실행 환경의 `open`/`xdg-open`/`rundll32`에 의존한다.
- 현재 browser auth는 backend runtime 전용이다.
- `auth status`는 session file이 있더라도 서버 검증에 실패하면 `authenticated: false`와 에러를 함께 보여준다.

## 다음 작업 후보

- `auth login`에 device-style short code UX 추가
- `auth login` 완료 후 connect/runtime 정보를 더 친절하게 묶어 출력
- public/attendee용 별도 auth 모델 설계
