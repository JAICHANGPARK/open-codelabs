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
- `create_codelab`, `update_codelab`, `replace_codelab_steps`, `list_attendees`, `list_help_requests`, `resolve_help_request`는 관리자 세션이 필요합니다.

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
| `list_codelabs` | 현재 세션에서 볼 수 있는 codelab 목록을 반환합니다. | 누구나 |
| `get_codelab` | 특정 codelab의 metadata, guide markdown, ordered steps를 반환합니다. | 누구나 |
| `create_codelab` | 새 codelab을 생성합니다. | 관리자 |
| `update_codelab` | 기존 codelab metadata를 수정합니다. | 관리자 |
| `replace_codelab_steps` | codelab step 전체를 교체합니다. | 관리자 |
| `list_attendees` | codelab 참석자 목록을 반환합니다. | 관리자 |
| `list_help_requests` | codelab help queue를 반환합니다. | 관리자 |
| `resolve_help_request` | help request 하나를 해결 처리합니다. | 관리자 |

## 노출되는 resources

| Resource URI | 의미 |
| --- | --- |
| `oc://connection` | 현재 연결 상태와 runtime probe 결과 |
| `oc://session` | 현재 session subject, role, session file |
| `oc://codelabs` | 현재 세션에서 볼 수 있는 codelab 목록 |
| `oc://codelabs/{id}` | codelab metadata |
| `oc://codelabs/{id}/guide` | guide markdown |
| `oc://codelabs/{id}/steps` | ordered steps |
| `oc://codelabs/{id}/attendees` | attendee 목록, 관리자 세션 필요 |
| `oc://codelabs/{id}/help` | help request 목록, 관리자 세션 필요 |

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
- guide와 steps는 각각 `oc://codelabs/{id}/guide`, `oc://codelabs/{id}/steps`로 분리돼 있어 필요한 쪽만 컨텍스트로 넣기 좋습니다.
- 관리자 write tool을 쓰려면 `oc auth login`으로 같은 profile의 세션을 먼저 갱신해두는 편이 안전합니다.
- 세션을 분리하고 싶다면 `--session-file`로 MCP 전용 세션 파일을 따로 둘 수 있습니다.
