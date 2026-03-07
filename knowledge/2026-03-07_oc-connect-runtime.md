# 2026-03-07 oc connect runtime

## 작업 범위

- `oc connect` 명령군을 추가했다.
- CLI profile 설정 파일을 도입했다.
- backend runtime probing용 `/api/cli/runtime` 엔드포인트를 추가했다.
- profile 별 세션 경로를 분리했다.

## 구현 내용

- `backend/src/cli/config.rs`
  - profile 저장 포맷과 `~/.open-codelabs/config.json` 기본 경로를 정의했다.
  - `current_profile`, `profiles`, `runtime` 설정을 저장하도록 했다.
  - profile 이름을 세션 경로로 사용할 때 안전하게 정규화하도록 했다.
- `backend/src/cli/app.rs`
  - 전역 옵션에 `--config-file`, `--profile`을 추가했다.
  - `connect add`, `connect use`, `connect list`, `connect status`를 추가했다.
  - 저장된 profile을 기준으로 base URL과 session file을 해석하도록 바꿨다.
  - 전역 옵션을 명령 뒤에 두어도 인식되도록 파서를 보강했다.
- `backend/src/api/dto/cli.rs`
  - runtime/capability probe 응답 DTO를 추가했다.
- `backend/src/api/handlers/cli.rs`
  - backend runtime 정보를 반환하는 `get_cli_runtime` 핸들러를 추가했다.
- `backend/src/api/routes.rs`
  - `/api/cli/runtime` 라우트를 등록했다.
- `backend/src/cli/client.rs`
  - CLI runtime probe 호출을 추가했다.

## 현재 동작

- `oc connect add --name local-dev --url http://localhost:8080 --runtime backend --activate`
- `oc connect list`
- `oc connect status`

`connect status`는 현재 profile, base URL, session file, runtime preference, probe 성공 여부, auth method, capability를 보여준다. backend가 떠 있지 않으면 probe 에러를 함께 출력한다.

## 확인

- `cargo test --lib cli::config -- --nocapture`
- `cargo test --bin oc --no-run`
- `cargo run --bin oc -- connect status --json`
- `cargo run --bin oc -- --config-file /tmp/open-codelabs-cli-connect.json connect add --name local-dev --url http://localhost:8080 --runtime backend --activate`
- `cargo run --bin oc -- --config-file /tmp/open-codelabs-cli-connect.json connect list`
- `cargo run --bin oc -- --config-file /tmp/open-codelabs-cli-connect.json connect status`

## 다음 작업

- backend DB-backed browser auth challenge 추가
- `oc auth login/status/logout` 명령군 추가
- connect/runtime 결과를 auth UX와 직접 연결
