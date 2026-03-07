# 2026-03-07 oc run local stack

## 배경

- 사용자가 `oc`만 설치한 뒤 `oc run`으로 로컬 Open Codelabs 스택을 바로 띄우고 싶어했다.
- 요구사항에는 `docker`/`podman` 자동 감지, 엔진 미설치 또는 미기동 시 설치/시작 안내, 프론트/백엔드 컨테이너 실행이 포함됐다.

## 구현

- `backend/src/cli/app.rs`에 `oc run` 명령을 추가했다.
- `oc run`은 `docker compose`, `docker-compose`, `podman compose`, `podman-compose`를 감지한다.
- Docker/Podman이 없거나 데몬/머신이 준비되지 않으면 OS별 설치/시작 가이드를 에러 메시지로 출력한다.
- 로컬 런타임 파일은 `~/.open-codelabs/runtime/local-stack/compose.yml`에 생성한다.
- compose 파일은 저장소 checkout 없이도 동작하도록 퍼블리시된 이미지를 직접 참조하는 형태로 CLI가 생성한다.
- 기본 데이터 저장 경로는 `~/open-codelabs`이고, SQLite를 기본으로 사용한다.
- `--postgres`를 주면 번들 PostgreSQL 서비스까지 포함해 실행한다.
- `--pull`, `--engine`, `--open`, `--admin-id`, `--admin-pw`, `--data-dir`, `--frontend-port`, `--backend-port`, `--image-registry`, `--image-namespace`, `--image-tag` 옵션을 지원한다.

## 문서화

- `README.md`에 CLI-first quickstart와 `oc run` 사용 예시를 추가했다.
- `docs/getting-started/installation.md`, `docs/en/getting-started/installation.md`에 `oc run` 설치/실행 흐름을 추가했다.

## 검증

- `cd backend && cargo fmt`
- `cd backend && cargo test --bin oc --no-run`
- `cd backend && cargo test cli::app --lib`
- `cd backend && cargo run --bin oc -- --help`
- `cd backend && cargo run --bin oc -- run --engine docker --json`

## 메모

- 현재 `oc run`은 백그라운드 실행만 지원한다.
- 런타임 compose 파일은 재사용 가능하므로 이후 `oc logs`, `oc down`, `oc ps` 같은 운영 명령으로 자연스럽게 확장할 수 있다.
