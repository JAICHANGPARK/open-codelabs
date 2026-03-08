# 2026-03-08 oc MCP MVP expansion

## 목적

- 초기 `oc mcp serve` MVP가 connection/codelab 기본 조회와 일부 admin action만 다루던 상태에서, 실제 운영/작성 워크플로우에 필요한 표면을 보강한다.
- 이미 `backend/src/cli/client.rs`에 있는 기능을 중심으로 MCP tools/resources를 확장해 AI host가 더 적은 왕복으로 작업할 수 있게 한다.

## 확장 범위

- codelab reference, copy, delete 추가
- materials 업로드/추가/삭제 및 목록 조회 추가
- quizzes 조회/갱신 및 quiz submissions 조회 추가
- feedback, learner submissions, chat history resource 추가
- workspace metadata/branch/folder 조회와 파일 읽기 도구 추가
- combined bundle resource 추가

## 새 MCP tools

- `get_codelab_reference`
- `copy_codelab`
- `delete_codelab`
- `list_materials`
- `upload_material_asset`
- `add_material`
- `delete_material`
- `list_quizzes`
- `update_quizzes`
- `list_feedback`
- `list_submissions`
- `list_quiz_submissions`
- `get_workspace_info`
- `list_workspace_branches`
- `list_workspace_folders`
- `list_workspace_branch_files`
- `read_workspace_branch_file`
- `list_workspace_folder_files`
- `read_workspace_folder_file`

## 새 MCP resources

- `oc://reference`
- `oc://codelabs/{id}/bundle`
- `oc://codelabs/{id}/materials`
- `oc://codelabs/{id}/quizzes`
- `oc://codelabs/{id}/quiz-submissions`
- `oc://codelabs/{id}/feedback`
- `oc://codelabs/{id}/submissions`
- `oc://codelabs/{id}/chat`
- `oc://codelabs/{id}/workspace`
- `oc://codelabs/{id}/workspace/branches`
- `oc://codelabs/{id}/workspace/folders`

## 구현 메모

- workspace file read 도구를 위해 `client.rs`의 query serializer 스코프를 줄여 future가 `Send`를 만족하도록 정리했다.
- admin-only read surface는 명시적으로 `require_admin_session()`을 통과해야 한다.
- `bundle` resource는 codelab metadata, steps, materials, quizzes를 한 번에 읽는 빠른 컨텍스트 공급용이다.

## 문서 반영

- MCP 가이드의 tool/resource 목록을 현재 구현에 맞게 확장
- CLI 레퍼런스의 `oc mcp serve` 설명을 최신화

## 검증

- `cd backend && cargo check --bin oc`
- `cd backend && cargo test mcp --lib`
- `cd backend && cargo run --bin oc -- --help`
- `git diff --check`
