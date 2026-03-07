# CLI 레퍼런스

이 문서는 현재 `oc --help` 기준으로 지원하는 Open Codelabs CLI 명령을 정리합니다.

`oc <명령> --help`는 각 명령군의 상세 usage를 바로 출력합니다.

## 공통 전역 옵션

모든 명령은 아래 전역 옵션을 공통으로 지원합니다.

| 옵션 | 설명 |
| --- | --- |
| `--base-url <url>` | 연결할 백엔드 기본 URL을 직접 지정합니다. |
| `--session-file <path>` | 세션 파일 경로를 덮어씁니다. |
| `--config-file <path>` | CLI 설정 파일 경로를 변경합니다. 기본값은 `~/.open-codelabs/config.json`입니다. |
| `--profile <name>` | 저장된 연결 프로필 중 하나를 선택합니다. |
| `--json` | 사람이 읽기 쉬운 출력 대신 JSON을 출력합니다. |
| `-h`, `--help` | 도움말을 출력합니다. |

## 기본 사용 흐름

### 1. CLI만으로 로컬 스택 실행

```bash
oc run --open
oc ps
oc logs --service backend --tail 200 --no-follow
```

### 2. 실행 중인 서버에 연결

```bash
oc connect add --name local --url http://localhost:8080 --runtime backend --activate
oc connect status
```

### 3. 관리자 인증

```bash
oc auth login
oc auth status
```

### 4. 참석자 세션 시작

```bash
oc attendee join --codelab-id demo --name "Jane" --code ABC123
```

## 세션과 권한 범위

| 범위 | 세션 획득 방법 | 대표 명령 |
| --- | --- | --- |
| 로컬 런타임 | 인증 불필요 | `oc run`, `oc ps`, `oc logs`, `oc restart`, `oc down` |
| 공개 읽기 | `oc connect`만 필요 | `oc codelab list`, `oc codelab reference`, `oc codelab get` |
| 관리자 | `oc auth login` | `admin`, `backup`, `audit`, `workspace`, `codelab create/update/delete/copy/export/import/push-steps` |
| 참석자 | `oc attendee join` | `help request`, `feedback submit`, `quiz submit`, `submission file/link/delete`, `chat history` |

실제 접근 제어는 연결한 런타임과 서버 설정에 따라 백엔드에서 최종 결정됩니다.

## 연결과 인증

### 연결 프로필

```bash
oc connect add --name <name> --url <url> [--runtime <auto|backend|firebase|supabase>] [--activate]
oc connect use --name <name>
oc connect list
oc connect status
```

- 여러 서버를 프로필로 저장하고 전환할 때 사용합니다.
- `connect status`는 현재 URL, 런타임 추정, capability를 보여줍니다.

### 관리자 인증

```bash
oc auth login [--no-open]
oc auth logout
oc auth status
```

- `oc auth login`은 브라우저 로그인 기반 관리자 인증을 시작합니다.
- 기본 세션 저장 위치는 `~/.open-codelabs/` 아래입니다.

### 레거시 별칭

```bash
oc login --admin-id <id> --admin-pw <pw>
oc logout
oc session
```

- 새 스크립트에서는 `auth` 명령군 사용을 권장합니다.

## 로컬 스택 런타임

```bash
oc run [--engine <auto|docker|podman>] [--postgres] [--pull] [--open] [--admin-id <id>] [--admin-pw <pw>] [--data-dir <path>] [--frontend-port <port>] [--backend-port <port>] [--image-registry <registry>] [--image-namespace <namespace>] [--image-tag <tag>]
oc ps [--service <name>]
oc logs [--service <name>] [--tail <n>] [--no-follow]
oc restart [--service <name>]
oc down [--volumes]
```

- `oc run`은 `docker` 또는 `podman`을 감지하고, 엔진이 없거나 꺼져 있으면 설치/실행 방법을 안내합니다.
- `oc down --volumes`는 `oc run`이 만든 로컬 bind mount 데이터 디렉터리를 함께 정리합니다.

## 관리 명령

### 관리자 설정

```bash
oc admin settings [--gemini-api-key <key>] [--admin-password <pw>]
oc admin updates
```

### 코드랩

```bash
oc codelab list
oc codelab reference
oc codelab get --id <id>
oc codelab create --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
oc codelab update --id <id> --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
oc codelab delete --id <id>
oc codelab copy --id <id>
oc codelab export --id <id> [--output <path>]
oc codelab import --file <zip>
oc codelab push-steps --id <id> --file <json>
```

### 백업과 감사 로그

```bash
oc backup export [--output <path>]
oc backup inspect --file <zip>
oc backup restore --file <zip>
oc audit logs [--limit <n>] [--offset <n>] [--action <name>] [--codelab-id <id>]
```

### 워크스페이스

```bash
oc workspace create --codelab-id <id> [--structure-type <branch|folder>] [--files-json <path>]
oc workspace info --codelab-id <id>
oc workspace download --codelab-id <id> [--output <path>]
oc workspace delete --codelab-id <id>
oc workspace branches --codelab-id <id>
oc workspace branch-create --codelab-id <id> --step-number <n> [--branch-type <start|end>]
oc workspace branch-files --codelab-id <id> --branch <name>
oc workspace branch-read --codelab-id <id> --branch <name> --file <path>
oc workspace branch-update --codelab-id <id> --branch <name> --files-json <path> [--delete-json <path>] [--commit-message <message>]
oc workspace folders --codelab-id <id>
oc workspace folder-create --codelab-id <id> --step-number <n> --files-json <path> [--folder-type <start|end>]
oc workspace folder-files --codelab-id <id> --folder <name>
oc workspace folder-read --codelab-id <id> --folder <name> --file <path>
oc workspace folder-update --codelab-id <id> --folder <name> --files-json <path> [--delete-json <path>]
```

## 참석자 운영과 라이브 도구

### 참석자

```bash
oc attendee join --codelab-id <id> --name <name> --code <code> [--email <email>]
oc attendee list --codelab-id <id>
oc attendee complete --codelab-id <id>
oc attendee certificate [--attendee-id <id>]
```

### 도움 요청과 피드백

```bash
oc help request --codelab-id <id> --step-number <n>
oc help list --codelab-id <id>
oc help resolve --codelab-id <id> --help-id <id>
oc feedback submit --codelab-id <id> --difficulty <1-5> --satisfaction <1-5> [--comment <text>]
oc feedback list --codelab-id <id>
```

### 자료, 퀴즈, 제출물

```bash
oc materials list --codelab-id <id>
oc materials upload --file <path>
oc materials add --codelab-id <id> --title <title> --type <link|file> [--url <url>] [--file-path <path>]
oc materials delete --codelab-id <id> --material-id <id>
oc quiz list --codelab-id <id>
oc quiz update --codelab-id <id> --file <json>
oc quiz submit --codelab-id <id> --file <json>
oc quiz submissions --codelab-id <id>
oc submission list --codelab-id <id>
oc submission file --codelab-id <id> [--attendee-id <id>] --file <path>
oc submission link --codelab-id <id> [--attendee-id <id>] --url <url> [--title <title>]
oc submission delete --codelab-id <id> [--attendee-id <id>] --submission-id <id>
```

### 채팅, 업로드, 인라인 댓글

```bash
oc chat history --codelab-id <id>
oc upload image --file <path>
oc inline list --codelab-id <id> [--target-type <guide|step>] [--target-step-id <id>]
oc inline create --codelab-id <id> --file <json>
oc inline reply --codelab-id <id> --thread-id <id> --file <json>
oc inline delete --codelab-id <id> --thread-id <id> --comment-id <id>
```

## AI 명령

```bash
oc ai conversations --codelab-id <id>
oc ai stream --file <json>
oc ai save --file <json>
oc ai threads
oc ai thread-create --title <title> [--codelab-id <id>]
oc ai thread-delete --thread-id <id>
oc ai messages --thread-id <id>
oc ai message-add --thread-id <id> --file <json>
```

- `conversations`는 코드랩 단위 대화 목록 조회입니다.
- `threads`, `messages`, `message-add`는 영속 스레드 관리용입니다.
- `stream`, `save`는 JSON payload 파일을 넘겨 자동화 스크립트에서 사용하기 좋습니다.

## 운영 팁

- 명령별 상세 옵션은 `oc <명령군> --help`로 바로 확인합니다.
- 자동화나 CI에서는 `--json`을 붙여 기계가 읽기 쉬운 출력을 사용합니다.
- 여러 환경을 쓸 때는 `oc connect add`와 `oc --profile <name>` 조합으로 관리합니다.
