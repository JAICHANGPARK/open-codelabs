# 2026-03-07 oc CLI bootstrap

## 작업 범위

- 기존 `oclabs` CLI 진입점을 공용 모듈로 이동했다.
- 신규 바이너리 `oc`를 추가해서 `oc ...` 형태로 같은 명령을 실행할 수 있게 했다.
- 세션 파일 저장 시 Unix 계열 환경에서 `0600` 권한을 적용하도록 보강했다.

## 구현 내용

- `backend/src/cli/app.rs`
  - 기존 CLI 파싱, 실행, 출력 로직을 공용 엔트리포인트로 이동했다.
  - 실행 파일 이름을 기준으로 help usage가 `oc` 또는 `oclabs`로 자연스럽게 표시되도록 바꿨다.
- `backend/src/bin/oc.rs`
  - 새 `oc` 바이너리를 추가했다.
- `backend/src/bin/oclabs.rs`
  - 기존 바이너리는 공용 엔트리포인트를 호출하는 thin wrapper로 단순화했다.
- `backend/src/cli/session.rs`
  - 세션 파일을 쓴 뒤 owner-only 권한으로 제한하도록 수정했다.
- `backend/Cargo.toml`
  - `oc` 바이너리를 Cargo bin target으로 등록했다.

## 확인

- `cargo test --bin oc --no-run`
- `cargo test --bin oclabs --no-run`
- `cargo run --bin oc -- --help`
- `cargo run --bin oclabs -- --help`

## 다음 작업

- profile 기반 `connect` 설정 저장소 추가
- 서버 capability/runtime probing 추가
- 브라우저 기반 `oc auth login` flow 추가
