# CLI 레퍼런스

이 문서는 현재 `oc --help`와 CLI 구현 코드를 기준으로 `oc`의 명령어, 옵션, 권한 범위, 입력 파일 의미를 상세히 설명합니다.

빠르게 목록만 보고 싶다면 `oc --help` 또는 `oc <명령군> --help`를 사용하고, 실제로 어떤 옵션을 언제 써야 하는지 이해하려면 이 문서를 기준 레퍼런스로 보면 됩니다.

## 핵심 개념

| 개념 | 설명 |
| --- | --- |
| Profile | `oc connect add`로 저장하는 서버 연결 정보입니다. 이름, base URL, runtime preference를 `~/.open-codelabs/config.json`에 저장합니다. |
| Session | `oc auth login` 또는 `oc attendee join`으로 받은 인증 상태입니다. 쿠키, CSRF, subject, role, expiry가 세션 파일에 저장됩니다. |
| Runtime | 서버 타입 추정값입니다. `backend`는 Rust API를 뜻하고, `firebase`/`supabase`는 제한된 CLI 지원을 전제로 합니다. |
| Local stack | `oc run`으로 띄우는 로컬 frontend/backend 컨테이너 스택입니다. `docker` 또는 `podman`을 사용합니다. |
| JSON output | `--json`을 붙이면 사람이 읽기 쉬운 표/문장 대신 자동화용 JSON을 출력합니다. CI나 스크립트에 적합합니다. |

## 공통 전역 옵션

모든 명령은 아래 전역 옵션을 공통으로 지원합니다.

| 옵션 | 의미 | 언제 쓰는가 |
| --- | --- | --- |
| `--base-url <url>` | 이번 실행에서만 백엔드 기본 URL을 강제로 덮어씁니다. | profile을 저장하지 않고 임시 서버에 붙을 때 사용합니다. |
| `--session-file <path>` | 이번 실행에서 사용할 세션 파일 경로를 바꿉니다. | 여러 역할의 세션을 분리하거나 테스트 세션을 별도로 둘 때 사용합니다. |
| `--config-file <path>` | profile 저장 파일 경로를 바꿉니다. 기본값은 `~/.open-codelabs/config.json`입니다. | 팀별 또는 프로젝트별로 연결 프로필을 분리할 때 사용합니다. |
| `--profile <name>` | 저장된 연결 프로필 중 하나를 강제로 선택합니다. | 현재 기본 profile이 아닌 다른 환경을 잠깐 사용할 때 유용합니다. |
| `--json` | 텍스트 대신 JSON을 출력합니다. | 스크립트, CI, 파이프라인에서 결과를 후처리할 때 사용합니다. |
| `-h`, `--help` | 도움말을 출력합니다. | 특정 명령군의 usage만 보고 싶을 때 사용합니다. |

## 값 해석 우선순위

CLI는 base URL과 session file을 아래 우선순위로 결정합니다.

### Base URL 우선순위

1. `--base-url`
2. `--profile` 또는 현재 활성 profile의 `base_url`
3. 저장된 session의 `base_url`
4. `OPEN_CODELABS_BASE_URL`
5. `http://localhost:8080`

### Session file 우선순위

1. `--session-file`
2. 선택된 profile 전용 세션 파일
3. 기본 세션 파일 `~/.open-codelabs/session.json`

## 환경 변수와 저장 경로

| 항목 | 기본값 | 설명 |
| --- | --- | --- |
| `OPEN_CODELABS_BASE_URL` | 없음 | profile이 없을 때 사용할 기본 서버 URL입니다. |
| `OPEN_CODELABS_PROFILE` | 없음 | 기본으로 사용할 profile 이름입니다. |
| `OPEN_CODELABS_CONFIG_FILE` | `~/.open-codelabs/config.json` | profile 저장 파일 위치를 바꿉니다. |
| `OPEN_CODELABS_SESSION_FILE` | `~/.open-codelabs/session.json` | profile을 쓰지 않을 때 기본 세션 파일 위치를 바꿉니다. |
| `OPEN_CODELABS_ADMIN_ID` | 없음 | 레거시 `oc login`과 `oc run` 기본 관리자 ID에 사용됩니다. |
| `OPEN_CODELABS_ADMIN_PW` | 없음 | 레거시 `oc login`, `oc run`, `oc admin settings` 암호화에 사용됩니다. |
| `~/.open-codelabs/config.json` | CLI 기본값 | 저장된 connection profile 목록입니다. |
| `~/.open-codelabs/profiles/<name>/session.json` | profile 사용 시 | profile별 세션 저장 위치입니다. |
| `~/.open-codelabs/runtime/local-stack/` | `oc run` 실행 시 | local stack compose 파일과 상태 파일이 저장됩니다. |
| `~/.open-codelabs/runtime/public/` | `oc public up` 실행 시 | public tunnel 상태 파일과 로그 파일이 저장됩니다. |

## 인터랙티브 동작

현재 `oc`는 `dialoguer` 기반의 터미널 인터랙션을 지원합니다. 선택형 단계에서는 화살표로 이동하고, 멀티 선택 화면에서는 스페이스로 토글한 뒤 엔터로 다음 단계로 넘어갑니다. 텍스트 값은 입력 필드에서 직접 수정하고, 비밀번호는 숨김 입력으로 처리됩니다.

| 명령 | 인터랙티브 동작 |
| --- | --- |
| `oc init` | 항상 인터랙티브 터미널이 필요합니다. 시작 모드 선택, `oc run` wizard, profile 저장, `oc auth login` 연결까지 한 흐름으로 안내합니다. |
| `oc run` | 옵션 없이 TTY에서 실행하면 local stack wizard로 진입합니다. 엔진 선택 후 startup 옵션과 검토할 설정을 스페이스로 토글합니다. `--interactive`를 붙이면 일부 플래그를 미리 준 상태에서도 wizard를 강제합니다. |
| `oc connect add` | `--interactive`를 붙이거나, TTY에서 `--name`/`--url` 없이 실행하면 URL, profile 이름, runtime, activation을 순서대로 질문합니다. |
| `oc connect use` | TTY에서 `--name` 없이 실행하면 저장된 profile 목록이 뜨고, 화살표로 골라 바로 current profile로 전환할 수 있습니다. |
| `oc auth login` | TTY에서 옵션 없이 실행하면 브라우저를 자동으로 열지, 로그인 URL만 출력할지 선택하게 합니다. |
| `oc login` | `--interactive`를 붙이거나 필수 값이 빠진 상태에서 TTY로 실행하면 관리자 ID와 비밀번호를 질문형으로 받습니다. 비밀번호는 숨김 입력입니다. |
| 나머지 명령 | 일반적으로 비대화형입니다. 필요한 값은 플래그나 입력 파일로 모두 전달해야 합니다. |

## 권한 범위

| 범위 | 세션 획득 방법 | 대표 명령 |
| --- | --- | --- |
| 로컬 런타임 | 인증 불필요 | `oc run`, `oc ps`, `oc logs`, `oc restart`, `oc down` |
| 공개 읽기 | `oc connect`만 필요 | `oc codelab list`, `oc codelab reference`, `oc codelab get` |
| 관리자 | `oc auth login` | `admin`, `backup`, `audit`, `workspace`, `codelab create/update/delete/copy/export/import/push-steps` |
| 참석자 | `oc attendee join` | `help request`, `feedback submit`, `quiz submit`, `submission file/link/delete`, `chat history` |

실제 허용 여부는 최종적으로 서버의 runtime capability와 backend 권한 체크가 결정합니다.

## 권장 시작 흐름

```bash
oc init
```

`oc init`이 가장 빠른 진입점입니다. 내부적으로 아래 두 흐름 중 하나를 구성합니다.

1. 로컬 스택 실행
2. 기존 서버 연결

이후 필요하면 profile 저장, `oc connect status`, `oc auth login`까지 이어집니다.

## 연결과 인증

### `oc init`

```bash
oc init
```

무엇을 하는가:

- 로컬 스택 실행과 기존 서버 연결 중 하나를 선택하게 합니다.
- local stack을 고르면 `oc run` wizard를 호출합니다.
- existing server를 고르면 `oc connect add` wizard를 호출합니다.
- 마지막에 profile 저장 여부와 browser-based admin login 진행 여부를 묻습니다.

옵션:

- 없음

### `oc connect add`

```bash
oc connect add --name <name> --url <url> [--runtime <auto|backend|firebase|supabase>] [--activate] [--interactive]
```

무엇을 하는가:

- 서버 연결 프로필을 저장합니다.
- `--activate`를 주면 저장과 동시에 현재 기본 profile로 설정합니다.
- profile이 하나도 없을 때는 `--activate` 없이도 첫 profile이 current profile이 됩니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--name <name>` | 보통 필요 | profile 이름입니다. config 파일의 key로 사용됩니다. 인터랙티브 모드에서는 자동 제안값이 나옵니다. |
| `--url <url>` | 보통 필요 | backend base URL입니다. 예: `http://localhost:8080` |
| `--runtime <auto|backend|firebase|supabase>` | 선택 | 이 profile이 어떤 런타임인지 힌트를 줍니다. `auto`는 연결 후 probe를 시도합니다. |
| `--activate` | 선택 | 저장한 profile을 즉시 current profile로 만듭니다. |
| `--interactive` | 선택 | profile 값을 질문형으로 입력받습니다. |

runtime 값 의미:

| 값 | 의미 |
| --- | --- |
| `auto` | 먼저 backend probe를 시도하고 실패하면 정적 추정값으로 표시합니다. |
| `backend` | Rust backend 기반 전체 관리자 API를 기대합니다. |
| `firebase` | frontend-managed 인증과 제한된 CLI 기능을 전제로 합니다. |
| `supabase` | frontend-managed 인증과 제한된 CLI 기능을 전제로 합니다. |

### `oc connect use`

```bash
oc connect use [--name <name>]
```

무엇을 하는가:

- 저장된 profile 중 하나를 current profile로 바꿉니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--name <name>` | 선택 | 활성화할 profile 이름입니다. 인터랙티브 터미널에서 생략하면 저장된 profile 목록에서 선택합니다. |

### `oc connect list`

```bash
oc connect list
```

무엇을 하는가:

- 저장된 모든 profile과 현재 current profile 여부를 출력합니다.

옵션:

- 없음

### `oc connect status`

```bash
oc connect status
```

무엇을 하는가:

- 현재 선택된 서버에 대해 base URL, runtime, capability, auth method, probe 성공 여부를 보여줍니다.
- `backend` 런타임이면 `/api/cli/runtime` probe를 시도합니다.
- `firebase`/`supabase`처럼 backend probe가 없는 런타임은 정적 capability만 보여줍니다.

옵션:

- 없음

### `oc auth login`

```bash
oc auth login [--no-open] [--interactive]
```

무엇을 하는가:

- browser-based CLI 인증을 시작합니다.
- 연결된 runtime이 browser auth를 지원해야 합니다.
- 브라우저에서 로그인 승인 후 세션을 세션 파일에 저장합니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--no-open` | 선택 | 브라우저를 자동으로 열지 않고, 인증 URL만 출력합니다. 원격 SSH나 헤드리스 터미널에서 유용합니다. |
| `--interactive` | 선택 | 브라우저 자동 열기와 URL만 출력 중 하나를 선택하는 화면을 강제로 엽니다. TTY에서 옵션 없이 실행해도 같은 선택 화면이 기본으로 표시됩니다. |

### `oc auth logout`

```bash
oc auth logout
```

무엇을 하는가:

- 서버 logout 요청을 보낸 뒤 로컬 세션 파일을 삭제합니다.

옵션:

- 없음

### `oc auth status`

```bash
oc auth status
```

무엇을 하는가:

- 현재 세션 파일이 있는지, 실제 서버 세션이 유효한지, role/subject/expiry가 무엇인지 보여줍니다.

옵션:

- 없음

### 레거시 별칭

#### `oc login`

```bash
oc login [--admin-id <id>] [--admin-pw <pw>] [--interactive]
```

무엇을 하는가:

- 기존 `/api/login` 비밀번호 기반 관리자 로그인을 수행합니다.
- 새 스크립트에서는 `oc auth login`을 권장합니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--admin-id <id>` | 선택 | 관리자 ID입니다. `OPEN_CODELABS_ADMIN_ID`가 있으면 생략 가능합니다. 인터랙티브 입력에서는 기본값으로 제안됩니다. |
| `--admin-pw <pw>` | 선택 | 관리자 비밀번호입니다. `OPEN_CODELABS_ADMIN_PW`가 있으면 생략 가능합니다. 인터랙티브 입력에서는 숨김 상태로 받습니다. |
| `--interactive` | 선택 | 관리자 ID와 비밀번호를 질문형으로 입력받습니다. |

#### `oc logout`

- `oc auth logout`의 별칭입니다.

#### `oc session`

- `oc auth status`의 별칭입니다.

## 공개 노출과 벤치마크

### `oc public up`

```bash
oc public up [--tunnel <ngrok|bore|cloudflare>] [--ngrok|--bore|--cloudflare] [--port <port>] [--log-file <path>] [--no-open]
```

무엇을 하는가:

- 로컬에서 실행 중인 frontend 포트를 외부에 공개하는 tunnel 프로세스를 시작합니다.
- 상태와 PID를 `~/.open-codelabs/runtime/public/state.json`에 저장합니다.
- 기본값은 `ngrok`이며, 포트를 생략하면 최근 local stack 상태에서 frontend 포트를 추론하고 없으면 `5173`을 사용합니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--tunnel <ngrok|bore|cloudflare>` | 선택 | 사용할 공개 tunnel 종류를 명시합니다. |
| `--ngrok`, `--bore`, `--cloudflare` | 선택 | `--tunnel`의 단축 플래그입니다. 마지막에 지정한 값이 우선합니다. |
| `--port <port>` | 선택 | 공개할 로컬 포트입니다. |
| `--log-file <path>` | 선택 | tunnel stdout/stderr를 저장할 로그 파일 경로입니다. |
| `--no-open` | 선택 | attendee URL을 브라우저로 자동으로 열지 않습니다. |

주의할 점:

- 같은 시점에 tunnel 하나만 관리합니다. 이미 살아 있는 tunnel이 있으면 먼저 `oc public down`을 실행해야 합니다.
- `ngrok`, `bore`, `cloudflared` 바이너리 설치 여부는 실행 전에 검사합니다.

### `oc public status`

```bash
oc public status
```

무엇을 하는가:

- 저장된 public tunnel 상태, PID, process 생존 여부, 마지막으로 확인한 public URL을 출력합니다.

옵션:

- 없음

### `oc public down`

```bash
oc public down
```

무엇을 하는가:

- 저장된 public tunnel PID를 종료하고 상태 파일만 정리합니다. 로그 파일은 보존합니다.

옵션:

- 없음

### `oc bench`

```bash
oc bench <local|ops|ws> [-- <bench options...>]
```

무엇을 하는가:

- 기존 benchmark runner를 `oc` 아래에서 실행합니다.
- 먼저 `oc`와 같은 디렉터리에 함께 설치된 `local_bench`, `ops_bench`, `ws_bench`를 찾고, 없으면 소스 체크아웃에서 `cargo run --release --bin ...`으로 fallback 합니다.

| 위치 인자 | 의미 |
| --- | --- |
| `local` | attendee/help/submission 중심의 API benchmark를 실행합니다. |
| `ops` | upload/backup/workspace 중심의 운영 benchmark를 실행합니다. |
| `ws` | WebSocket 부하 benchmark를 실행합니다. |

옵션 전달 규칙:

- `--` 뒤의 인자는 해당 benchmark binary로 그대로 전달됩니다.
- 예: `oc bench local -- --help`, `oc bench ops -- --profile paper --output bench-results/ops.json`

## 로컬 런타임

### `oc run`

```bash
oc run [--engine <auto|docker|podman>] [--postgres] [--pull] [--open] [--interactive] [--admin-id <id>] [--admin-pw <pw>] [--data-dir <path>] [--frontend-port <port>] [--backend-port <port>] [--image-registry <registry>] [--image-namespace <namespace>] [--image-tag <tag>]
```

무엇을 하는가:

- published frontend/backend 이미지를 이용해 로컬 워크숍 스택을 띄웁니다.
- `docker` 또는 `podman`을 자동 감지합니다.
- compose 파일과 상태 파일을 `~/.open-codelabs/runtime/local-stack/`에 저장합니다.
- 기본 데이터 디렉터리는 `~/open-codelabs` 계열 경로를 사용합니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--engine <auto|docker|podman>` | 선택 | 사용할 컨테이너 엔진을 고릅니다. `auto`는 감지 순서에 따라 결정합니다. |
| `--postgres` | 선택 | SQLite 대신 번들 PostgreSQL 컨테이너를 함께 띄우고 backend 연결도 PostgreSQL로 맞춥니다. |
| `--pull` | 선택 | `up` 전에 이미지를 먼저 pull 합니다. 최신 published 이미지를 받고 싶을 때 사용합니다. |
| `--open` | 선택 | 시작 후 관리자 로그인 URL을 브라우저로 엽니다. |
| `--interactive` | 선택 | 옵션 일부를 이미 넘겼더라도 wizard를 강제로 시작합니다. |
| `--admin-id <id>` | 선택 | local stack의 기본 관리자 ID입니다. |
| `--admin-pw <pw>` | 선택 | local stack의 기본 관리자 비밀번호입니다. |
| `--data-dir <path>` | 선택 | DB, uploads, workspaces, postgres 데이터를 저장할 호스트 경로입니다. |
| `--frontend-port <port>` | 선택 | frontend가 노출될 호스트 포트입니다. 기본값은 `5173`입니다. |
| `--backend-port <port>` | 선택 | backend가 노출될 호스트 포트입니다. 기본값은 `8080`입니다. |
| `--image-registry <registry>` | 선택 | 이미지 레지스트리 호스트를 바꿉니다. 예: `ghcr.io` |
| `--image-namespace <namespace>` | 선택 | 이미지 네임스페이스를 바꿉니다. 예: `jaichangpark` |
| `--image-tag <tag>` | 선택 | 사용할 이미지 태그입니다. 기본값은 `latest`입니다. |

주의할 점:

- 옵션 없이 TTY에서 실행하면 질문형 wizard로 들어갑니다.
- 관리자 비밀번호를 직접 입력했더라도 출력에는 실제 비밀번호가 아니라 힌트만 표시됩니다.

### `oc ps`

```bash
oc ps [--service <name>]
```

무엇을 하는가:

- local stack의 컨테이너 상태를 보여줍니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--service <name>` | 선택 | 특정 서비스만 보려면 사용합니다. 보통 `frontend`, `backend`, `postgres` 중 하나입니다. |

### `oc logs`

```bash
oc logs [--service <name>] [--tail <n>] [--no-follow]
```

무엇을 하는가:

- local stack 로그를 보여줍니다.
- 기본값은 follow 모드입니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--service <name>` | 선택 | 특정 서비스 로그만 봅니다. |
| `--tail <n>` | 선택 | 마지막 `n`줄만 출력합니다. |
| `--no-follow` | 선택 | 스트리밍 대신 현재 시점 로그만 출력합니다. |

주의할 점:

- `--json`은 `--no-follow`일 때만 쓸 수 있습니다.
- follow 모드에서는 로그 스트림을 붙잡고 종료할 때까지 계속 출력합니다.

### `oc restart`

```bash
oc restart [--service <name>]
```

무엇을 하는가:

- local stack 전체 또는 특정 서비스를 재시작합니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--service <name>` | 선택 | 특정 서비스만 재시작합니다. 생략하면 전체 stack 재시작입니다. |

### `oc down`

```bash
oc down [--volumes]
```

무엇을 하는가:

- local stack을 종료합니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--volumes` | 선택 | 컨테이너 종료와 함께 `oc run`이 만든 로컬 데이터 디렉터리도 삭제합니다. |

주의할 점:

- `--volumes`를 쓰면 bind mount 데이터까지 지워지므로 실질적인 데이터 초기화에 해당합니다.
- `oc ps`, `oc logs`, `oc restart`, `oc down`은 이전에 `oc run`이 저장한 local stack state를 사용합니다.

## 관리자 명령

### `oc admin settings`

```bash
oc admin settings [--gemini-api-key <key>] [--admin-password <pw>]
```

무엇을 하는가:

- 관리자 설정에 저장된 Gemini API 키를 갱신합니다.
- 전달된 API 키는 CLI에서 관리자 비밀번호로 암호화한 뒤 서버에 저장합니다.

| 옵션 | 필수 | 의미 |
| --- | --- | --- |
| `--gemini-api-key <key>` | 보통 예 | 저장할 Gemini API 키 원문입니다. |
| `--admin-password <pw>` | 조건부 | `--gemini-api-key`를 암호화할 관리자 비밀번호입니다. 없으면 `OPEN_CODELABS_ADMIN_PW`를 찾습니다. |

주의할 점:

- `oc admin settings`를 `--gemini-api-key` 없이 실행하면 현재 저장된 Gemini 키를 지우는 동작이 됩니다.

### `oc admin updates`

```bash
oc admin updates
```

무엇을 하는가:

- frontend/backend 배포 버전과 최신 버전을 비교해 업데이트 가능 여부를 보여줍니다.

옵션:

- 없음

## 코드랩 관리

### 목록 조회 계열

| 명령 | 의미 | 옵션 |
| --- | --- | --- |
| `oc codelab list` | 현재 세션에서 볼 수 있는 코드랩 목록을 조회합니다. | 없음 |
| `oc codelab reference` | 내장 reference codelab 원문을 출력합니다. | 없음 |
| `oc codelab get --id <id>` | 코드랩 메타데이터와 step 목록을 함께 조회합니다. | `--id`: 조회할 코드랩 ID |

### `oc codelab create`

```bash
oc codelab create --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
```

### `oc codelab update`

```bash
oc codelab update --id <id> --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
```

공통 옵션 의미:

| 옵션 | `create` | `update` | 의미 |
| --- | --- | --- | --- |
| `--id <id>` | 아니오 | 예 | 수정할 코드랩 ID입니다. |
| `--title <title>` | 예 | 예 | 코드랩 제목입니다. |
| `--description <desc>` | 예 | 예 | 목록과 상세에서 보이는 설명입니다. |
| `--author <author>` | 예 | 예 | 작성자 이름입니다. |
| `--private` | 선택 | 선택 | public 기본값을 뒤집어 private으로 만듭니다. |
| `--guide-file <path>` | 선택 | 선택 | guide markdown 파일을 읽어 guide 본문으로 저장합니다. |
| `--quiz-enabled` | 선택 | 선택 | 퀴즈 기능 자체를 켭니다. |
| `--require-quiz` | 선택 | 선택 | 수료 조건에 퀴즈 제출/통과를 요구합니다. |
| `--require-feedback` | 선택 | 선택 | 수료 조건에 피드백 제출을 요구합니다. |
| `--require-submission` | 선택 | 선택 | 수료 조건에 제출물을 요구합니다. |

주의할 점:

- `update`는 부분 patch가 아니라 전달한 값 기준으로 메타데이터를 다시 씁니다.
- `update`에서 `--private`, `--quiz-enabled`, `--require-*`를 생략하면 해당 값이 기본값으로 돌아갈 수 있으므로 현재 설정을 유지하려면 필요한 플래그를 다시 명시하는 것이 안전합니다.

### 기타 코드랩 명령

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc codelab delete --id <id>` | 코드랩과 관련 데이터를 삭제합니다. | `--id`: 삭제할 코드랩 ID |
| `oc codelab copy --id <id>` | 기존 코드랩과 step 구성을 복제합니다. | `--id`: 복제 원본 코드랩 ID |
| `oc codelab export --id <id> [--output <path>]` | 코드랩 ZIP 백업을 만듭니다. | `--output` 생략 시 `codelab_<id>.zip` |
| `oc codelab import --file <zip>` | export로 만든 ZIP에서 코드랩을 가져옵니다. | `--file`: 가져올 ZIP 경로 |
| `oc codelab pull --id <id> [--output <dir>] [--format <yaml|json>]` | 코드랩 메타데이터, guide, steps, quizzes, materials를 로컬 manifest 번들로 내려받습니다. | `--output` 생략 시 `codelab-<id>` 디렉터리, `--format`은 `codelab.yaml` 또는 `codelab.json` 형식을 고릅니다. |
| `oc codelab push --manifest <path> [--id <id>]` | manifest 번들의 메타데이터, guide, steps, quizzes, materials를 서버에 동기화합니다. | `--manifest`: manifest 파일 또는 manifest가 들어 있는 디렉터리, `--id`: manifest 안의 ID 대신 강제로 대상 코드랩 지정 |
| `oc codelab push-steps --id <id> --file <json>` | 코드랩의 step 목록 전체를 JSON으로 교체합니다. | `--file`: `UpdateStepsPayload` JSON 경로 |

## 백업과 감사 로그

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc backup export [--output <path>]` | 전체 플랫폼 백업 ZIP을 생성합니다. | `--output` 생략 시 `backup_full.zip` |
| `oc backup inspect --file <zip>` | ZIP을 복구하지 않고 내부 개수 요약만 확인합니다. | `--file`: 검사할 backup ZIP |
| `oc backup restore --file <zip>` | backup ZIP을 현재 backend에 복구합니다. | `--file`: 복구할 backup ZIP |
| `oc audit logs [--limit <n>] [--offset <n>] [--action <name>] [--codelab-id <id>]` | 관리자 감사 로그를 필터링해 조회합니다. | `--limit`: 개수 제한, `--offset`: 페이지 오프셋, `--action`: 액션 이름 필터, `--codelab-id`: 특정 코드랩 관련 로그만 조회 |

## 워크스페이스

워크스페이스 명령은 code-server 기반 파일 구조를 관리할 때 사용합니다.

### 생성과 조회

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc workspace create --codelab-id <id> [--structure-type <branch|folder>] [--files-json <path>]` | 코드랩용 워크스페이스를 생성합니다. | `--structure-type`: snapshot 전략 선택, `--files-json`: 초기 파일 목록 JSON |
| `oc workspace info --codelab-id <id>` | 워크스페이스 메타데이터를 조회합니다. | `--codelab-id`: 대상 코드랩 |
| `oc workspace download --codelab-id <id> [--output <path>]` | 워크스페이스를 tar.gz로 다운로드합니다. | `--output` 생략 시 `workspace_<id>.tar.gz` |
| `oc workspace delete --codelab-id <id>` | 워크스페이스를 삭제합니다. | `--codelab-id`: 대상 코드랩 |

### 브랜치 snapshot

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc workspace branches --codelab-id <id>` | 저장된 branch snapshot 이름 목록을 봅니다. | `--codelab-id`: 대상 코드랩 |
| `oc workspace branch-create --codelab-id <id> --step-number <n> [--branch-type <start|end>]` | step 시작 시점 또는 종료 시점 branch snapshot을 만듭니다. | `--step-number`: step 번호, `--branch-type`: `start` 또는 `end`, 기본값 `start` |
| `oc workspace branch-files --codelab-id <id> --branch <name>` | branch 안의 파일 경로 목록을 조회합니다. | `--branch`: 읽을 branch 이름 |
| `oc workspace branch-read --codelab-id <id> --branch <name> --file <path>` | branch 안의 파일 내용을 읽습니다. | `--file`: 읽을 파일 경로 |
| `oc workspace branch-update --codelab-id <id> --branch <name> --files-json <path> [--delete-json <path>] [--commit-message <message>]` | branch 안 파일을 쓰거나 삭제합니다. | `--files-json`: 쓸 파일 목록 또는 update payload, `--delete-json`: 삭제 파일 경로 목록, `--commit-message`: 변경 설명 |

### 폴더 snapshot

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc workspace folders --codelab-id <id>` | 저장된 folder snapshot 이름 목록을 봅니다. | `--codelab-id`: 대상 코드랩 |
| `oc workspace folder-create --codelab-id <id> --step-number <n> --files-json <path> [--folder-type <start|end>]` | step 시작/종료 시점 folder snapshot을 만듭니다. | `--files-json`: snapshot에 들어갈 파일 목록 |
| `oc workspace folder-files --codelab-id <id> --folder <name>` | folder snapshot 안의 파일 목록을 조회합니다. | `--folder`: 읽을 folder 이름 |
| `oc workspace folder-read --codelab-id <id> --folder <name> --file <path>` | folder snapshot 안 파일 내용을 읽습니다. | `--file`: 읽을 파일 경로 |
| `oc workspace folder-update --codelab-id <id> --folder <name> --files-json <path> [--delete-json <path>]` | folder snapshot 안 파일을 갱신합니다. | `--files-json`: 쓸 파일 목록 또는 update payload, `--delete-json`: 삭제할 파일 경로 목록 |

실무 팁:

- `branch` 모드는 Git branch처럼 단계별 상태를 나눠 관리할 때 적합합니다.
- `folder` 모드는 각 step 결과물을 독립 디렉터리처럼 저장할 때 적합합니다.

## 참석자와 라이브 운영

### 참석자 세션

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc attendee join --codelab-id <id> --name <name> --code <code> [--email <email>]` | 참석자로 등록하거나 재입장합니다. 성공 시 attendee 세션을 저장합니다. | `--code`: 참가 코드, `--email`: 선택 메타데이터 |
| `oc attendee list --codelab-id <id>` | 코드랩 참석자 목록을 조회합니다. | `--codelab-id`: 대상 코드랩 |
| `oc attendee complete --codelab-id <id>` | 현재 attendee 세션을 완료 상태로 표시합니다. | `--codelab-id`: 완료 처리할 코드랩 |
| `oc attendee certificate [--attendee-id <id>]` | 수료증 정보를 조회합니다. | `--attendee-id`가 없으면 현재 attendee 세션의 subject를 사용합니다. |

### 도움 요청

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc help request --codelab-id <id> --step-number <n>` | 현재 attendee가 특정 step에서 도움 요청을 생성합니다. | `--step-number`: 막힌 step 번호 |
| `oc help list --codelab-id <id>` | 코드랩의 도움 요청 목록을 조회합니다. | `--codelab-id`: 대상 코드랩 |
| `oc help resolve --codelab-id <id> --help-id <id>` | 도움 요청을 해결 상태로 바꿉니다. | `--help-id`: resolve할 요청 ID |

### 피드백

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc feedback submit --codelab-id <id> --difficulty <1-5> --satisfaction <1-5> [--comment <text>]` | attendee 피드백을 제출합니다. | `--difficulty`, `--satisfaction`: 일반적으로 1-5 점수 문자열, `--comment`: 자유 의견 |
| `oc feedback list --codelab-id <id>` | 코드랩 피드백 목록을 조회합니다. | `--codelab-id`: 대상 코드랩 |

## 자료, 퀴즈, 제출물

### 자료

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc materials list --codelab-id <id>` | 코드랩 자료 목록을 조회합니다. | `--codelab-id`: 대상 코드랩 |
| `oc materials upload --file <path>` | 자료 파일을 업로드하고 나중에 `materials add`에 넣을 file path/URL을 얻습니다. | `--file`: 업로드할 로컬 파일 |
| `oc materials add --codelab-id <id> --title <title> --type <link|file> [--url <url>] [--file-path <path>]` | 자료 레코드를 추가합니다. | `--type link`이면 `--url`, `--type file`이면 `--file-path`를 주는 것이 일반적입니다. |
| `oc materials delete --codelab-id <id> --material-id <id>` | 자료 레코드를 삭제합니다. | `--material-id`: 삭제 대상 자료 ID |

### 퀴즈

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc quiz list --codelab-id <id>` | 현재 퀴즈 정의를 조회합니다. | `--codelab-id`: 대상 코드랩 |
| `oc quiz update --codelab-id <id> --file <json>` | 코드랩의 퀴즈 전체를 JSON으로 교체합니다. | `--file`: `CreateQuiz[]` JSON 파일 |
| `oc quiz submit --codelab-id <id> --file <json>` | 현재 attendee 답안을 제출합니다. | `--file`: `QuizSubmissionPayload` JSON 파일 |
| `oc quiz submissions --codelab-id <id>` | 관리자 관점의 퀴즈 제출 목록을 봅니다. | `--codelab-id`: 대상 코드랩 |

### 제출물

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc submission list --codelab-id <id>` | 현재 역할이 볼 수 있는 제출물 목록을 봅니다. | `--codelab-id`: 대상 코드랩 |
| `oc submission file --codelab-id <id> [--attendee-id <id>] --file <path>` | 파일 제출물을 업로드합니다. | `--attendee-id`가 없으면 현재 attendee 세션을 사용합니다. |
| `oc submission link --codelab-id <id> [--attendee-id <id>] --url <url> [--title <title>]` | 링크 제출물을 생성합니다. | `--title`은 표시용 제목입니다. |
| `oc submission delete --codelab-id <id> [--attendee-id <id>] --submission-id <id>` | 제출물을 삭제합니다. | `--attendee-id`가 없으면 현재 attendee 세션에서 subject를 추론합니다. |

## 채팅, 업로드, 인라인 댓글

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc chat history --codelab-id <id>` | 코드랩 채팅 히스토리를 조회합니다. | `--codelab-id`: 대상 코드랩 |
| `oc upload image --file <path>` | 이미지 에셋을 업로드하고 URL을 반환합니다. | `--file`: 업로드할 이미지 파일 |
| `oc inline list --codelab-id <id> [--target-type <guide|step>] [--target-step-id <id>]` | 인라인 댓글 thread 목록을 조회합니다. | `--target-type step`일 때 `--target-step-id`를 함께 주는 것이 일반적입니다. |
| `oc inline create --codelab-id <id> --file <json>` | 새 인라인 댓글 thread를 만듭니다. | `--file`: `CreateInlineCommentPayload` JSON 파일 |
| `oc inline reply --codelab-id <id> --thread-id <id> --file <json>` | 기존 thread에 댓글을 답글로 추가합니다. | `--file`: `ReplyInlineCommentPayload` JSON 파일 |
| `oc inline delete --codelab-id <id> --thread-id <id> --comment-id <id>` | 특정 댓글 메시지를 삭제합니다. | `--thread-id`: thread ID, `--comment-id`: 삭제할 댓글 ID |

## AI 명령

| 명령 | 의미 | 옵션 설명 |
| --- | --- | --- |
| `oc ai conversations --codelab-id <id>` | 코드랩에 저장된 AI 대화 이력을 조회합니다. | `--codelab-id`: 대상 코드랩 |
| `oc ai stream --file <json>` | AI 요청 JSON을 보내고 최종 SSE 텍스트를 출력합니다. | `--file`: `AiRequest` JSON |
| `oc ai save --file <json>` | 한 번의 AI 대화 교환을 영속 저장합니다. | `--file`: `SaveAiConversationPayload` JSON |
| `oc ai threads` | 현재 관리자 소유 AI thread 목록을 조회합니다. | 없음 |
| `oc ai thread-create --title <title> [--codelab-id <id>]` | 새 AI thread를 만듭니다. | `--codelab-id`: thread를 특정 코드랩에 연결할 때 사용합니다. |
| `oc ai thread-delete --thread-id <id>` | AI thread를 삭제합니다. | `--thread-id`: 삭제 대상 thread ID |
| `oc ai messages --thread-id <id>` | 특정 thread의 메시지를 조회합니다. | `--thread-id`: 조회 대상 thread ID |
| `oc ai message-add --thread-id <id> --file <json>` | 특정 thread에 메시지를 추가합니다. | `--file`: `AddAiMessagePayload` JSON |

## JSON 입력 파일 치트시트

파일 플래그는 단순 “아무 JSON”이 아니라 특정 payload를 기대합니다.

| 명령 옵션 | 기대하는 내용 | 설명 |
| --- | --- | --- |
| `codelab create/update --guide-file` | Markdown 텍스트 파일 | 파일 내용을 그대로 guide markdown으로 읽습니다. |
| `codelab push --manifest` | `CodelabManifest` YAML/JSON | 메타데이터, guide 상대 경로, `steps[]`, `quizzes[]`, `materials[]`를 담는 manifest입니다. file material은 manifest 기준 상대 경로를 사용합니다. |
| `codelab push-steps --file` | `UpdateStepsPayload` JSON | 보통 `steps` 배열 전체를 담습니다. 기존 step 목록을 통째로 교체합니다. |
| `workspace create --files-json` | `WorkspaceFile[]` JSON | 초기 워크스페이스 파일 목록입니다. |
| `workspace branch-update --files-json` | `UpdateWorkspaceFilesRequest` 또는 `WorkspaceFile[]` JSON | 간단한 배열을 주면 write 목록으로 처리합니다. |
| `workspace branch-update --delete-json` | `string[]` JSON | 삭제할 파일 경로 목록입니다. |
| `workspace folder-create --files-json` | `WorkspaceFile[]` JSON | folder snapshot 생성 시 포함할 파일 목록입니다. |
| `workspace folder-update --files-json` | `UpdateWorkspaceFilesRequest` 또는 `WorkspaceFile[]` JSON | folder snapshot에 쓸 파일 목록입니다. |
| `workspace folder-update --delete-json` | `string[]` JSON | folder snapshot에서 삭제할 파일 경로 목록입니다. |
| `quiz update --file` | `CreateQuiz[]` JSON | 전체 퀴즈 집합을 교체합니다. |
| `quiz submit --file` | `QuizSubmissionPayload` JSON | attendee가 제출할 답안 payload입니다. |
| `inline create --file` | `CreateInlineCommentPayload` JSON | 새 thread의 본문, target 정보 등을 담습니다. |
| `inline reply --file` | `ReplyInlineCommentPayload` JSON | 기존 thread에 추가할 답글 내용입니다. |
| `ai stream --file` | `AiRequest` JSON | 모델, 메시지, 컨텍스트 등 AI 호출 입력입니다. |
| `ai save --file` | `SaveAiConversationPayload` JSON | 코드랩에 저장할 AI 대화 단위입니다. |
| `ai message-add --file` | `AddAiMessagePayload` JSON | thread에 추가할 메시지 payload입니다. |

## 운영 팁

- 서버를 아직 모른다면 `oc init`으로 시작하는 것이 가장 안전합니다.
- 자동화에서는 `--json`을 기본으로 두고, 사람이 직접 보는 명령만 텍스트 출력을 쓰는 편이 좋습니다.
- 관리자와 attendee 세션을 동시에 다룬다면 `--session-file`을 분리하거나 profile별 session 경로를 쓰는 것이 안전합니다.
- `firebase`/`supabase` profile은 `connect status`에서 capability가 제한적으로 보일 수 있습니다. 이 경우 관리자 명령보다는 공개 읽기/참석자 명령 위주로 접근하는 편이 낫습니다.
- 실제 현재 usage는 언제든 `oc <명령군> --help`로 다시 확인할 수 있습니다.
