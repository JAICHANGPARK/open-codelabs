# AntiGravity Hands-on System (Open Codelabs)

[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/svelte-v5-ff3e00.svg)](https://svelte.dev/)
[![Bun](https://img.shields.io/badge/bun-v1.0+-black.svg)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/docker-blue.svg)](https://www.docker.com/)

**AntiGravity Hands-on System**은 Google Codelab 스타일의 핸즈온 세션을 손쉽게 운영할 수 있도록 설계된 오픈 소스 Codelab 플랫폼입니다.

## 🎯 프로젝트 개요

이 프로젝트는 교육자(Facilitator)가 단계별 가이드를 만들고, 참가자(Attendee)가 자신의 속도에 맞춰 학습할 수 있는 대화형 핸즈온 플랫폼입니다. SaaS 아키텍처를 기반으로 하며, Markdown을 통해 콘텐츠를 관리합니다.

## ✨ 주요 특징

### 🎭 역할 기반 시스템
- **Facilitator (관리자)**: Codelab 생성, 편집, 참가자 관리
- **Attendee (참가자)**: 단계별 학습, 진행 상황 추적, 실시간 도움 요청

### 🎨 Google Codelab 스타일 UI
- 익숙하고 직관적인 사용자 인터페이스
- 반응형 디자인으로 모바일/태블릿 지원
- 다크 모드 지원

### 💬 실시간 상호작용
- WebSocket 기반 실시간 채팅
- 1:1 DM 기능
- 도움 요청 및 관리 시스템
- 참가자 진행 상황 실시간 모니터링

### 📝 Markdown 기반 콘텐츠
- 간편한 콘텐츠 작성
- 코드 하이라이팅 지원
- 이미지 업로드 및 관리
- Import/Export 기능

### 🚀 쉬운 배포
- Docker 기반 원클릭 배포
- ngrok/bore를 통한 로컬 서버 공개
- QR 코드로 참가자 초대
- SQLite로 가볍게 시작, 확장 가능

## 🏗️ 기술 스택

### Frontend
- **Framework**: SvelteKit 5 (Vite + TypeScript)
- **Runtime**: Bun
- **Styling**: Tailwind CSS 4.0
- **Markdown**: marked & dompurify
- **Icons**: Lucide Svelte
- **QR Code**: svelte-qrcode

### Backend
- **Language**: Rust
- **Framework**: Axum (Tokio async runtime)
- **Database**: SQLite with SQLx
- **WebSocket**: Axum WebSocket support
- **Serialization**: Serde (JSON)

### DevOps
- Docker & Docker Compose
- Multi-stage builds
- ngrok/bore for tunneling

## 🚀 빠른 시작

Docker로 전체 시스템을 실행하는 가장 간단한 방법:

```bash
docker-compose up --build
```

- **Frontend**: [http://localhost:5173](http://localhost:5173)
- **Backend API**: [http://localhost:8080](http://localhost:8080)

더 자세한 설치 가이드는 [설치 가이드](getting-started/installation.md)를 참조하세요.

## 📚 문서 구조

- **[시작하기](getting-started/quickstart.md)**: 빠른 시작 가이드 및 설치 방법
- **[Self-Hosting](self-hosting/docker.md)**: Docker, 로컬 개발, 공개 배포 가이드
- **[프로젝트 명세](specification/overview.md)**: 기능 명세 및 API 레퍼런스
- **[아키텍처](architecture/system-architecture.md)**: 시스템 구조 및 설계 문서
- **[코드 가이드](code-guide/backend-examples.md)**: 코드 예제 및 사용법
- **[기여하기](contributing/guide.md)**: 프로젝트 기여 방법
- **[FAQ](faq.md)**: 자주 묻는 질문

## 🤝 기여하기

프로젝트에 기여하고 싶으신가요? [기여 가이드](contributing/guide.md)를 확인해주세요!

## 📄 라이선스

이 프로젝트는 [MIT License](license.md)를 따릅니다.

## 🔗 링크

- [GitHub Repository](https://github.com/JAICHANGPARK/open-codelabs)
- [Issue Tracker](https://github.com/JAICHANGPARK/open-codelabs/issues)
- [Pull Requests](https://github.com/JAICHANGPARK/open-codelabs/pulls)
