# 환경 변수 설정

Open Codelabs의 모든 환경 변수에 대한 상세 가이드입니다.

!!! info "환경 변수 파일 위치"
    - Docker Compose: 리포지토리 루트 `.env` (docker-compose.yml에서 참조)
    - 로컬 개발: `backend/.env`, `frontend/.env`

## Backend 환경 변수

### 필수 환경 변수

#### DATABASE_URL

데이터베이스 연결 문자열 (SQLite, PostgreSQL, MySQL 지원)

**SQLite (기본값)**:
```bash
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
```

**PostgreSQL**:
```bash
DATABASE_URL=postgres://user:password@localhost:5432/dbname
```

**MySQL**:
```bash
DATABASE_URL=mysql://user:password@localhost:3306/dbname
```

**형식**: `sqlite:<경로>?<옵션>` 또는 `<db_type>://<user>:<password>@<host>:<port>/<dbname>`

**옵션**:
- `mode=rwc`: Read, Write, Create 모드
- `mode=ro`: Read-only 모드
- `mode=memory`: 인메모리 데이터베이스

**예시**:

```bash
# 로컬 개발
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc

# Docker
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc

# 인메모리 (테스트용)
DATABASE_URL=sqlite::memory:
```

#### ADMIN_ID

관리자 로그인 ID

```bash
ADMIN_ID=admin
```

!!! warning "보안"
    프로덕션에서는 `admin` 같은 기본값을 사용하지 마세요.

**권장**:
- 8자 이상
- 예측하기 어려운 값
- 이메일 형식 사용 가능

```bash
ADMIN_ID=facilitator_2024
ADMIN_ID=admin@example.com
```

#### ADMIN_PW

관리자 비밀번호

```bash
ADMIN_PW=admin123
```

!!! danger "필수"
    프로덕션에서는 반드시 강력한 비밀번호로 변경하세요!

**권장**:
- 최소 20자 이상
- 대소문자, 숫자, 특수문자 조합
- 생성기 사용 권장

```bash
# 강력한 비밀번호 생성
openssl rand -base64 32
# 출력: 8vYR3jkLm9nP2qTxWz6CbF4hK7dN5sVuG1aE0iJ3XyO=

ADMIN_PW=8vYR3jkLm9nP2qTxWz6CbF4hK7dN5sVuG1aE0iJ3XyO=
```

### 선택적 환경 변수

#### AUTH_SECRETS

JWT 서명 시크릿 목록 (쉼표 구분). 첫 번째가 활성 키이며, 나머지는 검증용으로 남겨 키 롤링을 지원합니다.
비어있으면 `ADMIN_PW`를 대체로 사용합니다.

```bash
AUTH_SECRETS=primary_secret,previous_secret
```

!!! note "레거시"
    `AUTH_SECRET`도 읽지만 신규 설정은 `AUTH_SECRETS` 사용을 권장합니다.

#### AUTH_ISSUER

JWT issuer 클레임.

```bash
AUTH_ISSUER=open-codelabs
```

기본값: `open-codelabs`

#### AUTH_AUDIENCE

JWT audience 클레임.

```bash
AUTH_AUDIENCE=open-codelabs
```

기본값: `open-codelabs`

#### ADMIN_SESSION_TTL_SECONDS

관리자 세션 TTL(초).

```bash
ADMIN_SESSION_TTL_SECONDS=28800
```

기본값: `28800` (8시간)

#### ATTENDEE_SESSION_TTL_SECONDS

참가자 세션 TTL(초).

```bash
ATTENDEE_SESSION_TTL_SECONDS=43200
```

기본값: `43200` (12시간)

#### COOKIE_SECURE

HTTPS 환경에서 `true`로 설정 (Secure 쿠키 + `__Host-` 프리픽스).

```bash
COOKIE_SECURE=true
```

기본값: `false`

#### COOKIE_SAMESITE

쿠키 SameSite 정책. `lax`(기본), `strict`, `none`.

```bash
COOKIE_SAMESITE=lax
```

!!! warning "주의"
    `COOKIE_SAMESITE=none`을 사용하려면 `COOKIE_SECURE=true`가 필요합니다.

#### TRUST_PROXY

리버스 프록시 뒤에서 `X-Forwarded-*` 헤더를 신뢰할 때 `true`.

```bash
TRUST_PROXY=true
```

기본값: `false`

#### CORS_ALLOWED_ORIGINS

허용할 오리진 목록(쉼표 구분). 비어있으면 로컬 기본값을 허용합니다.

```bash
CORS_ALLOWED_ORIGINS=http://localhost:5173,http://127.0.0.1:5173
```

#### RATE_LIMIT_GENERAL_PER_MINUTE

일반 API 요청 분당/IP 제한.

```bash
RATE_LIMIT_GENERAL_PER_MINUTE=120
```

기본값: `120`

#### RATE_LIMIT_LOGIN_PER_5_MIN

로그인 요청 5분/IP 제한.

```bash
RATE_LIMIT_LOGIN_PER_5_MIN=20
```

기본값: `20`

#### RATE_LIMIT_AI_PER_MINUTE

AI 프록시 요청 분당/IP 제한.

```bash
RATE_LIMIT_AI_PER_MINUTE=30
```

기본값: `30`

#### RATE_LIMIT_UPLOAD_PER_MINUTE

업로드 및 제출 POST 분당/IP 제한.

```bash
RATE_LIMIT_UPLOAD_PER_MINUTE=20
```

기본값: `20`

#### CSP_HEADER

UI 응답의 Content-Security-Policy 헤더를 오버라이드합니다. 비어있으면 기본값을 사용합니다.

```bash
CSP_HEADER=default-src 'self'; img-src 'self' data: blob:
```

#### HSTS_HEADER

Strict-Transport-Security 헤더를 오버라이드합니다. HTTPS일 때만 적용됩니다.

```bash
HSTS_HEADER=max-age=63072000; includeSubDomains; preload
```

#### ALLOWED_GEMINI_MODELS

허용할 Gemini 모델 ID 목록(쉼표 구분). 설정 시 목록에 없는 모델 요청은 거부됩니다.

```bash
ALLOWED_GEMINI_MODELS=gemini-3-flash-preview,gemini-1.5-pro
```

#### GEMINI_API_KEY

관리자 키가 저장되지 않았을 때 사용할 기본 Gemini API 키.

```bash
GEMINI_API_KEY=your_gemini_api_key_here
```

#### RUST_LOG

로그 레벨 설정

```bash
RUST_LOG=backend=debug,tower_http=debug
```

**레벨**: `error`, `warn`, `info`, `debug`, `trace`

**모듈별 설정**:

```bash
# 전체 디버그
RUST_LOG=debug

# 모듈별 지정
RUST_LOG=backend=debug,sqlx=info,tower_http=warn

# 특정 핸들러만
RUST_LOG=backend::handlers::codelabs=trace

# 프로덕션
RUST_LOG=backend=info,tower_http=info
```

#### PORT (Backend)

Backend API 서버 포트

```bash
# 기본값: 8080
PORT=8080

# 커스텀 포트
PORT=3000
```

코드에서 사용하려면 `main.rs` 수정:

```rust
let port = std::env::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .expect("PORT must be a valid number");

let addr = SocketAddr::from(([0, 0, 0, 0], port));
```

## Frontend 환경 변수

### VITE_API_URL

Backend API URL

```bash
VITE_API_URL=http://localhost:8080
```

**다양한 환경**:

```bash
# 로컬 개발
VITE_API_URL=http://localhost:8080

# Docker Compose (내부 네트워크)
VITE_API_URL=http://backend:8080

# 프로덕션
VITE_API_URL=https://api.example.com

# ngrok
VITE_API_URL=https://abc123.ngrok-free.app

# Cloudflare Tunnel
VITE_API_URL=https://abc123.trycloudflare.com
```

!!! info "Vite 환경 변수"
    - `VITE_`로 시작하는 변수만 클라이언트에 노출됨
    - 빌드 타임에 코드에 삽입됨
    - 민감한 정보는 포함하지 마세요

### VITE_USE_SUPABASE

Enable Supabase serverless mode in the frontend.

```bash
VITE_USE_SUPABASE=true
```

### VITE_SUPABASE_URL

Supabase project URL.

```bash
VITE_SUPABASE_URL=https://your-project.supabase.co
```

### VITE_SUPABASE_ANON_KEY

Supabase public anon key (safe for client).

```bash
VITE_SUPABASE_ANON_KEY=your_anon_key
```

### VITE_SUPABASE_STORAGE_BUCKET

Storage bucket name for uploads (images/materials/submissions).

```bash
VITE_SUPABASE_STORAGE_BUCKET=open-codelabs
```

### VITE_ADMIN_ID / VITE_ADMIN_PW

Optional admin fallback for serverless mode (Supabase/Firebase).

```bash
VITE_ADMIN_ID=admin
VITE_ADMIN_PW=admin
```

### VITE_ADMIN_ENCRYPTION_PASSWORD

Gemini API 키를 브라우저에서 암호화할 때 사용하는 비밀번호.
백엔드 `ADMIN_PW`와 동일해야 복호화가 가능합니다. 비워두면 UI에서 입력을 요구합니다.

```bash
VITE_ADMIN_ENCRYPTION_PASSWORD=your_admin_pw
```

### FRONTEND_PORT

Docker Compose에서 프론트엔드 컨테이너 포트(호스트 매핑)와 `PORT`에 전달됩니다.

```bash
FRONTEND_PORT=5173
```

### FRONTEND_HOST

Docker Compose에서 프론트엔드 컨테이너 바인딩 호스트로 사용되며 `HOST`에 전달됩니다.

```bash
FRONTEND_HOST=0.0.0.0
```

### PORT

Frontend 서버 포트

```bash
PORT=5173  # 기본값
```

Docker Compose에서는 `FRONTEND_PORT` 값이 `PORT`로 전달됩니다.

### HOST

바인딩 호스트

```bash
HOST=0.0.0.0  # 모든 인터페이스
HOST=127.0.0.1  # localhost만
```

Docker Compose에서는 `FRONTEND_HOST` 값이 `HOST`로 전달됩니다.

## 환경별 설정

### 로컬 개발

`backend/.env`:

```bash
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
AUTH_SECRETS=change_me_primary,change_me_old
GEMINI_API_KEY=your_gemini_api_key_here
RUST_LOG=backend=debug,tower_http=debug,sqlx=info
```

`frontend/.env`:

```bash
VITE_API_URL=http://localhost:8080
VITE_ADMIN_ENCRYPTION_PASSWORD=admin123
```

### Docker Compose

리포지토리 루트의 `.env`가 `docker-compose.yml`에 주입됩니다.

`.env` (repo root):

```bash
DATA_VOLUME_PATH=~/open-codelabs
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=YourSecurePassword123!
AUTH_SECRETS=change_me_primary,change_me_old
VITE_API_URL=http://localhost:8080
VITE_ADMIN_ENCRYPTION_PASSWORD=YourSecurePassword123!
FRONTEND_PORT=5173
FRONTEND_HOST=0.0.0.0
```

`docker-compose.yml` (발췌):

```yaml
services:
  backend:
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - ADMIN_ID=${ADMIN_ID}
      - ADMIN_PW=${ADMIN_PW}
    volumes:
      - ${DATA_VOLUME_PATH}/data:/app/data
      - ${DATA_VOLUME_PATH}/uploads:/app/static/uploads

  frontend:
    environment:
      - VITE_API_URL=${VITE_API_URL}
      - VITE_ADMIN_ENCRYPTION_PASSWORD=${ADMIN_PW}
      - PORT=${FRONTEND_PORT}
      - HOST=${FRONTEND_HOST}
```

기본 docker-compose.yml에서는 `VITE_ADMIN_ENCRYPTION_PASSWORD` 대신 `ADMIN_PW`를 프론트에 전달합니다. 다른 값을 쓰려면 `frontend.environment`를 수정하세요.

필요하면 backend 환경 변수(AUTH_*, COOKIE_*, CORS_*, RATE_LIMIT_*, CSP_HEADER, HSTS_HEADER, ALLOWED_GEMINI_MODELS 등)를 `backend.environment`에 추가하세요.

또는 docker-compose.yml에 env_file을 추가해 `backend/.env`, `frontend/.env`를 직접 주입할 수 있습니다.

```yaml
services:
  backend:
    env_file:
      - backend/.env

  frontend:
    env_file:
      - frontend/.env
```

### 프로덕션

`backend/.env.production`:

```bash
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
ADMIN_ID=${ADMIN_ID}  # 외부에서 주입
ADMIN_PW=${ADMIN_PW}  # 외부에서 주입
AUTH_SECRETS=${AUTH_SECRETS}
COOKIE_SECURE=true
TRUST_PROXY=true
RUST_LOG=backend=info,tower_http=warn
```

`frontend/.env.production`:

```bash
VITE_API_URL=https://api.yourdomain.com
VITE_ADMIN_ENCRYPTION_PASSWORD=${ADMIN_PW}
```

**비밀 관리**:

```bash
# .env.local (Git에 추가하지 않음)
ADMIN_ID=your_real_admin_id
ADMIN_PW=your_real_secure_password

# 환경 변수로 주입
export ADMIN_ID="your_real_admin_id"
export ADMIN_PW="your_real_secure_password"
docker-compose up
```

## 환경 변수 검증

### Backend 시작 시 검증

`main.rs`에 추가:

```rust
fn validate_env() -> anyhow::Result<()> {
    let required = vec!["DATABASE_URL", "ADMIN_ID", "ADMIN_PW"];

    for var in required {
        std::env::var(var)
            .map_err(|_| anyhow::anyhow!("{} must be set", var))?;
    }

    // 비밀번호 강도 확인
    let pw = std::env::var("ADMIN_PW")?;
    if pw.len() < 12 {
        tracing::warn!("ADMIN_PW is too short! Use at least 12 characters.");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    validate_env()?;

    // ...
}
```

### 스크립트로 검증

`check-env.sh`:

```bash
#!/bin/bash

required_backend=("DATABASE_URL" "ADMIN_ID" "ADMIN_PW")
required_frontend=("VITE_API_URL")

echo "Checking backend environment..."
for var in "${required_backend[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ $var is not set"
        exit 1
    else
        echo "✅ $var is set"
    fi
done

echo "Checking frontend environment..."
for var in "${required_frontend[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ $var is not set"
        exit 1
    else
        echo "✅ $var is set"
    fi
done

echo "✅ All required environment variables are set!"
```

```bash
chmod +x check-env.sh
source backend/.env && source frontend/.env && ./check-env.sh
```

## 보안 베스트 프랙티스

### 1. .env 파일 보호

`.gitignore`:

```gitignore
# 환경 변수
.env
.env.local
.env.production
.env.*.local
backend/.env
frontend/.env

# 데이터베이스
*.db
*.db-*
```

### 2. 예제 파일 제공

`.env.example`:

```bash
# Backend Configuration
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=your_admin_id_here
ADMIN_PW=your_secure_password_here
RUST_LOG=backend=info,tower_http=info

# Instructions:
# 1. Copy this file to .env
# 2. Replace placeholder values
# 3. Never commit .env to version control
```

사용자:

```bash
cp .env.example .env
nano .env  # 값 수정
```

### 3. CI/CD에서 비밀 관리

#### GitHub Actions

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Deploy with secrets
        env:
          ADMIN_ID: ${{ secrets.ADMIN_ID }}
          ADMIN_PW: ${{ secrets.ADMIN_PW }}
        run: |
          echo "DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc" > backend/.env
          echo "ADMIN_ID=$ADMIN_ID" >> backend/.env
          echo "ADMIN_PW=$ADMIN_PW" >> backend/.env
          docker-compose up -d
```

Repository Settings → Secrets에서 설정

### 4. 프로덕션 비밀번호 정책

```bash
# 최소 요구사항 검증 스크립트
validate_password() {
    local pw=$1
    local len=${#pw}

    if [ $len -lt 20 ]; then
        echo "❌ Password too short (minimum 20 characters)"
        return 1
    fi

    if ! [[ "$pw" =~ [A-Z] ]]; then
        echo "❌ Password must contain uppercase letters"
        return 1
    fi

    if ! [[ "$pw" =~ [a-z] ]]; then
        echo "❌ Password must contain lowercase letters"
        return 1
    fi

    if ! [[ "$pw" =~ [0-9] ]]; then
        echo "❌ Password must contain numbers"
        return 1
    fi

    echo "✅ Password meets requirements"
    return 0
}

validate_password "$ADMIN_PW"
```

## 환경 변수 디버깅

### 현재 설정 확인

```bash
# Backend
cd backend
cargo run --bin print-env

# 또는 직접
env | grep -E '(DATABASE_URL|ADMIN_ID|RUST_LOG)'

# Frontend
cd frontend
bun run dev --mode development
```

### 환경 변수 로깅

Backend `main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing::info!("DATABASE_URL: {}", std::env::var("DATABASE_URL")?);
    tracing::info!("ADMIN_ID: {}", std::env::var("ADMIN_ID")?);
    tracing::info!("ADMIN_PW: ********");  // 비밀번호는 로깅하지 않음
    tracing::info!("RUST_LOG: {:?}", std::env::var("RUST_LOG").ok());

    // ...
}
```

!!! warning "주의"
    로그에 민감한 정보(비밀번호, API 키)를 출력하지 마세요!

## 다음 단계

- [Docker 배포](docker.md) - Docker로 프로덕션 배포
- [보안 가이드](security.md) - 운영 보안 체크리스트
- [FAQ](../faq.md) - 자주 묻는 질문
