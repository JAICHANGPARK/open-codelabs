# Open Codelabs (핸즈온 시스템)

[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/svelte-v5-ff3e00.svg)](https://svelte.dev/)
[![Bun](https://img.shields.io/badge/bun-v1.0+-black.svg)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/docker-blue.svg)](https://www.docker.com/)
[![Firebase](https://img.shields.io/badge/firebase-yellow.svg)](https://firebase.google.com/)

**Open Codelabs**는 Google Codelab 스타일의 핸즈온 세션을 손쉽게 운영하고 관리할 수 있도록 설계된 오픈 소스 플랫폼입니다. 최신 기술 스택으로 구축되었으며, 퍼실리테이터(관리자)와 참가자
역할을 모두 지원합니다. 콘텐츠는 Markdown으로 직접 관리하거나 AI를 통해 자동으로 생성할 수 있습니다.

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md)

---

## 🚀 주요 특징

- **퍼실리테이터 & 참가자 분리**: 관리자는 코드랩을 생성 및 관리하고, 참가자는 정교하게 설계된 UI를 통해 단계를 따라갈 수 있습니다.
- **AI 코드랩 생성기**: Google Gemini AI를 사용하여 소스 코드나 참조 문서로부터 전문가 수준의 코드랩을 자동으로 생성합니다.
- **퀴즈·피드백·수료증**: 퀴즈와 피드백 제출을 수료 조건으로 설정하고, 검증 URL이 포함된 수료증을 자동 발급합니다.
- **준비 가이드 & 자료 관리**: 사전 준비 가이드를 직접 작성하거나 AI로 생성하고, 링크/파일을 한 곳에서 배포합니다.
- **라이브 워크숍 도구**: 실시간 채팅/DM, 도움 요청 큐, 제출물 패널, 수료증 보유자만 대상인 룰렛 추첨 기능을 제공합니다.
- **멀티 런타임 지원**: 로컬/프라이빗 세션을 위한 **Rust (Axum) + SQLite** 백엔드 실행 또는 서버리스 환경을 위한 **Firebase (Firestore/Hosting)** 배포를
  지원합니다.
- **Google Codelab Look & Feel**: 익숙하고 가독성 높은 구글 스타일의 디자인을 차용했습니다.
- **간편한 외부 공개**: `ngrok` 및 `bore` 통합 스크립트를 통해 로컬 서버를 즉시 외부에 공개하고 QR 코드로 접속할 수 있게 지원합니다.
- **다국어 지원**: 글로벌 워크숍 운영을 위한 i18n 지원이 내장되어 있습니다.

---

## ⚡ 퀵스타트 (Quickstart)

단 몇 초 만에 시스템을 실행해보세요:

```bash
# 저장소 복제
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs

# Docker Compose로 시작
docker compose up --build
```

### 🦭 Podman 사용자 가이드

Podman을 사용하는 경우 `podman-compose`를 사용할 수 있습니다:

```bash
podman-compose up --build
```

또는 Podman의 Docker 호환 레이어를 사용하세요.

---

## 🛠 기술 스택

### Frontend

- **Framework**: [SvelteKit 5](https://svelte.dev/) (Vite + TypeScript)
- **Runtime**: [Bun](https://bun.sh/)
- **Styling**: Tailwind CSS 4.0
- **State Management**: Svelte Runes
- **i18n**: `svelte-i18n`

### Backend (자체 호스팅)

- **Language**: [Rust](https://www.rust-lang.org/)
- **Framework**: Axum (Tokio stack)
- **Database**: SQLite (via [SQLx](https://github.com/launchbadge/sqlx))

### Cloud (서버리스 옵션)

- **Platform**: [Firebase](https://firebase.google.com/) (Hosting, Firestore, Storage)

---

## 📂 프로젝트 구조

```text
open-codelabs/
├── backend/          # Rust Axum API 서버
│   ├── src/          # API 로직
│   └── migrations/   # 데이터베이스 마이그레이션
├── frontend/         # SvelteKit 클라이언트
│   ├── src/          # 컴포넌트, 라우트 및 라이브러리
│   └── static/       # 정적 에셋
├── docs/             # 문서 (MkDocs)
├── docker-compose.yml # 전체 시스템 오케스트레이션
└── run-public.sh     # 공개 배포 스크립트 (ngrok/bore)
```

---

## 🚦 시작하기

### 사전 준비 사항

- [Docker](https://www.docker.com/) & Docker Compose
- [Bun](https://bun.sh/) (로컬 개발용)
- [Rust](https://www.rust-lang.org/) (로컬 백엔드 개발용)

### 1. Docker로 실행 (권장)

가장 간단하게 전체 시스템을 실행하는 방법입니다.

> **참고**: 기본적으로 데이터는 호스트 머신의 `~/open-codelabs` 폴더에 저장됩니다. 저장 위치를 변경하려면 루트 디렉토리의 `.env` 파일에서 `DATA_VOLUME_PATH`를 수정하세요.
> - **macOS/Linux**: `~/open-codelabs`
> - **Windows**: `C:/open-codelabs` (슬래시 `/` 사용 권장)

```bash
docker-compose up --build
```

- **Frontend**: [http://localhost:5173](http://localhost:5173)
- **Backend API**: [http://localhost:8080](http://localhost:8080)

### 2. 로컬 개발 환경

#### Backend

```bash
cd backend
# .env 파일 생성 (DATABASE_URL=sqlite:data/sqlite.db?mode=rwc)
cargo run
```

#### Frontend

```bash
cd frontend
bun install
# .env 파일 생성 (VITE_API_URL=http://localhost:8080)
bun run dev
```

### 3. 클라우드 배포 (AWS / GCP / Firebase)

서버리스 환경이나 클라우드 인프라에서 운영하려면 AWS, GCP 또는 Firebase를 사용할 수 있습니다.

- **AWS**: 컨테이너 기반 또는 VM 배포. [AWS 배포 가이드](docs/self-hosting/aws.md) 참조.
- **GCP (Cloud Run)**: 컨테이너 기반 배포. [GCP 배포 가이드](docs/self-hosting/gcp.md) 참조.
- **Firebase**: 가장 빠른 서버리스 설정. [Firebase 배포 가이드](docs/self-hosting/firebase.md) 참조.

---

## 🤖 AI 코드랩 생성기

Open Codelabs에는 코드를 구조화된 튜토리얼로 변환하는 AI 생성기가 내장되어 있습니다.

1. 설정에서 Gemini API 키를 입력합니다.
2. 소스 코드나 기술 설명을 제공합니다.
3. AI가 각 단계, 설명 및 검증 과정을 자동으로 생성합니다.

---

## 🧭 퍼실리테이터 툴킷 (신규)
- **라이브 탭**: 참석자 목록, 실시간 채팅/DM, 도움 요청 처리.
- **퀴즈 & 피드백**: 수료 조건 설정 및 결과 집계.
- **준비 가이드 & 자료**: 세션 전 준비 가이드를 작성/AI 생성하고 링크·파일을 배포.
- **제출물 관리**: 참석자 업로드 파일을 수집·검토.
- **수료증 룰렛 추첨**: 수료증 발급이 완료된 참석자만 대상으로 공정하게 추첨.

---

## 🌐 외부 공개하기 (ngrok / bore)

로컬 장비에서 워크숍을 진행할 때 `run-public.sh` 스크립트를 사용하여 외부 접속 주소를 생성할 수 있습니다.

```bash
chmod +x run-public.sh
./run-public.sh --ngrok  # ngrok 사용
# 또는
./run-public.sh --bore   # bore 사용 (Rust 기반)
```

---

## 📚 문서

전체 문서는 GitHub Pages에서 확인할 수 있습니다:
**[📖 Open Codelabs 문서 보기](https://JAICHANGPARK.github.io/open-codelabs/)**

---

## 📄 라이선스

이 프로젝트는 [Apache License 2.0](LICENSE)를 따릅니다.
