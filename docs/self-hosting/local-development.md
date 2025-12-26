# 로컬 개발 환경

개발자를 위한 로컬 개발 환경 구성 가이드입니다.

## 개발 환경 설정

### 필수 도구 설치

=== "macOS"
    ```bash
    # Homebrew로 설치
    brew install rust bun sqlite

    # 버전 확인
    rustc --version
    cargo --version
    bun --version
    ```

=== "Linux (Ubuntu/Debian)"
    ```bash
    # Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env

    # Bun
    curl -fsSL https://bun.sh/install | bash

    # SQLite
    sudo apt-get install sqlite3 libsqlite3-dev
    ```

=== "Windows (WSL2)"
    WSL2 Ubuntu에서 Linux 설치 방법을 따르세요.

### IDE 설정

#### VS Code (권장)

필수 확장:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "svelte.svelte-vscode",
    "oven.bun-vscode",
    "tamasfe.even-better-toml",
    "serayuzgur.crates"
  ]
}
```

`.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

#### IntelliJ IDEA / RustRover

플러그인:

- Rust
- Svelte
- Bun
- TOML

## Backend 개발 환경

### 프로젝트 구조

```
backend/
├── Cargo.toml              # 의존성 설정
├── .env                    # 환경 변수
├── src/
│   ├── main.rs            # 엔트리 포인트
│   ├── models.rs          # 데이터 모델
│   ├── state.rs           # 애플리케이션 상태
│   └── handlers/          # API 핸들러
│       ├── mod.rs
│       ├── admin.rs
│       ├── codelabs.rs
│       ├── attendees.rs
│       ├── feedback.rs
│       ├── upload.rs
│       └── websocket.rs
├── migrations/            # 데이터베이스 마이그레이션
│   ├── 20251226161500_init.sql
│   ├── 20251226161600_attendees.sql
│   └── ...
└── data/                  # SQLite 데이터베이스
    └── sqlite.db
```

### 개발 서버 실행

```bash
cd backend

# 환경 변수 설정
cat > .env << EOF
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
RUST_LOG=backend=debug,tower_http=debug,sqlx=info
EOF

# 데이터 디렉토리 생성
mkdir -p data

# 의존성 확인
cargo check

# 개발 모드로 실행
cargo run

# 또는 watch 모드 (파일 변경 시 자동 재시작)
cargo install cargo-watch
cargo watch -x run
```

### 코드 품질 도구

#### Clippy (Lint)

```bash
# Clippy 실행
cargo clippy

# 모든 경고를 에러로 처리
cargo clippy -- -D warnings

# 자동 수정 (안전한 것만)
cargo clippy --fix
```

#### Rustfmt (포맷팅)

```bash
# 포맷팅 확인
cargo fmt --check

# 자동 포맷팅
cargo fmt
```

`.rustfmt.toml`:

```toml
max_width = 100
hard_tabs = false
tab_spaces = 4
edition = "2021"
```

#### 테스트

```bash
# 모든 테스트 실행
cargo test

# 특정 테스트만
cargo test test_name

# 출력 보기
cargo test -- --nocapture

# 단일 스레드로 실행
cargo test -- --test-threads=1
```

### 데이터베이스 작업

#### SQLx CLI 설치

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

#### 마이그레이션 관리

```bash
# 새 마이그레이션 생성
sqlx migrate add create_new_table

# 마이그레이션 실행
sqlx migrate run

# 마이그레이션 되돌리기
sqlx migrate revert

# 마이그레이션 상태 확인
sqlx migrate info
```

#### 데이터베이스 쿼리 검증

컴파일 타임에 SQL 쿼리 검증:

```bash
# 쿼리 메타데이터 준비
cargo sqlx prepare

# CI에서 검증
cargo sqlx prepare --check
```

#### 데이터베이스 초기화

```bash
# 데이터베이스 삭제 및 재생성
rm data/sqlite.db
cargo run

# 또는
sqlx database reset
```

### 디버깅

#### 로그 레벨 조정

`.env`:

```bash
# 전체 디버그
RUST_LOG=debug

# 모듈별 설정
RUST_LOG=backend=debug,tower_http=info,sqlx=warn

# 특정 핸들러만
RUST_LOG=backend::handlers::codelabs=trace
```

#### LLDB 디버거

```bash
# 디버그 빌드
cargo build

# LLDB로 실행
rust-lldb target/debug/backend

# VS Code launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Backend",
      "cargo": {
        "args": ["build", "--bin=backend"]
      },
      "args": [],
      "cwd": "${workspaceFolder}/backend"
    }
  ]
}
```

## Frontend 개발 환경

### 프로젝트 구조

```
frontend/
├── package.json
├── bun.lock
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
├── src/
│   ├── app.html          # HTML 템플릿
│   ├── app.css           # 글로벌 스타일
│   ├── lib/              # 라이브러리
│   │   ├── api.ts        # API 클라이언트
│   │   ├── Progress.ts   # 진행 상황 관리
│   │   └── i18n/         # 다국어 지원
│   └── routes/           # 페이지 라우트
│       ├── +layout.svelte
│       ├── +page.svelte
│       ├── admin/
│       │   ├── +page.svelte
│       │   └── [id]/
│       ├── codelabs/
│       │   └── [id]/
│       └── login/
└── static/               # 정적 파일
    └── favicon.png
```

### 개발 서버 실행

```bash
cd frontend

# 의존성 설치
bun install

# 개발 서버 (Hot reload)
bun run dev

# 포트 변경
bun run dev --port 3000

# 네트워크 노출 (모바일 테스트)
bun run dev --host
```

### 코드 품질 도구

#### Svelte Check

```bash
# 타입 체크
bun run check

# Watch 모드
bun run check:watch
```

#### ESLint & Prettier

```bash
# ESLint 설치
bun add -d eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin

# Prettier 설치
bun add -d prettier prettier-plugin-svelte

# 실행
bunx eslint src
bunx prettier --write src
```

`.prettierrc`:

```json
{
  "useTabs": true,
  "singleQuote": true,
  "trailingComma": "es5",
  "printWidth": 100,
  "plugins": ["prettier-plugin-svelte"],
  "overrides": [
    {
      "files": "*.svelte",
      "options": {
        "parser": "svelte"
      }
    }
  ]
}
```

### 빌드 및 프리뷰

```bash
# 프로덕션 빌드
bun run build

# 빌드 결과 프리뷰
bun run preview

# 빌드 분석
bun run build -- --mode analyze
```

### 환경 변수

`.env`:

```bash
# Backend API URL
VITE_API_URL=http://localhost:8080

# 다른 환경
VITE_API_URL=https://api.example.com
```

코드에서 사용:

```typescript
const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080';
```

## 통합 개발 워크플로우

### Tmux를 사용한 멀티 터미널

`dev.sh`:

```bash
#!/bin/bash

# Tmux 세션 생성
tmux new-session -d -s codelabs

# Backend 창
tmux rename-window -t codelabs:0 'backend'
tmux send-keys -t codelabs:0 'cd backend && cargo watch -x run' C-m

# Frontend 창
tmux new-window -t codelabs:1 -n 'frontend'
tmux send-keys -t codelabs:1 'cd frontend && bun run dev' C-m

# 로그 창
tmux new-window -t codelabs:2 -n 'logs'

# 세션 연결
tmux attach-session -t codelabs
```

```bash
chmod +x dev.sh
./dev.sh
```

### Make를 사용한 자동화

`Makefile`:

```makefile
.PHONY: dev backend frontend check test clean

# 전체 개발 환경 시작
dev:
	@echo "Starting development environment..."
	@make -j2 backend frontend

# Backend 실행
backend:
	cd backend && cargo watch -x run

# Frontend 실행
frontend:
	cd frontend && bun run dev

# 코드 검증
check:
	cd backend && cargo clippy && cargo fmt --check
	cd frontend && bun run check

# 테스트
test:
	cd backend && cargo test
	cd frontend && bun test

# 빌드
build:
	cd backend && cargo build --release
	cd frontend && bun run build

# 정리
clean:
	cd backend && cargo clean
	cd frontend && rm -rf node_modules .svelte-kit build
```

사용:

```bash
make dev      # 개발 환경 시작
make check    # 코드 검증
make test     # 테스트 실행
make build    # 프로덕션 빌드
make clean    # 정리
```

## Hot Reload 설정

### Backend Auto-reload

`cargo-watch` 사용:

```bash
# 설치
cargo install cargo-watch

# 파일 변경 감지 및 재시작
cargo watch -x run

# 특정 파일만 감지
cargo watch -w src -x run

# 빌드 + 테스트 + 실행
cargo watch -x check -x test -x run
```

### Frontend HMR

Vite가 자동으로 HMR(Hot Module Replacement)을 지원합니다.

`vite.config.ts` 커스터마이징:

```typescript
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		strictPort: false,
		hmr: {
			overlay: true
		},
		watch: {
			usePolling: true  // Docker에서 필요
		}
	}
});
```

## 디버깅 팁

### Backend API 테스트

#### curl

```bash
# Codelab 목록 조회
curl http://localhost:8080/api/codelabs

# Codelab 생성
curl -X POST http://localhost:8080/api/codelabs \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","description":"Test Desc","author":"Me"}'

# 로그인
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{"admin_id":"admin","admin_pw":"admin123"}'
```

#### HTTPie (추천)

```bash
# 설치
brew install httpie  # macOS
pip install httpie   # Python

# 사용
http GET http://localhost:8080/api/codelabs
http POST http://localhost:8080/api/codelabs title="Test" description="Desc" author="Me"
```

### WebSocket 테스트

`websocat` 사용:

```bash
# 설치
brew install websocat

# WebSocket 연결
websocat ws://localhost:8080/api/ws/codelab_id

# 메시지 전송 (JSON)
{"type":"chat","message":"Hello"}
```

### 브라우저 DevTools

- **Network 탭**: API 요청/응답 확인
- **Console**: 에러 로그 확인
- **Application**: LocalStorage, SessionStorage 확인

## 성능 프로파일링

### Backend 프로파일링

```bash
# Flamegraph 생성
cargo install flamegraph
cargo flamegraph

# 결과: flamegraph.svg
```

### Frontend 번들 분석

```bash
# 번들 분석
bun run build
bunx vite-bundle-visualizer
```

## 다음 단계

- [공개 배포](public-deployment.md) - ngrok으로 외부 공개
- [환경 변수](environment.md) - 상세 설정 가이드
- [기여 가이드](../contributing/guide.md) - 코드 기여 방법
