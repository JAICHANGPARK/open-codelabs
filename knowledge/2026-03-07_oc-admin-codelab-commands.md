# 2026-03-07 oc admin codelab commands

## 작업 범위

- `admin settings`, `admin updates` 명령을 추가했다.
- `codelab update`, `codelab delete`, `codelab copy` 명령을 추가했다.

## 구현 내용

- `backend/src/cli/client.rs`
  - `update_codelab`, `delete_codelab`, `copy_codelab`
  - `save_admin_settings`, `check_updates`
  - 를 추가했다.
  - 다음 단계에서 사용할 workspace branch/folder file read/update client 메서드도 함께 추가했다.
- `backend/src/cli/app.rs`
  - `admin` 명령군을 추가했다.
  - `admin settings [--gemini-api-key <key>] [--admin-password <pw>]`
  - `admin updates`
  - `codelab update --id <id> ...`
  - `codelab delete --id <id>`
  - `codelab copy --id <id>`
  를 추가했다.
  - `admin settings`는 서버와 같은 암호화 방식으로 Gemini API key를 암호화해서 보낸다.

## 사용 메모

- `admin settings`에서 Gemini API key를 설정하려면 admin password가 필요하다.
  - `--admin-password`
  - 또는 `OPEN_CODELABS_ADMIN_PW`
- key를 비우려면 `--gemini-api-key ""` 형태로 clear 가능하다.

## 확인

- `cargo test --bin oc --no-run`
- `cargo run --bin oc -- --help`
- `cargo run --bin oc -- admin updates --json`

마지막 명령은 session이 없는 환경에서 실행해 인증 에러가 정상적으로 나는 것까지 확인했다.

## 다음 작업

- workspace branch/folder file list/read/update 명령 추가
- attendee/help/materials/quiz/submission/feed/chat 계열 명령 추가
