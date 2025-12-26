# AntiGravity Hands-on System (Open Codelabs)

[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/svelte-v5-ff3e00.svg)](https://svelte.dev/)
[![Bun](https://img.shields.io/badge/bun-v1.0+-black.svg)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/docker-blue.svg)](https://www.docker.com/)

**AntiGravity Hands-on System**은 Google Codelab 스타일의 핸즈온 세션을 손쉽게 운영할 수 있도록 설계된 오픈 소스 Codelab 플랫폼입니다. SaaS 아키텍처를 기반으로 Facilitator와 Attendee 역할을 지원하며, Markdown 기반으로 콘텐츠를 관리합니다.

---

## 주요 특징

- **Facilitator & Attendee 분리**: 관리자는 코드랩을 생성 및 관리하고, 참가자는 정교하게 설계된 UI를 통해 단계를 따라갈 수 있습니다.
- **Google Codelab Look & Feel**: 익숙하고 가독성 높은 구글 스타일의 디자인을 차용했습니다.
- **Local-First & SaaS Ready**: 개발 단계에서는 SQLite를 사용하여 가볍게 운영하며, Docker를 통해 어디서든 배포 가능합니다.
- **Easy Public Access**: `ngrok` 통합 스크립트를 통해 로컬 서버를 즉시 외부에 공개하고 QR 코드로 접속할 수 있게 지원합니다.

---

## 기술 스택

### Frontend
- **Framework**: [SvelteKit 5](https://svelte.dev/) (Vite + TypeScript)
- **Runtime**: [Bun](https://bun.sh/)
- **Styling**: Tailwind CSS 4.0
- **Markdown**: `marked` & `dompurify`
- **Icon**: Lucide Svelte
- **Utilities**: `svelte-qrcode` (참가자 접속용)

### Backend
- **Language**: [Rust](https://www.rust-lang.org/)
- **Framework**: Axum (Tokio stack)
- **Database**: SQLite (via [SQLx](https://github.com/launchbadge/sqlx))
- **Serialization**: Serde (JSON)
- **Logging**: Tracing

### DevOps
- **Container**: Docker, Docker Compose (Multi-stage builds)
- **Expose**: ngrok

---

## 프로젝트 구조

```text
open-codelabs/
├── backend/          # Rust Axum API 서버
│   ├── src/          # API 로직
│   ├── migrations/   # SQLx 데이터베이스 마이그레이션
│   └── data/         # SQLite DB 파일 저장소
├── frontend/         # SvelteKit 클라이언트
│   ├── src/          # 컴포넌트 및 페이지
│   └── static/       # 정적 에셋
├── docker-compose.yml # 전체 시스템 오케스트레이션
└── run-public.sh     # ngrok 기반 공개 배포 스크립트
```

---

## 시작하기

### 사전 준비 사항
- [Docker](https://www.docker.com/) & Docker Compose
- [Bun](https://bun.sh/) (로컬 개발용)
- [Rust](https://www.rust-lang.org/) (로컬 개발용)
- [ngrok](https://ngrok.com/) (외부 공개용)

### 1. Docker로 전체 시스템 실행 (권장)
가장 간단하게 전체 시스템을 실행하는 방법입니다.

```bash
docker-compose up --build
```
- **Frontend**: [http://localhost:5173](http://localhost:5173)
- **Backend API**: [http://localhost:8080](http://localhost:8080)

### 2. 로컬 개발 환경 실행

#### Backend
```bash
cd backend
# .env 설정 (DATABASE_URL=sqlite:data/sqlite.db)
cargo run
```

#### Frontend
```bash
cd frontend
bun install
bun run dev
```

### 3. 세션 공개하기 (ngrok 또는 bore 활용)
행사나 워크샵 환경에서 로컬 장비를 서버로 사용할 때 유용합니다.

#### ngrok 사용 (기본)
```bash
chmod +x run-public.sh
./run-public.sh --ngrok
```

#### bore 사용 (Rust 기반 대안)
[bore](https://github.com/ekzhang/bore)가 설치되어 있어야 합니다 (`cargo install bore-cli`).
```bash
chmod +x run-public.sh
./run-public.sh --bore
```

### 4. Podman 사용자 가이드
Docker 대신 [Podman](https://podman.io/)을 사용하는 경우, `podman-compose`가 설치되어 있어야 합니다. `run-public.sh` 스크립트는 자동으로 `podman-compose`를 감지하여 실행합니다.

직접 실행하려면:
```bash
podman-compose up --build
```
---

---

## 📚 문서

완전한 문서는 GitHub Pages에서 확인하세요:

**[📖 Open Codelabs 문서 보기](https://JAICHANGPARK.github.io/open-codelabs/)**

### 주요 문서

- [빠른 시작](https://JAICHANGPARK.github.io/open-codelabs/getting-started/quickstart/) - 5분 안에 시작하기
- [설치 가이드](https://JAICHANGPARK.github.io/open-codelabs/getting-started/installation/) - 상세 설치 방법
- [API 레퍼런스](https://JAICHANGPARK.github.io/open-codelabs/specification/api-reference/) - REST API 문서
- [기여 가이드](https://JAICHANGPARK.github.io/open-codelabs/contributing/guide/) - 프로젝트 기여 방법
- [FAQ](https://JAICHANGPARK.github.io/open-codelabs/faq/) - 자주 묻는 질문

### 로컬에서 문서 보기

```bash
# MkDocs 설치
pip install -r requirements.txt

# 문서 서버 실행
mkdocs serve

# http://localhost:8000 에서 확인
```

---

## 라이선스

이 프로젝트는 [Apache License 2.0](LICENSE)를 따릅니다.
