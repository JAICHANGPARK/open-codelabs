# Open Codelabs Firebase Deployment Guide

이 문서는 서버(Rust/SQLite)를 직접 운영하기 어려운 사용자를 위해 Firebase(Hosting + Firestore)를 사용하여 Open Codelabs를 배포하는 방법을 설명합니다.

## 1. Firebase 프로젝트 준비

1. [Firebase Console](https://console.firebase.google.com/)에서 새 프로젝트를 생성합니다.
2. **Firestore Database**를 활성화합니다 (프로덕션 모드 또는 테스트 모드).
3. **Firebase Storage**를 활성화합니다 (이미지 업로드용).
4. **Project Settings**에서 Web App을 추가하고 Firebase SDK 구성을 확인합니다.

## 2. 프론트엔드 설정

`frontend/.env` 파일에 다음과 같은 설정을 추가합니다.

```bash
# Firebase 모드 활성화
VITE_USE_FIREBASE=true

# 관리자 로그인 정보 (Firebase 모드 전용)
VITE_ADMIN_ID=admin
VITE_ADMIN_PW=admin123

# Firebase SDK 설정
VITE_FIREBASE_API_KEY=your_api_key
VITE_FIREBASE_AUTH_DOMAIN=your_project.firebaseapp.com
VITE_FIREBASE_PROJECT_ID=your_project_id
VITE_FIREBASE_STORAGE_BUCKET=your_project.appspot.com
VITE_FIREBASE_MESSAGING_SENDER_ID=your_sender_id
VITE_FIREBASE_APP_ID=your_app_id
```

## 3. SvelteKit Adapter 변경 (권장)

Firebase Hosting에 정적 사이트로 배포하려면 `adapter-static`을 사용하는 것이 좋습니다.

1. `frontend` 디렉토리에서 패키지 설치:
   ```bash
   cd frontend
   bun add -D @sveltejs/adapter-static
   ```
2. `svelte.config.js` 수정:
   ```javascript
   import adapter from '@sveltejs/adapter-static';
   // ...
   ```
3. `src/routes/+layout.ts` 생성 (또는 수정)하여 CSR 활성화:
   ```typescript
   export const prerender = true;
   export const trailingSlash = 'always';
   ```

## 4. 배포하기

1. Firebase CLI 설치 및 로그인:
   ```bash
   npm install -g firebase-tools
   firebase login
   ```
2. 프로젝트 빌드:
   ```bash
   cd frontend
   bun run build
   ```
3. Firebase 배포:
   ```bash
   cd ..
   firebase deploy
   ```

## 주의사항

- **실시간 기능**: Firebase 모드에서는 WebSocket 대신 Firestore의 Snapshot 기능을 사용하여 실시간 채팅 및 진행 상황 공유를 구현합니다.
- **보안 규칙**: `firestore.rules`를 적절히 수정하여 데이터 보안을 강화하세요.
- **제한 사항**: 현재 Firebase 모드에서는 코드랩 내보내기/가져오기 기능이 지원되지 않습니다.
