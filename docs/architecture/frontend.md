# 프론트엔드 구조

SvelteKit 5 기반 Frontend 아키텍처를 설명합니다.

## 디렉토리 구조

```
frontend/src/
├── routes/                  # 페이지 라우트
│   ├── +layout.svelte      # 루트 레이아웃
│   ├── +page.svelte        # 홈페이지
│   ├── admin/              # 관리자
│   ├── codelabs/           # Codelab 뷰
│   └── login/              # 로그인
├── lib/                     # 라이브러리
│   ├── api.ts              # API 클라이언트
│   └── Progress.ts         # 진행 상황
└── app.css                 # 글로벌 스타일
```

## 주요 기능

### 1. 라우팅
파일 기반 라우팅 (SvelteKit)

### 2. 상태 관리
Svelte stores 및 context

### 3. API 통신
Fetch API 래퍼

### 4. 실시간 업데이트
WebSocket 클라이언트

## 다음 단계

- [Frontend 코드 예제](../code-guide/frontend-examples.md)
- [API 사용법](../code-guide/api-usage.md)
