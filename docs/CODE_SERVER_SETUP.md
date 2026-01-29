# Code Server Workspace Setup

이 가이드는 대화형 코딩 워크샵을 위한 code-server 설정 방법을 설명합니다.

## 개요

AI 생성기로 코드랩을 생성할 때, code-server 워크스페이스를 선택적으로 생성할 수 있습니다:
- 업로드된 모든 코드 파일 포함
- 각 단계별 Git 브랜치 (시작/종료 상태)
- 참석자가 ngrok 터널을 통해 접속 가능
- 퍼실리테이터가 다운로드 가능

## 아키텍처

하나의 공유 code-server 컨테이너를 사용하며, 각 코드랩은 별도의 디렉토리를 가집니다.

## 설정

```bash
docker-compose up -d
```

`.env` 파일에서 설정:
```bash
CODESERVER_PORT=8443
CODESERVER_PASSWORD=codelab123
```

## 사용 방법

### 1. 코드랩 생성
1. Admin 페이지 이동
2. "Generate with AI" 클릭
3. 코드 파일 업로드
4. "Create Code Server Workspace" 체크
5. 코드랩 생성

### 2. 참석자와 공유

ngrok으로 터널링:
```bash
ngrok http 8443
```

생성된 URL을 참석자에게 공유 (비밀번호: `codelab123`)

### 3. 워크스페이스 다운로드

Admin 페이지에서 📤 버튼 클릭하여 `.tar.gz` 파일 다운로드

## API 엔드포인트

- `POST /api/codeserver` - 워크스페이스 생성
- `GET /api/codeserver/{id}` - 워크스페이스 정보
- `POST /api/codeserver/{id}/branch` - Git 브랜치 생성
- `GET /api/codeserver/{id}/download` - 워크스페이스 다운로드
- `DELETE /api/codeserver/{id}` - 워크스페이스 삭제
