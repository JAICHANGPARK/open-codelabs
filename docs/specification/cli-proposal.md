# CLI 제안 및 매트릭스

Open Codelabs에 CLI를 추가할지 판단하기 위한 제안 문서입니다. 이 문서는 "무엇을 CLI로 내놓을 것인가"보다 먼저 "어떤 사용자를 위해 어떤 기능을 CLI로 노출해야 하는가"를 정리하는 데 목적이 있습니다.

## 요약

- Open Codelabs의 핵심 제품은 여전히 웹 UI입니다.
- 그러나 운영 기능은 이미 HTTP API 단위로 충분히 분리되어 있어 CLI로 감싸기 좋은 상태입니다.
- CLI의 1차 대상은 `참가자`가 아니라 `운영자`, `콘텐츠 작성자`, `셀프호스터`, `프로젝트 메인테이너`입니다.
- 따라서 첫 CLI는 "터미널에서 워크숍을 운영하고 자동화하는 도구"로 정의하는 것이 맞습니다.

## 목표

CLI는 아래 네 가지 문제를 해결해야 합니다.

### 1. 운영 자동화
- 공개 배포 주소 열기
- 백업/복구 자동화
- 워크스페이스 준비 및 다운로드

### 2. 콘텐츠를 코드처럼 다루기
- 코드랩 생성/복제/내보내기/가져오기
- Step 일괄 반영
- CI 또는 Git 기반 파이프라인 연계

### 3. 브라우저 의존도 축소
- 단순 반복 작업을 관리자 UI 대신 명령 한 줄로 수행
- 로컬/원격 서버를 대상으로 동일한 작업 방식 제공

### 4. 유지보수 도구 제품화
- 현재 내부 벤치 바이너리와 운영 스크립트를 사용자-facing 도구로 승격

## 비목표

초기 CLI는 아래 영역을 직접 대체하지 않습니다.

- 참석자의 학습용 Step 탐색 경험
- 실시간 채팅, DM, 도움 요청 처리의 주 인터페이스
- 화면 공유, 라이브 모니터링, 룰렛 등 실시간 UI 중심 기능
- 브라우저 기반 Markdown 편집/미리보기 경험

이 영역들은 제품의 본질이 시각적이고 실시간성이 강해서 웹 UI가 더 적합합니다.

## 사용자 페르소나

### 운영자 / 퍼실리테이터
- 코드랩 생성과 배포를 반복한다.
- 백업과 복구를 자동화하고 싶다.
- 워크숍 시작 전에 공개 URL, 워크스페이스, 자료 상태를 점검하고 싶다.

### 콘텐츠 작성자
- Markdown 또는 리포지토리 파일 기준으로 코드랩을 관리하고 싶다.
- UI에서 수동으로 여러 단계를 수정하기보다 파일 기반으로 반영하고 싶다.

### 셀프호스터
- 서버 기동, 환경 점검, 공개 터널 연결, 백업 등을 셸 스크립트나 CI에 넣고 싶다.

### 메인테이너
- 성능 벤치, 운영 점검, 회귀 확인을 반복 실행하고 싶다.

## CLI 기본 방향

### 제안하는 제품 정의

`Open Codelabs CLI = 운영/콘텐츠/배포 자동화를 위한 관리용 CLI`

### 제안하는 명령 이름

CLI 명령 이름은 `oc`로 고정합니다.

- 구현과 문서가 이미 `oc` 기준으로 정리되어 있습니다.
- 길이가 짧아 반복 사용에 유리합니다.
- 충돌 가능성은 남아 있지만, Open Codelabs 저장소 문맥에서는 일관성이 더 중요합니다.

## 구현 가능성 근거

이미 저장소에는 CLI로 발전시키기 좋은 토대가 있습니다.

- 공개 배포 스크립트: `run-public.sh`
- 백엔드 API 라우트: 로그인, 코드랩 CRUD, 백업, 워크스페이스, 감사 로그, AI 스레드 등
- 내부 Rust 바이너리: `local_bench`, `ops_bench`, `ws_bench`
- 내부 벤치 코드가 이미 쿠키/CSRF 기반 인증 흐름을 구현하고 있음

즉, "웹 전용 로직만 있는 프로젝트"가 아니라 "기능은 API로 있고 UI가 그 위에 얹혀 있는 프로젝트"에 가깝습니다.

## CLI 매트릭스

아래 표는 각 명령군이 CLI에 얼마나 적합한지, 그리고 MVP에 포함해야 하는지를 정리한 것입니다.

| 명령군 | 주요 사용자 | 현재 기반 | CLI 적합도 | MVP 포함 | 비고 |
|--------|-------------|-----------|------------|----------|------|
| `public up` / `public status` | 운영자, 셀프호스터 | 기존 `run-public.sh`, Docker/Podman, tunnel 스크립트 | 매우 높음 | 예 | 이미 사용 가치가 명확함 |
| `login` / `logout` / `session` | 운영자 | `/api/login`, `/api/logout`, `/api/session` | 높음 | 예 | 다른 모든 관리자 명령의 전제 |
| `codelab list` / `get` | 운영자, 작성자 | `/api/codelabs`, `/api/codelabs/{id}` | 매우 높음 | 예 | 읽기 전용부터 시작 가능 |
| `codelab create` / `copy` / `delete` | 운영자 | 코드랩 CRUD API | 높음 | 예 | 운영 자동화 가치가 큼 |
| `codelab export` / `import` | 운영자, 작성자 | ZIP import/export API | 매우 높음 | 예 | Git/백업/이관 시나리오와 잘 맞음 |
| `codelab push-steps` | 작성자 | `/api/codelabs/{id}/steps` | 매우 높음 | 예 | "콘텐츠를 코드로 관리"하는 핵심 |
| `backup export` / `inspect` / `restore` | 운영자, 셀프호스터 | 백업 API | 매우 높음 | 예 | CLI의 대표 사용 사례 |
| `workspace create` / `info` / `download` | 운영자 | code-server API | 높음 | 예 | 워크숍 준비 자동화에 유리 |
| `workspace branch` / `folder` / `sync-files` | 운영자, 작성자 | code-server branch/folder/files API | 높음 | 보류 | 구조 정의부터 필요 |
| `audit logs` | 운영자 | `/api/admin/audit-logs` | 높음 | 예 | 운영 이슈 조사에 유용 |
| `attendee list` | 운영자 | `/api/codelabs/{id}/attendees` | 중간 이상 | 보류 | 읽기 조회는 유용하나 우선순위는 다소 낮음 |
| `attendee register` / `complete` | 운영자, 스크립트 사용자 | 참석자 API | 중간 | 보류 | 운영 자동화엔 쓸 수 있으나 UI 우선 영역 |
| `materials add` / `list` | 작성자, 운영자 | materials API | 중간 이상 | 보류 | 파일 업로드 UX 정의가 필요 |
| `quiz pull` / `push` / `submissions` | 작성자, 운영자 | quiz API | 중간 이상 | 보류 | 스키마와 파일 포맷 정의 선행 필요 |
| `ai generate` / `ai threads` | 작성자 | AI API, settings 암호화 흐름 | 중간 | 보류 | 인증/암호화 UX를 먼저 정리해야 함 |
| `live help` / `chat` | 운영자 | help/chat/WebSocket API | 낮음 | 아니오 | 실시간 운영의 본질은 UI에 있음 |
| `screen-share` 관련 | 운영자, 참석자 | WebSocket 기반 라이브 기능 | 매우 낮음 | 아니오 | CLI의 장점이 거의 없음 |
| `bench local` / `bench ops` / `bench ws` | 메인테이너, 셀프호스터 | 기존 Rust 벤치 바이너리 | 높음 | 예 | 일반 사용자보다는 고급 사용자 대상 |

## 권장 MVP 범위

첫 릴리스는 "작지만 바로 쓸 수 있는 운영 CLI"로 잘라내는 편이 맞습니다.

### MVP 명령군

```text
oc auth login
oc auth status
oc public up
oc codelab list
oc codelab create
oc codelab export
oc codelab import
oc codelab push-steps
oc backup export
oc backup inspect
oc backup restore
oc workspace create
oc workspace download
oc audit logs
oc bench local
oc bench ops
oc bench ws
```

### MVP에서 의도적으로 제외할 것

- 참석자 학습 플로우
- 실시간 채팅/DM 처리
- 화면 공유 기능
- 브라우저형 에디터 경험을 그대로 대체하는 기능

## 단계별 출시 제안

### Phase 1: 운영 자동화

목표는 "지금 당장 실무에 쓸 수 있는 관리자 CLI"입니다.

- 인증 세션 관리
- 공개 배포 자동화
- 백업/복구
- 코드랩 조회/생성/가져오기/내보내기

### Phase 2: 콘텐츠 as Code

목표는 "파일 기반으로 코드랩을 관리"하는 것입니다.

- 로컬 Markdown/JSON manifest에서 코드랩 반영
- Step 일괄 업로드
- 자료, 퀴즈, 가이드 파일 동기화
- CI 파이프라인용 non-interactive 모드

### Phase 3: 고급 운영 및 진단

- 워크스페이스 구조 생성
- 감사 로그 조회 및 필터링
- 벤치/헬스체크/운영 점검
- 배포 전 검증 명령

## 권장 파일 포맷

CLI가 유의미해지려면 단순 API 래퍼를 넘어 파일 기반 워크플로우를 제공해야 합니다.

### 최소 manifest 예시

```yaml
version: 1
title: Rust Async Codelab
description: Axum과 Tokio 기초
author: Open Codelabs
is_public: true
quiz_enabled: false
require_quiz: false
require_feedback: false
require_submission: false
guide_markdown: docs/guide.md
steps:
  - title: 환경 준비
    file: steps/01-setup.md
  - title: 첫 번째 핸들러
    file: steps/02-handler.md
materials: []
quizzes: []
```

이 포맷은 추후 `oc codelab push` 같은 상위 명령으로 확장할 수 있습니다.

## 설계 원칙

### 1. API 래퍼에서 시작하되 파일 워크플로우로 확장
- 초기에는 기존 API를 안정적으로 감싼다.
- 그 다음 단계에서 manifest 기반 동기화로 발전시킨다.

### 2. 사람과 자동화 모두 지원
- 사람 친화적인 표 출력
- `--json` 옵션으로 기계 친화적인 출력
- 비대화형 인증 또는 토큰/세션 저장 구조 필요

### 3. 웹 UI와 경쟁하지 말고 보완
- 실시간/시각 중심 경험은 웹에 남긴다.
- 반복 작업과 배치 작업은 CLI가 맡는다.

### 4. 초기 구현은 Rust 단일 바이너리 우선
- 백엔드와 같은 언어를 사용해 코드 재사용 여지가 있다.
- 내부 벤치 바이너리도 Rust 기반이라 통합이 쉽다.

## 주요 리스크

### 인증 UX
- 현재 관리자 인증은 쿠키 + CSRF 기반이다.
- 내부 벤치 코드는 이미 이 흐름을 처리하지만, 제품용 CLI에서는 세션 저장 위치와 만료 처리 정책을 정해야 한다.

### 파일 포맷 결정
- Step, guide, quiz, materials를 어떤 파일 구조로 표현할지 빨리 정해야 한다.
- 포맷이 흔들리면 CLI가 단순 API 래퍼에 머물게 된다.

### 명령 이름 충돌
- `oc`는 다른 도구와 충돌 가능성이 있다.
- 다만 현재 저장소에서는 이미 `oc`로 구현되어 있어 문서/배포 일관성을 유지하는 편이 낫다.

### 제품 포지셔닝 혼선
- 참가자 CLI를 만들 것처럼 보이면 범위가 급격히 커진다.
- 문서와 README에서 "운영용/작성자용 CLI"임을 명확히 해야 한다.

## 결정 사항

이 문서 기준으로 다음을 1차 결정으로 둡니다.

- CLI는 만들 가치가 있다.
- 1차 대상 사용자는 운영자/작성자/셀프호스터/메인테이너다.
- 실시간 참석자 경험은 초기 범위에서 제외한다.
- MVP는 `public`, `auth`, `codelab`, `backup`, `workspace`, `audit`, `bench` 중심으로 설계한다.

## 다음 작업

구현 전 다음 순서로 진행하는 것이 적절합니다.

1. CLI 명칭 확정
2. 세션 저장 방식과 출력 규약 정의
3. `codelab push`용 manifest 스키마 초안 작성
4. Phase 1 명령의 UX spec 작성
5. 그 다음에 실제 바이너리 골격 구현
