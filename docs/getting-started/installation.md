# 설치 가이드

Open Codelabs를 설치하는 다양한 방법을 소개합니다.

## 시스템 요구사항

### 최소 요구사항

- **메모리**: 2GB RAM
- **디스크**: 1GB 여유 공간
- **OS**: Linux, macOS, Windows (WSL2)

### 소프트웨어 요구사항

=== "Docker (권장)"
    - Docker Engine 20.10+
    - Docker Compose v2.0+

=== "로컬 개발"
    - Rust 1.75+
    - Bun 1.0+
    - SQLite 3.35+

=== "Podman"
    - Podman 4.0+
    - podman-compose 1.0+

## Docker로 설치하기

Docker는 가장 간단하고 권장되는 설치 방법입니다.

### 1. Docker 설치

=== "Linux"
    ```bash
    # Ubuntu/Debian
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh

    # Docker Compose 설치
    sudo apt-get install docker-compose-plugin
    ```

=== "macOS"
    [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop)을 다운로드하여 설치합니다.

=== "Windows"
    [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop)를 다운로드하여 설치합니다.

    WSL2를 사용하는 것을 권장합니다.

!!! note
    Docker Desktop에는 Docker Compose가 기본 포함됩니다. Linux는 `docker-compose-plugin` 설치가 필요할 수 있습니다.
    환경에 따라 `docker compose` 대신 `docker-compose`를 사용해야 할 수 있습니다.

### 2. 프로젝트 클론

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs
```

### 3. 환경 변수 설정 (선택사항)

기본 설정으로도 실행 가능하지만, 관리자 계정을 변경하고 싶다면:

```bash
# docker-compose.yml 편집
nano docker-compose.yml
```

```yaml
services:
  backend:
    environment:
      - DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
      - ADMIN_ID=your_admin_id        # 변경
      - ADMIN_PW=your_secure_password # 변경
```

### 4. 실행

```bash
docker compose up -d
```

- `-d`: 백그라운드에서 실행

### 5. 로그 확인

```bash
# 모든 서비스 로그
docker compose logs -f

# Backend만
docker compose logs -f backend

# Frontend만
docker compose logs -f frontend
```

### 6. 중지 및 시작

```bash
# 중지
docker compose stop

# 시작
docker compose start

# 중지 및 컨테이너 제거
docker compose down

# 볼륨까지 제거 (데이터 삭제)
docker compose down -v
```

## Podman으로 설치하기

Docker 대신 Podman을 사용하는 경우:

### 1. Podman 설치

=== "Fedora/RHEL"
    ```bash
    sudo dnf install podman podman-compose
    ```

=== "Ubuntu"
    ```bash
    sudo apt-get install podman podman-compose
    ```

=== "macOS"
    ```bash
    brew install podman podman-compose
    podman machine init
    podman machine start
    ```

### 2. 실행

```bash
podman-compose up -d
```

나머지 명령어는 `docker compose` 대신 `podman-compose`를 사용하면 됩니다.

## 로컬 개발 환경 설치

개발자를 위한 상세 설치 가이드입니다.

### 1. Rust 설치

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 버전 확인
rustc --version
cargo --version
```

### 2. Bun 설치

```bash
curl -fsSL https://bun.sh/install | bash

# 버전 확인
bun --version
```

### 3. SQLite 설치

=== "Linux"
    ```bash
    # Ubuntu/Debian
    sudo apt-get install sqlite3 libsqlite3-dev

    # Fedora
    sudo dnf install sqlite sqlite-devel
    ```

=== "macOS"
    ```bash
    brew install sqlite
    ```

=== "Windows"
    SQLite는 대부분의 Windows 시스템에 기본 포함되어 있습니다.

### 4. 프로젝트 클론 및 설정

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs
```

### 5. Backend 설정

```bash
cd backend

# .env 파일 생성
cat > .env << EOF
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
RUST_LOG=backend=debug,tower_http=debug
EOF

# 데이터 디렉토리 생성
mkdir -p data

# 의존성 체크 (선택사항)
cargo check

# 실행
cargo run
```

Backend는 `http://localhost:8080`에서 실행됩니다.

### 6. Frontend 설정

새 터미널 창에서:

```bash
cd frontend

# 의존성 설치
bun install

# 환경 변수 설정 (선택사항)
cat > .env << EOF
VITE_API_URL=http://localhost:8080
EOF

# 개발 서버 실행
bun run dev
```

Frontend는 `http://localhost:5173`에서 실행됩니다.

## 프로덕션 빌드

### Docker 프로덕션 이미지

```bash
# 프로덕션 이미지 빌드
docker compose -f docker-compose.prod.yml build

# 실행
docker compose -f docker-compose.prod.yml up -d
```

### 로컬 프로덕션 빌드

#### Backend

```bash
cd backend

# Release 빌드
cargo build --release

# 실행
./target/release/backend
```

#### Frontend

```bash
cd frontend

# 프로덕션 빌드
bun run build

# 프리뷰
bun run preview
```

## SQLx CLI 설치 (개발자용)

데이터베이스 마이그레이션을 관리하려면:

```bash
cargo install sqlx-cli --no-default-features --features sqlite

# 마이그레이션 실행
cd backend
sqlx migrate run

# 마이그레이션 되돌리기
sqlx migrate revert

# 새 마이그레이션 생성
sqlx migrate add <migration_name>
```

## 데이터베이스 초기화

데이터베이스를 초기 상태로 되돌리려면:

```bash
# Docker 사용 시
docker compose down -v
rm -rf backend/data/sqlite.db
docker compose up -d

# 로컬 개발 시
rm -rf backend/data/sqlite.db
cargo run
```

## 환경 변수 전체 목록

### Backend

| 변수 | 설명 | 기본값 | 필수 |
|------|------|--------|------|
| `DATABASE_URL` | SQLite 데이터베이스 경로 | `sqlite:data/sqlite.db?mode=rwc` | ✅ |
| `ADMIN_ID` | 관리자 ID | `admin` | ✅ |
| `ADMIN_PW` | 관리자 비밀번호 | `admin123` | ✅ |
| `RUST_LOG` | 로그 레벨 | `backend=debug,tower_http=debug` | ❌ |

### Frontend

| 변수 | 설명 | 기본값 | 필수 |
|------|------|--------|------|
| `VITE_API_URL` | Backend API URL | `http://localhost:8080` | ❌ |
| `PORT` | 프론트엔드 포트 | `5173` | ❌ |
| `HOST` | 바인딩 호스트 | `0.0.0.0` | ❌ |

## 검증

설치가 완료되었는지 확인:

### 1. Health Check

```bash
# Backend health check
curl http://localhost:8080/api/codelabs

# 응답 예시
[]
```

### 2. Frontend 접속

브라우저에서 [http://localhost:5173](http://localhost:5173) 접속

### 3. 관리자 로그인

1. [http://localhost:5173/login](http://localhost:5173/login) 접속
2. 설정한 자격증명으로 로그인

## 다음 단계

설치가 완료되었습니다! 이제:

- [첫 번째 Codelab 만들기](first-codelab.md)
- [공개 배포 가이드](../self-hosting/public-deployment.md)
- [API 레퍼런스](../specification/api-reference.md)

## 문제 해결

### 포트 충돌

다른 프로세스가 포트를 사용 중인 경우:

```bash
# 포트 사용 확인 (Linux/macOS)
lsof -i :8080
lsof -i :5173

# 프로세스 종료
kill -9 <PID>
```

### 데이터베이스 마이그레이션 오류

```bash
cd backend
rm -rf data/sqlite.db
cargo run
```

### Docker 빌드 오류

```bash
# 캐시 없이 다시 빌드
docker compose build --no-cache

# Docker 시스템 정리
docker system prune -a
```

더 많은 문제 해결은 [FAQ](../faq.md)를 참조하세요.
