# Docker로 배포하기

Docker를 사용하여 Open Codelabs를 배포하는 완전한 가이드입니다.

## 기본 배포

### docker-compose.yml 구조

프로젝트의 `docker-compose.yml` 파일:

```yaml
services:
  backend:
    build: ./backend
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
      - ADMIN_ID=admin
      - ADMIN_PW=admin123
    volumes:
      - ./backend/data:/app/data

  frontend:
    build: ./frontend
    ports:
      - "5173:5173"
    environment:
      - VITE_API_URL=http://backend:8080
      - PORT=5173
      - HOST=0.0.0.0
    depends_on:
      - backend
```

### 기본 실행

```bash
# 빌드 및 실행
docker-compose up --build

# 백그라운드 실행
docker-compose up -d

# 로그 확인
docker-compose logs -f

# 중지
docker-compose down
```

## Backend Dockerfile

`backend/Dockerfile`:

```dockerfile
# Multi-stage build for optimal image size
FROM rust:1.75 AS builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install required libraries
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /app/target/release/backend /app/backend

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Create data directory
RUN mkdir -p /app/data

# Expose port
EXPOSE 8080

# Run the application
CMD ["./backend"]
```

### 주요 특징

- **Multi-stage build**: 최종 이미지 크기 최소화
- **Release mode**: 최적화된 바이너리
- **런타임 의존성만 포함**: Rust 컴파일러 불필요
- **마이그레이션 포함**: 자동 DB 초기화

## Frontend Dockerfile

`frontend/Dockerfile`:

```dockerfile
FROM oven/bun:1 AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
RUN bun run build

# Runtime stage
FROM oven/bun:1-slim

WORKDIR /app

# Copy built files
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./

# Install production dependencies only
RUN bun install --production

EXPOSE 5173

# Run with bun
CMD ["bun", "run", "build/index.js"]
```

## 환경 변수 설정

### .env 파일 사용

`docker-compose.yml`에서 환경 파일 참조:

```yaml
services:
  backend:
    env_file:
      - backend/.env
    build: ./backend
    # ...
```

`backend/.env`:

```bash
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=SecurePassword123!
RUST_LOG=backend=info,tower_http=info
```

### 보안 권장사항

!!! danger "프로덕션 보안"
    - 절대 기본 비밀번호(`admin123`)를 사용하지 마세요
    - `.env` 파일을 Git에 커밋하지 마세요
    - 강력한 비밀번호 사용 (20자 이상 권장)

```bash
# 강력한 비밀번호 생성
openssl rand -base64 32
```

## 데이터 영속성

### Volume 설정

데이터베이스 데이터를 유지하려면 볼륨 사용:

#### SQLite 사용 시
```yaml
services:
  backend:
    volumes:
      - ./backend/data:/app/data          # 호스트 디렉토리
      - backend_data:/app/data            # Docker 볼륨 (권장)
    # ...

volumes:
  backend_data:
```

#### PostgreSQL 사용 시 (예시)
```yaml
services:
  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=codelab
      - POSTGRES_PASSWORD=secure_password
      - POSTGRES_DB=open_codelabs
    volumes:
      - postgres_data:/var/lib/postgresql/data

  backend:
    environment:
      - DATABASE_URL=postgres://codelab:secure_password@db:5432/open_codelabs
    depends_on:
      - db
    # ...

volumes:
  postgres_data:
```

### 백업 전략

#### 데이터베이스 백업

```bash
# SQLite 데이터베이스 백업
docker-compose exec backend sqlite3 /app/data/sqlite.db ".backup /app/data/backup.db"

# 호스트로 복사
docker cp <container_id>:/app/data/backup.db ./backup.db
```

#### 자동 백업 스크립트

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# 데이터베이스 백업
docker-compose exec -T backend sqlite3 /app/data/sqlite.db ".backup /app/data/backup_$TIMESTAMP.db"
docker cp $(docker-compose ps -q backend):/app/data/backup_$TIMESTAMP.db $BACKUP_DIR/

echo "Backup created: $BACKUP_DIR/backup_$TIMESTAMP.db"

# 30일 이상 된 백업 삭제
find $BACKUP_DIR -name "backup_*.db" -mtime +30 -delete
```

```bash
# 실행 권한 부여
chmod +x backup.sh

# Cron으로 매일 자동 백업
0 2 * * * /path/to/backup.sh
```

## 리버스 프록시 설정

### Nginx를 사용한 프록시

`docker-compose.yml`에 nginx 추가:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - frontend
      - backend

  backend:
    # ports를 expose로 변경 (외부 노출 안 함)
    expose:
      - "8080"

  frontend:
    expose:
      - "5173"
```

`nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    upstream frontend {
        server frontend:5173;
    }

    upstream backend {
        server backend:8080;
    }

    server {
        listen 80;
        server_name your-domain.com;

        # Frontend
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Backend API
        location /api {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # WebSocket support
        location /api/ws {
            proxy_pass http://backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
        }
    }
}
```

### HTTPS/SSL 설정

Let's Encrypt 사용:

```yaml
services:
  certbot:
    image: certbot/certbot
    volumes:
      - ./certs:/etc/letsencrypt
      - ./certbot-data:/var/www/certbot
    command: certonly --webroot --webroot-path=/var/www/certbot --email your@email.com --agree-tos --no-eff-email -d your-domain.com
```

HTTPS nginx 설정:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /etc/nginx/certs/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/nginx/certs/live/your-domain.com/privkey.pem;

    # SSL 설정
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # ... 나머지 설정
}

# HTTP를 HTTPS로 리다이렉트
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

## 프로덕션 최적화

### 리소스 제한

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

### Health Checks

```yaml
services:
  backend:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/codelabs"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  frontend:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5173"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### 자동 재시작

```yaml
services:
  backend:
    restart: unless-stopped

  frontend:
    restart: unless-stopped
```

## Docker Compose 명령어 참조

### 기본 명령어

```bash
# 빌드만
docker-compose build

# 캐시 없이 빌드
docker-compose build --no-cache

# 백그라운드 실행
docker-compose up -d

# 특정 서비스만 실행
docker-compose up backend

# 스케일링 (여러 인스턴스)
docker-compose up --scale backend=3

# 중지
docker-compose stop

# 재시작
docker-compose restart

# 완전 삭제 (볼륨 포함)
docker-compose down -v
```

### 로그 및 모니터링

```bash
# 모든 로그
docker-compose logs

# 실시간 로그
docker-compose logs -f

# 특정 서비스 로그
docker-compose logs -f backend

# 최근 100줄만
docker-compose logs --tail=100

# 타임스탬프 포함
docker-compose logs -t
```

### 디버깅

```bash
# 컨테이너 접속
docker-compose exec backend sh

# 명령 실행
docker-compose exec backend ls /app/data

# 파일 복사 (컨테이너 → 호스트)
docker cp <container_id>:/app/data/sqlite.db ./

# 파일 복사 (호스트 → 컨테이너)
docker cp ./config.toml <container_id>:/app/
```

### 정리

```bash
# 중지된 컨테이너 제거
docker-compose rm

# 사용하지 않는 이미지 제거
docker image prune

# 전체 시스템 정리
docker system prune -a
```

## 문제 해결

### 마이그레이션 오류 (Checksum Mismatch)

최근 업데이트로 PostgreSQL/MySQL 지원을 위해 마이그레이션 파일이 수정되었습니다. 기존 SQLite 사용자 중 `Error: migration ... was previously applied but has been modified` 에러가 발생하는 경우 다음 방법 중 하나를 시도하세요:

1. **데이터베이스 초기화 (권장)**: 기존 데이터를 보존할 필요가 없다면 SQLite 파일을 삭제하고 다시 시작합니다.
   ```bash
   rm backend/data/sqlite.db
   docker-compose up --build
   ```
2. **PostgreSQL/MySQL로 전환**: 새로운 데이터베이스를 사용하는 경우 이 문제가 발생하지 않습니다. [환경 변수 가이드](environment.md)를 참조하여 `DATABASE_URL`을 설정하세요.

### 컨테이너가 계속 재시작됨

```bash
# 로그 확인
docker-compose logs backend

# 수동으로 실행하여 에러 확인
docker-compose run backend sh
```

### 포트 충돌

```bash
# 사용 중인 포트 확인
sudo lsof -i :8080

# docker-compose.yml에서 포트 변경
ports:
  - "8081:8080"  # 호스트:컨테이너
```

### 디스크 공간 부족

```bash
# Docker가 사용 중인 공간 확인
docker system df

# 정리
docker system prune -a --volumes
```

### 네트워크 문제

```bash
# 네트워크 재생성
docker-compose down
docker network prune
docker-compose up
```

## 다음 단계

- [로컬 개발 환경](local-development.md) - 개발 환경 구성
- [공개 배포](public-deployment.md) - ngrok으로 외부 공개
- [환경 변수](environment.md) - 상세 설정 가이드

## 참고 자료

- [Docker Compose 공식 문서](https://docs.docker.com/compose/)
- [Docker 보안 베스트 프랙티스](https://docs.docker.com/develop/security-best-practices/)
- [Multi-stage builds](https://docs.docker.com/develop/develop-images/multistage-build/)
