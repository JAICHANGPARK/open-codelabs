# 빠른 시작

이 가이드를 통해 5분 안에 Open Codelabs를 실행할 수 있습니다.

## 사전 준비

최소 요구사항:

- [Docker](https://www.docker.com/get-started) (권장) 또는
- [Bun](https://bun.sh/) + [Rust](https://www.rust-lang.org/) (로컬 개발용)

## Docker로 실행하기 (권장)

가장 간단한 방법입니다. Docker만 설치되어 있으면 됩니다.

### 1. 저장소 클론

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs
```

### 2. Docker Compose로 실행

```bash
docker compose up --build
```

첫 실행 시 이미지를 빌드하므로 몇 분 정도 걸릴 수 있습니다.

### 2-1. 사전 빌드 이미지로 실행 (더 빠르게 시작하기)

로컬 빌드 과정을 건너뛰고 싶다면 퍼블리시된 이미지를 사용할 수 있습니다.

```bash
# 환경 변수 설정 (기본적으로 ghcr.io/jaichangpark/ 이미지를 사용합니다)
cp .env.sample .env

# 사전 빌드 이미지용 컴포즈 파일 실행
docker compose -f docker-compose.images.yml up
```

### 3. 브라우저에서 접속

빌드가 완료되면:

- **Facilitator (관리자)**: [http://localhost:5173/login](http://localhost:5173/login)
  - ID: `admin`
  - PW: `admin`
- **Attendee (참가자)**: [http://localhost:5173](http://localhost:5173)

## CLI로 시작하기

브라우저보다 먼저 CLI 중심으로 시작하고 싶다면 `oc`를 설치한 뒤 질문형 온보딩을 사용할 수 있습니다.

```bash
cargo install --path backend --bin oc
oc init
```

원하면 바로 로컬 스택만 띄울 수도 있습니다.

```bash
oc run --open
```

- 설치와 삭제, 정리 방법은 [설치 가이드](installation.md)에서 확인할 수 있습니다.
- 지원하는 전체 명령군은 [CLI 레퍼런스](../user-guide/cli.md)에 정리되어 있습니다.

## 로컬 개발 환경 실행

개발자라면 로컬에서 직접 실행할 수도 있습니다.

### Backend 실행

```bash
cd backend

# 환경 변수 설정
cat > .env << EOF
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
EOF

# 데이터베이스 디렉토리 생성
mkdir -p data

# 실행
cargo run
```

Backend는 `http://localhost:8080`에서 실행됩니다.

### Frontend 실행

새 터미널에서:

```bash
cd frontend

# 의존성 설치
bun install

# 개발 서버 실행
bun run dev
```

Frontend는 `http://localhost:5173`에서 실행됩니다.

## 첫 번째 Codelab 만들기

### 1. 관리자 로그인

1. [http://localhost:5173/login](http://localhost:5173/login) 접속
2. 기본 자격증명으로 로그인:
   - ID: `admin`
   - PW: `admin123`

### 2. Codelab 생성

1. "새 Codelab 만들기" 버튼 클릭
2. 정보 입력:
   - **제목**: "나의 첫 번째 Codelab"
   - **설명**: "Rust로 웹 서버 만들기"
   - **작성자**: "홍길동"
3. "생성" 클릭

### 3. Step 추가

생성된 Codelab 카드를 클릭하여 편집 페이지로 이동:

1. "Step 추가" 버튼 클릭
2. Step 정보 입력:
   - **제목**: "프로젝트 설정"
   - **내용**: Markdown으로 작성

   ```markdown
   # 프로젝트 설정

   먼저 새로운 Rust 프로젝트를 만듭니다:

   ```bash
   cargo new my-web-server
   cd my-web-server
   ```

   ## 의존성 추가

   `Cargo.toml`에 다음 의존성을 추가합니다:

   ```toml
   [dependencies]
   axum = "0.7"
   tokio = { version = "1.0", features = ["full"] }
   ```
   ```

3. "저장" 클릭

### 4. 참가자로 테스트

1. 새 시크릿 창(또는 다른 브라우저)에서 [http://localhost:5173](http://localhost:5173) 접속
2. Codelab 선택
3. 이름과 참가 코드 입력하여 등록
4. Step을 따라가며 학습 진행

## 다음 단계

축하합니다! 첫 번째 Codelab을 만들었습니다. 🎉

이제 다음을 알아보세요:

- [설치 가이드](installation.md) - 상세한 설치 옵션
- [CLI 레퍼런스](../user-guide/cli.md) - `oc` 설치, 삭제, 연결, 운영 명령
- [첫 번째 Codelab 만들기](first-codelab.md) - 고급 기능 활용
- [공개 배포](../self-hosting/public-deployment.md) - ngrok/bore/cloudflare로 외부에 공개하기
- [API 레퍼런스](../specification/api-reference.md) - API 활용법

## 문제 해결

### Docker 컨테이너가 시작되지 않아요

```bash
# 기존 컨테이너 정리
docker compose down

# 볼륨 포함 완전 정리
docker compose down -v

# 다시 시작
docker compose up --build
```

### 포트가 이미 사용 중이에요

`docker-compose.yml`에서 포트를 변경하세요:

```yaml
services:
  frontend:
    ports:
      - "3000:5173"  # 5173 대신 3000 사용
  backend:
    ports:
      - "3080:8080"  # 8080 대신 3080 사용
```

### 데이터베이스 오류가 발생해요

```bash
# Backend 데이터 초기화
rm -rf backend/data/sqlite.db

# 다시 시작
docker compose restart backend
```

더 많은 문제 해결 방법은 [FAQ](../faq.md)를 참조하세요.
