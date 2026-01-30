# 프론트엔드 구조

SvelteKit 5 기반 Frontend 아키텍처를 설명합니다.

## 디렉토리 구조

```
frontend/src/
├── routes/                  # 페이지 라우트
│   ├── +layout.svelte      # 루트 레이아웃
│   ├── +page.svelte        # 홈
│   ├── admin/              # 관리자
│   │   ├── +page.svelte
│   │   ├── [id]/+page.svelte
│   │   └── audit-logs/+page.svelte
│   ├── codelabs/           # Codelab 뷰
│   │   ├── +page.svelte
│   │   └── [id]/+page.svelte
│   ├── codelabs/[id]/entry/+page.svelte
│   ├── codelabs/[id]/live/+page.svelte
│   ├── certificate/[id]/+page.svelte
│   ├── verify/[id]/+page.svelte
│   └── login/+page.svelte  # 로그인
├── lib/                     # 라이브러리
│   ├── api.ts              # API 라우터
│   ├── api-backend.ts      # Backend API
│   ├── api-firebase.ts     # Firebase API
│   ├── api-supabase.ts     # Supabase API
│   ├── components/         # 공용 컴포넌트
│   ├── i18n/               # 다국어 리소스
│   ├── types.ts            # 공용 타입
│   ├── markdown.ts         # Markdown 유틸
│   ├── crypto.ts           # 암호화 유틸
│   ├── gemini.ts           # AI 연동
│   ├── tts.ts              # TTS
│   ├── playground.ts       # 실행 예제
│   ├── uploadFilters.ts    # 업로드 필터
│   ├── theme.svelte.ts     # 테마 상태
│   └── utils.ts            # 유틸리티
├── hooks.server.ts         # 서버 훅 (프록시)
├── app.css                 # 글로벌 스타일
└── app.html                # HTML 템플릿
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
