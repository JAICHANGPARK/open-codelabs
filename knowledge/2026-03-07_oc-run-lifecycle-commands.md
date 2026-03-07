# 2026-03-07 oc run lifecycle commands

## 배경

- `oc run`으로 스택을 띄운 뒤에도 사용자가 다시 `docker compose`나 `podman compose`로 돌아가야 했다.
- CLI 설치 경험을 완성하려면 조회, 로그, 재시작, 종료까지 `oc` 안에서 이어져야 한다.

## 구현

- `backend/src/cli/app.rs`에 `oc ps`, `oc logs`, `oc restart`, `oc down`을 추가했다.
- `oc run` 성공 시 `~/.open-codelabs/runtime/local-stack/state.json`에 런타임 상태를 저장한다.
- lifecycle 명령은 이 상태 파일을 읽어 같은 compose 파일과 같은 엔진 계열(`docker` 또는 `podman`)을 재사용한다.
- 이전 버전 `oc run`으로 state 파일 없이 `compose.yml`만 있는 경우를 위해 fallback도 넣었다.

## 명령

- `oc ps [--service <name>]`
- `oc logs [--service <name>] [--tail <n>] [--no-follow]`
- `oc restart [--service <name>]`
- `oc down [--volumes]`

## 문서화

- `README.md`
- `docs/getting-started/installation.md`
- `docs/en/getting-started/installation.md`

모두 `oc run` 뒤에 이어서 사용할 lifecycle 예시를 추가했다.

## 검증

- `cd backend && cargo fmt`
- `cd backend && cargo test cli::app --lib`
- `cd backend && cargo test --bin oc --no-run`
- `cd backend && cargo run --bin oc -- --help`
- `cd backend && cargo run --bin oc -- ps --json`

## 메모

- `oc logs`는 기본적으로 follow 모드다. JSON 출력은 `--no-follow`일 때만 허용한다.
- `oc down`은 컨테이너를 내리지만 runtime metadata는 유지한다. 이후 `oc run`이나 `oc ps`가 같은 런타임 정의를 계속 참조할 수 있다.
