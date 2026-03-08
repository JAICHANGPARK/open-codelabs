# MCP 서버

`oc mcp serve`는 현재 `oc` profile과 session을 재사용해서 Open Codelabs를 stdio 기반 MCP 서버로 노출합니다. 별도 토큰 발급이나 별도 설정 파일 없이, 이미 `oc connect`와 `oc auth login`으로 만든 연결 상태를 그대로 AI 클라이언트에 붙일 수 있습니다.

## 언제 쓰는가

- Claude Desktop, Codex, Cursor 같은 MCP host에서 Open Codelabs 데이터를 읽고 싶을 때
- codelab 메타데이터, guide, steps를 AI에게 바로 넘기고 싶을 때
- 관리자 세션으로 codelab 생성/수정, help request 조회/해결까지 연결하고 싶을 때

## 시작 전 준비

최소 준비는 아래 둘입니다.

```bash
oc connect add --name local --url http://localhost:8080 --runtime backend --activate
oc auth login
```

- 읽기 전용 public codelab만 다룰 거면 `oc auth login` 없이도 일부 도구와 리소스는 동작할 수 있습니다.
- `create_codelab`, `update_codelab`, `copy_codelab`, `delete_codelab`, `replace_codelab_steps`, material/quiz/workspace 계열 도구, `list_attendees`, `list_help_requests`, `resolve_help_request`는 관리자 세션이 필요합니다.

## 서버 실행

```bash
oc mcp serve
```

이 명령은 stdio transport를 사용합니다. 즉, 터미널에 직접 사람이 붙어서 쓰는 명령이 아니라 MCP host가 child process로 실행하는 형태를 전제로 합니다.

명령 자체의 전용 옵션은 없고, 아래 전역 옵션을 그대로 사용할 수 있습니다.

- `oc --profile workshop-prod mcp serve`
- `oc --base-url https://labs.example.com mcp serve`
- `oc --session-file /tmp/oc-admin-session.json mcp serve`

## 노출되는 tools

현재 MVP에서 제공하는 도구는 아래와 같습니다.

| Tool | 의미 | 권한 |
| --- | --- | --- |
| `get_connection` | 현재 profile, base URL, runtime probe, session 상태를 반환합니다. | 누구나 |
| `get_codelab_reference` | 내장 reference payload를 반환합니다. | 누구나 |
| `list_codelabs` | 현재 세션에서 볼 수 있는 codelab 목록을 반환합니다. | 누구나 |
| `get_codelab` | 특정 codelab의 metadata, guide markdown, ordered steps를 반환합니다. | 누구나 |
| `get_codelab_bundle` | metadata, guide, steps, materials, quizzes를 한 번에 반환합니다. | 관리자 |
| `create_codelab` | 새 codelab을 생성합니다. | 관리자 |
| `update_codelab` | 기존 codelab metadata를 수정합니다. | 관리자 |
| `copy_codelab` | 기존 codelab을 복제합니다. | 관리자 |
| `delete_codelab` | 기존 codelab을 삭제합니다. | 관리자 |
| `replace_codelab_steps` | codelab step 전체를 교체합니다. | 관리자 |
| `list_materials` | codelab materials를 반환합니다. | 관리자 |
| `upload_material_asset` | 로컬 파일을 업로드하고 material asset URL을 반환합니다. | 관리자 |
| `add_material` | codelab에 link/file material 레코드를 추가합니다. | 관리자 |
| `delete_material` | material 레코드를 삭제합니다. | 관리자 |
| `list_quizzes` | codelab quiz 정의를 반환합니다. | 관리자 |
| `update_quizzes` | codelab quiz 세트를 전체 교체합니다. | 관리자 |
| `list_feedback` | attendee feedback를 반환합니다. | 관리자 |
| `list_submissions` | learner submission 목록을 반환합니다. | 관리자 |
| `list_quiz_submissions` | quiz submission 목록을 반환합니다. | 관리자 |
| `get_chat_history` | 저장된 codelab chat history를 반환합니다. | 관리자 |
| `list_attendees` | codelab 참석자 목록을 반환합니다. | 관리자 |
| `list_help_requests` | codelab help queue를 반환합니다. | 관리자 |
| `resolve_help_request` | help request 하나를 해결 처리합니다. | 관리자 |
| `get_workspace_info` | codelab workspace metadata를 반환합니다. | 관리자 |
| `list_workspace_branches` | branch snapshot 이름 목록을 반환합니다. | 관리자 |
| `list_workspace_folders` | folder snapshot 이름 목록을 반환합니다. | 관리자 |
| `list_workspace_branch_files` | 특정 branch snapshot 안의 파일 목록을 반환합니다. | 관리자 |
| `read_workspace_branch_file` | branch snapshot 안의 파일 내용을 반환합니다. | 관리자 |
| `list_workspace_folder_files` | 특정 folder snapshot 안의 파일 목록을 반환합니다. | 관리자 |
| `read_workspace_folder_file` | folder snapshot 안의 파일 내용을 반환합니다. | 관리자 |

## 노출되는 resources

| Resource URI | 의미 |
| --- | --- |
| `oc://connection` | 현재 연결 상태와 runtime probe 결과 |
| `oc://session` | 현재 session subject, role, session file |
| `oc://reference` | 내장 reference payload |
| `oc://codelabs` | 현재 세션에서 볼 수 있는 codelab 목록 |
| `oc://codelabs/{id}` | codelab metadata |
| `oc://codelabs/{id}/bundle` | metadata, guide, steps, materials, quizzes를 합친 bundle |
| `oc://codelabs/{id}/guide` | guide markdown |
| `oc://codelabs/{id}/steps` | ordered steps |
| `oc://codelabs/{id}/attendees` | attendee 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/help` | help request 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/materials` | material 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/quizzes` | quiz 정의 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/quiz-submissions` | quiz submission 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/feedback` | feedback 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/submissions` | learner submission 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/chat` | 저장된 chat history, 관리자 세션 필요 |
| `oc://codelabs/{id}/workspace` | workspace metadata, 관리자 세션 필요 |
| `oc://codelabs/{id}/workspace/branches` | workspace branch snapshot 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/workspace/folders` | workspace folder snapshot 목록, 관리자 세션 필요 |

리소스 목록은 `oc://codelabs`를 기준으로 동적으로 확장됩니다. 즉 현재 보이는 codelab마다 detail, guide, steps 리소스가 같이 노출됩니다.

## Claude Desktop 예시

`oc`가 PATH에 있다면 아래처럼 등록할 수 있습니다.

```json
{
  "mcpServers": {
    "open-codelabs": {
      "command": "oc",
      "args": ["mcp", "serve"]
    }
  }
}
```

특정 profile을 고정하고 싶다면 전역 옵션을 앞에 둡니다.

```json
{
  "mcpServers": {
    "open-codelabs-prod": {
      "command": "oc",
      "args": ["--profile", "prod", "mcp", "serve"]
    }
  }
}
```

## 운영 팁

- 먼저 `oc://connection`을 읽게 하면 AI가 현재 서버와 권한 상태를 빠르게 이해할 수 있습니다.
- 전체 작성 맥락이 필요하면 `oc://codelabs/{id}/bundle`, 일부만 필요하면 `guide`, `steps`, `materials`, `quizzes` 리소스를 개별로 읽는 편이 좋습니다.
- 관리자 write tool을 쓰려면 `oc auth login`으로 같은 profile의 세션을 먼저 갱신해두는 편이 안전합니다.
- 세션을 분리하고 싶다면 `--session-file`로 MCP 전용 세션 파일을 따로 둘 수 있습니다.
