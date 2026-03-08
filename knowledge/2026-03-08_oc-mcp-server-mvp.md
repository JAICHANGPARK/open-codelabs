# 2026-03-08 oc MCP server MVP

## 목적

- `oc`가 기존 profile/session을 재사용해서 Open Codelabs를 MCP host에 붙일 수 있게 한다.
- 별도 서버 프로세스나 별도 인증 설계 없이 `oc mcp serve` 하나로 stdio MCP 서버를 제공한다.

## 구현 범위

- `rmcp` 공식 Rust SDK를 backend crate에 추가
- `oc mcp serve` 명령을 실제 stdio MCP 서버로 구현
- 읽기 중심 resources 제공
- 안전한 관리자 도구 일부 제공
- CLI 및 문서에 MCP 사용법 반영

## 추가된 MCP tools

- `get_connection`
- `list_codelabs`
- `get_codelab`
- `create_codelab`
- `update_codelab`
- `replace_codelab_steps`
- `list_attendees`
- `list_help_requests`
- `resolve_help_request`

## 추가된 MCP resources

- `oc://connection`
- `oc://session`
- `oc://codelabs`
- `oc://codelabs/{id}`
- `oc://codelabs/{id}/guide`
- `oc://codelabs/{id}/steps`
- `oc://codelabs/{id}/attendees`
- `oc://codelabs/{id}/help`

## 구현 메모

- `oc mcp serve`는 `oc connect`의 active profile과 저장된 session file을 그대로 사용한다.
- 관리자 세션이 없으면 admin write tool과 admin-only resource는 명시적으로 실패한다.
- resource 목록은 `list_codelabs` 결과를 기준으로 동적으로 확장된다.
- guide는 `text/markdown`, 나머지 structured payload는 `application/json` 텍스트 리소스로 노출한다.

## 문서 반영

- CLI 레퍼런스에 `oc mcp serve` 설명 추가
- 전용 사용자 가이드 `docs/user-guide/mcp.md`, `docs/en/user-guide/mcp.md` 추가
- README / README.ko에 MCP 서버 사용 흐름 추가
- MkDocs nav에 MCP 가이드 추가

## 검증

- `cd backend && cargo fmt`
- `cd backend && cargo check --bin oc`
- `cd backend && cargo test mcp --lib`
- `cd backend && cargo run --bin oc -- --help`
- `git diff --check`
