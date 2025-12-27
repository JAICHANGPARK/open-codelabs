# Firebase 배포

Firebase Hosting을 사용하면 프론트엔드를 Google의 글로벌 CDN에 배포할 수 있습니다.

## 개요

Firebase는 Google의 모바일 및 웹 애플리케이션 플랫폼으로, 다음과 같은 서비스를 제공합니다:

- **Firebase Hosting**: 정적 웹 파일 호스팅
- **Firebase Functions**: 서버리스 백엔드
- **Firestore**: NoSQL 데이터베이스
- **Firebase Authentication**: 사용자 인증

Open Codelabs의 프론트엔드를 Firebase Hosting에 배포할 수 있습니다.

## 사전 준비

- Firebase 계정 ([firebase.google.com](https://firebase.google.com))
- Node.js 18 이상
- Firebase CLI

## 1. Firebase CLI 설치

```bash
npm install -g firebase-tools
```

버전 확인:

```bash
firebase --version
```

## 2. Firebase 로그인

```bash
firebase login
```

브라우저에서 Google 계정으로 로그인합니다.

## 3. Firebase 프로젝트 생성

### 방법 1: Firebase Console에서 생성

1. [Firebase Console](https://console.firebase.google.com) 접속
2. "Add project" 클릭
3. 프로젝트 이름 입력 (예: `open-codelabs`)
4. Google Analytics 설정 (선택사항)

### 방법 2: CLI에서 생성

```bash
firebase projects:create open-codelabs
```

## 4. 프로젝트 설정

프로젝트 루트에서:

```bash
# Firebase 초기화
firebase init hosting
```

설정 옵션:

```
? What do you want to use as your public directory? frontend/build
? Configure as a single-page app (rewrite all urls to /index.html)? Yes
? Set up automatic builds and deploys with GitHub? No
```

생성된 파일:

- `.firebaserc`: 프로젝트 ID 설정
- `firebase.json`: 호스팅 설정

### .firebaserc 수정

```json
{
  "projects": {
    "default": "open-codelabs"
  }
}
```

### firebase.json 확인

```json
{
  "hosting": {
    "public": "frontend/build",
    "ignore": [
      "firebase.json",
      "**/.*",
      "**/node_modules/**"
    ],
    "rewrites": [
      {
        "source": "**",
        "destination": "/index.html"
      }
    ]
  }
}
```

## 5. 프론트엔드 빌드

```bash
cd frontend
npm install
npm run build
```

빌드된 파일은 `frontend/build` 디렉토리에 생성됩니다.

빌드 확인:

```bash
ls -la build/
```

## 6. Firebase 배포

프로젝트 루트에서:

```bash
firebase deploy --only hosting
```

배포 과정:

```
=== Deploying to 'open-codelabs'...

i  deploying hosting
i  hosting[open-codelabs]: beginning deploy...
i  hosting[open-codelabs]: found 25 files in frontend/build
✔  hosting[open-codelabs]: file upload complete
i  hosting[open-codelabs]: finalizing version...
✔  hosting[open-codelabs]: version finalized
i  hosting[open-codelabs]: releasing new version...
✔  hosting[open-codelabs]: release complete

✔  Deploy complete!

Project Console: https://console.firebase.google.com/project/open-codelabs/overview
Hosting URL: https://open-codelabs.web.app
```

## 7. 배포 확인

배포된 URL로 접속:

```bash
open https://open-codelabs.web.app
```

또는:

```bash
open https://open-codelabs.firebaseapp.com
```

## 커스텀 도메인 연결

### 1. 도메인 추가

Firebase Console에서:

1. Hosting 섹션으로 이동
2. "Add custom domain" 클릭
3. 도메인 입력 (예: `codelabs.example.com`)

### 2. DNS 설정

제공된 DNS 레코드를 도메인 관리 페이지에 추가:

```
Type: A
Name: @
Value: 151.101.1.195

Type: A
Name: @
Value: 151.101.65.195
```

또는 CNAME:

```
Type: CNAME
Name: codelabs
Value: open-codelabs.web.app.
```

### 3. 인증 대기

DNS 전파 및 SSL 인증서 발급까지 최대 24시간 소요될 수 있습니다.

## 백엔드 배포 옵션

Firebase Hosting은 정적 파일만 호스팅하므로, 백엔드는 별도로 배포해야 합니다.

### Option 1: Firebase Functions

서버리스 백엔드를 Firebase Functions로 구현:

```bash
# Functions 초기화
firebase init functions

# TypeScript 또는 JavaScript 선택
? What language would you like to use? TypeScript
? Do you want to use ESLint? Yes
? Do you want to install dependencies now? Yes
```

Functions 작성 (`functions/src/index.ts`):

```typescript
import * as functions from 'firebase-functions';

export const api = functions.https.onRequest((request, response) => {
  response.json({ message: "Hello from Firebase!" });
});
```

배포:

```bash
firebase deploy --only functions
```

프론트엔드에서 사용:

```javascript
const response = await fetch('https://us-central1-open-codelabs.cloudfunctions.net/api');
```

### Option 2: Google Cloud Run

Docker 컨테이너로 백엔드 배포:

```bash
# Google Cloud SDK 설치
brew install google-cloud-sdk  # macOS
# 또는 https://cloud.google.com/sdk/docs/install

# 로그인
gcloud auth login

# 프로젝트 설정
gcloud config set project open-codelabs

# 백엔드 이미지 빌드 및 푸시
cd backend
gcloud builds submit --tag gcr.io/open-codelabs/backend

# Cloud Run에 배포
gcloud run deploy backend \
  --image gcr.io/open-codelabs/backend \
  --platform managed \
  --region asia-northeast1 \
  --allow-unauthenticated
```

배포 후 제공되는 URL을 프론트엔드 환경 변수에 설정:

```bash
# frontend/.env
VITE_API_URL=https://backend-xxxxx-an.a.run.app
```

### Option 3: 별도 서버

기존 서버 또는 다른 클라우드 서비스에 백엔드 배포:

```bash
# frontend/.env
VITE_API_URL=https://api.example.com
```

CORS 설정 필요:

```javascript
// backend/src/main.ts
app.enableCors({
  origin: 'https://open-codelabs.web.app',
  credentials: true,
});
```

## 배포 자동화 (CI/CD)

### GitHub Actions

`.github/workflows/deploy.yml` 생성:

```yaml
name: Deploy to Firebase

on:
  push:
    branches: [ main ]
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        run: |
          cd frontend
          npm ci

      - name: Build
        run: |
          cd frontend
          npm run build

      - name: Deploy to Firebase
        uses: FirebaseExtended/action-hosting-deploy@v0
        with:
          repoToken: '${{ secrets.GITHUB_TOKEN }}'
          firebaseServiceAccount: '${{ secrets.FIREBASE_SERVICE_ACCOUNT }}'
          projectId: open-codelabs
          channelId: live
```

### GitHub Secrets 설정

1. Firebase 서비스 계정 키 생성:

```bash
firebase init hosting:github
```

또는 수동으로:

```bash
# 서비스 계정 키 생성
gcloud iam service-accounts keys create firebase-key.json \
  --iam-account firebase-adminsdk@open-codelabs.iam.gserviceaccount.com
```

2. GitHub Repository Settings → Secrets → Actions:
   - `FIREBASE_SERVICE_ACCOUNT`: `firebase-key.json` 내용 붙여넣기

### 자동 배포 테스트

```bash
git add .
git commit -m "Add Firebase deployment"
git push origin main
```

GitHub Actions 탭에서 배포 진행 상황 확인

## 프리뷰 채널

배포 전 테스트를 위한 임시 URL 생성:

### 수동 프리뷰

```bash
firebase hosting:channel:deploy preview
```

출력 예시:

```
✔  hosting:channel: Channel URL (preview): https://open-codelabs--preview-xxxxx.web.app [expires 2024-01-30 12:00:00]
```

### 만료 시간 설정

```bash
# 7일 후 만료
firebase hosting:channel:deploy preview --expires 7d

# 특정 날짜에 만료
firebase hosting:channel:deploy preview --expires 2024-12-31
```

### PR 자동 프리뷰

`.github/workflows/preview.yml`:

```yaml
name: Deploy Preview

on:
  pull_request:
    branches: [ main ]

jobs:
  preview:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install and build
        run: |
          cd frontend
          npm ci
          npm run build

      - name: Deploy to preview channel
        uses: FirebaseExtended/action-hosting-deploy@v0
        with:
          repoToken: '${{ secrets.GITHUB_TOKEN }}'
          firebaseServiceAccount: '${{ secrets.FIREBASE_SERVICE_ACCOUNT }}'
          projectId: open-codelabs
          expires: 7d
```

PR에 자동으로 프리뷰 URL 댓글 추가됨

## 버전 관리 및 롤백

### 배포 이력 확인

```bash
firebase hosting:releases:list
```

출력 예시:

```
Version     Status   Created                      Released
abc123def   Current  2024-01-15 10:30:00 +0900   2024-01-15 10:31:00 +0900
xyz789uvw   Expired  2024-01-14 15:20:00 +0900   2024-01-14 15:21:00 +0900
```

### 이전 버전으로 롤백

```bash
# 특정 버전으로 롤백
firebase hosting:clone SOURCE_SITE_ID:SOURCE_VERSION TARGET_SITE_ID

# 또는 Firebase Console에서:
# Hosting → Release history → 원하는 버전 선택 → "Rollback"
```

### Firebase Console에서 관리

1. [Firebase Console](https://console.firebase.google.com) 접속
2. Hosting → Release history
3. 버전별 비교 및 롤백 가능

## Firebase Hosting 특징

### 장점

- **글로벌 CDN**: 전 세계 150+ 엣지 로케이션
- **무료 SSL**: 자동 인증서 발급 및 갱신
- **자동 최적화**: Gzip/Brotli 압축, HTTP/2
- **쉬운 롤백**: 클릭 한 번으로 이전 버전 복구
- **무료 티어**: 중소 규모 프로젝트에 충분
- **빠른 배포**: 평균 30초 내 전 세계 배포

### 무료 티어 제한

- 저장공간: 10GB
- 전송량: 월 360MB/일 (약 10GB/월)
- 빌드 시간: 제한 없음
- 커스텀 도메인: 무제한
- SSL 인증서: 무제한

### 유료 플랜 (Blaze)

- 종량제: $0.026/GB (전송량)
- 저장공간: $0.026/GB/월
- 무료 티어 포함

### 제한사항

- 정적 파일만 호스팅
- 서버 사이드 로직 불가 (Functions 필요)
- 파일 크기 제한: 2GB
- 빌드 캐시: 비지원

## 비용 최적화

### 전송량 줄이기

1. **이미지 최적화**

```bash
# 빌드 시 자동 최적화
npm install -D vite-plugin-imagemin

# vite.config.js
import viteImagemin from 'vite-plugin-imagemin';

export default {
  plugins: [
    viteImagemin({
      gifsicle: { optimizationLevel: 7 },
      optipng: { optimizationLevel: 7 },
      mozjpeg: { quality: 80 },
      pngquant: { quality: [0.8, 0.9] },
      svgo: { plugins: [{ name: 'removeViewBox' }] },
    }),
  ],
};
```

2. **코드 스플리팅**

```javascript
// 라우트별 lazy loading
const Home = lazy(() => import('./pages/Home'));
const Admin = lazy(() => import('./pages/Admin'));
```

3. **캐싱 설정**

`firebase.json`:

```json
{
  "hosting": {
    "public": "frontend/build",
    "headers": [
      {
        "source": "**/*.@(jpg|jpeg|gif|png|svg|webp)",
        "headers": [
          {
            "key": "Cache-Control",
            "value": "max-age=31536000"
          }
        ]
      },
      {
        "source": "**/*.@(js|css)",
        "headers": [
          {
            "key": "Cache-Control",
            "value": "max-age=31536000"
          }
        ]
      }
    ]
  }
}
```

### 사용량 모니터링

```bash
# Firebase Console에서 확인
open https://console.firebase.google.com/project/open-codelabs/usage

# CLI로 확인
firebase projects:usage
```

### 알림 설정

Firebase Console → Usage and billing → Details & settings:

- 월 전송량 80% 도달 시 이메일 알림
- 예산 한도 설정

## 보안 설정

### 보안 헤더 추가

`firebase.json`:

```json
{
  "hosting": {
    "headers": [
      {
        "source": "**",
        "headers": [
          {
            "key": "X-Content-Type-Options",
            "value": "nosniff"
          },
          {
            "key": "X-Frame-Options",
            "value": "SAMEORIGIN"
          },
          {
            "key": "X-XSS-Protection",
            "value": "1; mode=block"
          },
          {
            "key": "Referrer-Policy",
            "value": "strict-origin-when-cross-origin"
          }
        ]
      }
    ]
  }
}
```

### 환경 변수 관리

민감한 정보는 환경 변수로:

```bash
# GitHub Secrets에 저장
# Firebase에서는 .env 파일 사용
```

빌드 시 주입:

```yaml
# GitHub Actions
- name: Create .env
  run: |
    cd frontend
    echo "VITE_API_URL=${{ secrets.API_URL }}" > .env
    echo "VITE_API_KEY=${{ secrets.API_KEY }}" >> .env
```

## 문제 해결

### 배포 실패

```bash
# 상세 로그 확인
firebase deploy --only hosting --debug

# 캐시 삭제
rm -rf frontend/build frontend/node_modules/.vite
```

### 404 오류

SPA 라우팅 문제일 수 있음. `firebase.json` 확인:

```json
{
  "hosting": {
    "rewrites": [
      {
        "source": "**",
        "destination": "/index.html"
      }
    ]
  }
}
```

### 배포 속도 느림

```bash
# 파일 수 줄이기
npm run build -- --minify

# 또는 .firebaseignore 활용
echo "*.map" >> .firebaseignore
```

### SSL 인증서 오류

커스텀 도메인 DNS 설정 확인:

```bash
# DNS 전파 확인
dig codelabs.example.com

# Firebase 연결 상태 확인
firebase hosting:sites:list
```

## 다음 단계

- [Docker 배포](docker.md) - 전체 스택 배포
- [환경 변수 설정](environment.md) - 설정 관리
- [공개 배포 (ngrok/bore)](public-deployment.md) - 로컬 터널링
- [FAQ](../faq.md) - 자주 묻는 질문
