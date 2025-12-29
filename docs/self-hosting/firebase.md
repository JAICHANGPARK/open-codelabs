# Open Codelabs Firebase 배포 가이드

이 문서는 서버(Rust/SQLite)를 직접 운영하기 어려운 사용자를 위해 Firebase(Hosting + Firestore + Realtime Database)를 사용하여 Open Codelabs를 배포하는 방법을 설명합니다.

## 1. Firebase 프로젝트 준비

1. [Firebase Console](https://console.firebase.google.com/)에서 새 프로젝트를 생성합니다.
2. **Firestore Database**를 활성화합니다 (프로덕션 모드 또는 테스트 모드).
3. **Realtime Database**를 활성화합니다 (실시간 채팅 및 진행 상황 공유용).
4. **Firebase Storage**를 활성화합니다 (이미지 업로드용).
5. **Firebase Authentication**에서 **Google 로그인**을 활성화합니다.
6. **Project Settings**에서 Web App을 추가하고 Firebase SDK 구성을 확인합니다.

## 2. 프론트엔드 설정

`frontend/.env` 파일에 다음과 같은 설정을 추가합니다.

```bash
# Firebase 모드 활성화
VITE_USE_FIREBASE=true

# 관리자 로그인 정보 (Firebase 모드 전용 - 백업용)
VITE_ADMIN_ID=admin
VITE_ADMIN_PW=admin123

# Firebase SDK 설정
VITE_FIREBASE_API_KEY=your_api_key
VITE_FIREBASE_AUTH_DOMAIN=your_project.firebaseapp.com
VITE_FIREBASE_PROJECT_ID=your_project_id
VITE_FIREBASE_STORAGE_BUCKET=your_project.appspot.com
VITE_FIREBASE_MESSAGING_SENDER_ID=your_sender_id
VITE_FIREBASE_APP_ID=your_app_id
VITE_FIREBASE_DATABASE_URL=https://your_project.firebaseio.com
```

## 3. SvelteKit Adapter 변경

Firebase Hosting에 정적 사이트로 배포하려면 `adapter-static`을 사용합니다.

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
3. `src/routes/+layout.ts`에서 CSR 및 프리렌더링 설정 (이미 프로젝트에 설정되어 있을 수 있습니다).

## 4. 보안 규칙 및 인덱스 설정

프로젝트 루트에 포함된 다음 파일들이 배포 시 함께 적용됩니다:
- `firestore.rules`: Firestore 접근 권한 설정
- `database.rules.json`: Realtime Database 보안 규칙 (채팅 등)
- `storage.rules`: Storage 이미지 업로드 규칙
- `firestore.indexes.json`: 효율적인 쿼리를 위한 인덱스 설정

## 5. 배포하기

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

## 💡 주요 특징 및 주의사항

- **구글 로그인**: Firebase 인증의 구글 로그인을 지원하며, 로그인한 사용자는 자신의 코드랩을 관리하거나 참여한 목록을 확인할 수 있습니다.
- **실시간 기능**: Realtime Database를 사용하여 실시간 채팅, 도움 요청 및 참석자 진행 상황을 공유합니다.
- **데이터 저장**: 코드랩 메타데이터 및 단계 정보는 Firestore에 저장됩니다.
- **제한 사항**: 현재 Firebase 모드에서는 코드랩 내보내기/가져오기(ZIP) 기능이 지원되지 않습니다.

## 🔗 관련 링크
- [Firebase Console](https://console.firebase.google.com/)
- [Firebase Hosting 문서](https://firebase.google.com/docs/hosting)
