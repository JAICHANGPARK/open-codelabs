# 공개 배포 (ngrok/bore)

로컬 서버를 외부에 공개하여 워크샵이나 행사에서 사용하는 방법입니다.

## 개요

Open Codelabs는 `run-public.sh` 스크립트를 제공하여 로컬 서버를 쉽게 공개할 수 있습니다:

- **ngrok**: 가장 인기 있는 터널링 서비스
- **bore**: Rust 기반 오픈소스 대안

## ngrok 사용하기

### 1. ngrok 설치

=== "macOS"
    ```bash
    brew install ngrok
    ```

=== "Linux"
    ```bash
    # Snap
    snap install ngrok

    # 또는 직접 다운로드
    wget https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-linux-amd64.tgz
    tar xvzf ngrok-v3-stable-linux-amd64.tgz
    sudo mv ngrok /usr/local/bin
    ```

=== "Windows"
    [ngrok 다운로드 페이지](https://ngrok.com/download)에서 설치

### 2. ngrok 인증 (선택사항)

무료 플랜에서도 사용 가능하지만, 계정 등록 시 더 많은 기능 사용 가능:

```bash
# ngrok.com에서 가입 후 토큰 복사
ngrok config add-authtoken <your_token>
```

### 3. 실행

```bash
chmod +x run-public.sh
./run-public.sh --ngrok
```

출력 예시:

```
🚀 Starting Open-Codelabs: Hands-on System using docker...
✅ Containers are up!
🌐 Starting ngrok tunnel on port 5173...
------------------------------------------------
🎉 Your Codelab is now PUBLIC!
Admin Dashboard: https://abc123.ngrok-free.app/admin
Attendee Entry:  https://abc123.ngrok-free.app
------------------------------------------------
```

### 4. QR 코드 생성

참가자가 쉽게 접속할 수 있도록 QR 코드 생성:

```bash
# qrencode 설치
brew install qrencode  # macOS
apt-get install qrencode  # Linux

# QR 코드 생성
echo "https://abc123.ngrok-free.app" | qrencode -t UTF8

# 또는 이미지로 저장
echo "https://abc123.ngrok-free.app" | qrencode -o qr.png
```

관리자 대시보드에도 자동으로 QR 코드가 표시됩니다!

## bore 사용하기

bore는 오픈소스 대안으로, 자체 서버에서 운영 가능합니다.

### 1. bore 설치

```bash
cargo install bore-cli
```

### 2. 실행

```bash
./run-public.sh --bore
```

bore는 기본적으로 `bore.pub` 서버를 사용합니다.

### 3. 커스텀 bore 서버

자체 bore 서버 운영:

```bash
# 서버 실행
bore server --secret <your_secret>

# 클라이언트 연결
bore local 5173 --to your-server.com --port 80 --secret <your_secret>
```

## run-public.sh 스크립트 상세

### 스크립트 내용

```bash
#!/bin/bash

set -e

# Default values
TUNNEL_TYPE="ngrok"
CONTAINER_ENGINE="docker"

# Check for podman
if command -v podman-compose &> /dev/null; then
    CONTAINER_ENGINE="podman"
elif command -v docker-compose &> /dev/null; then
    CONTAINER_ENGINE="docker"
else
    echo "❌ No container engine found!"
    exit 1
fi

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --bore) TUNNEL_TYPE="bore"; shift ;;
        --ngrok) TUNNEL_TYPE="ngrok"; shift ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
done

echo "🚀 Starting Open-Codelabs Hands-on System using $CONTAINER_ENGINE..."

# Start containers in background
if [ "$CONTAINER_ENGINE" == "podman" ]; then
    podman-compose up -d
else
    docker-compose up -d
fi

echo "✅ Containers are up!"

if [ "$TUNNEL_TYPE" == "ngrok" ]; then
    echo "🌐 Starting ngrok tunnel on port 5173..."
    ngrok http 5173 --log=stdout &
    sleep 5
    PUBLIC_URL=$(curl -s http://localhost:4040/api/tunnels | grep -o 'https://[^"]*.ngrok-free.app' | head -n 1)
else
    echo "🌐 Starting bore tunnel on port 5173..."
    bore local 5173 --to bore.pub &
    sleep 5
    echo "⚠️  Please check the bore output above for your public URL."
    PUBLIC_URL="[Check Bore Output]"
fi

if [ -z "$PUBLIC_URL" ] || [ "$PUBLIC_URL" == "[Check Bore Output]" ]; then
    if [ "$TUNNEL_TYPE" == "ngrok" ]; then
        echo "❌ Failed to get ngrok URL. Is ngrok running?"
    fi
else
    echo "------------------------------------------------"
    echo "🎉 Your Codelab is now PUBLIC!"
    echo "Admin Dashboard: $PUBLIC_URL/admin"
    echo "Attendee Entry:  $PUBLIC_URL"
    echo "------------------------------------------------"
fi

# Keep script running
wait
```

### 커스터마이징

포트 변경:

```bash
# 스크립트 수정
ngrok http 3000  # 5173 대신 3000

# 또는 환경 변수로
PORT=3000 ./run-public.sh --ngrok
```

## 워크샵 시나리오

### 사전 준비 (행사 전날)

1. **로컬 테스트**

```bash
# Docker로 전체 시스템 테스트
docker-compose up

# 브라우저에서 확인
open http://localhost:5173
```

2. **Codelab 생성 및 검증**
   - 모든 Step 작성 완료
   - 이미지 업로드 완료
   - Export하여 백업

3. **네트워크 테스트**

```bash
# ngrok 터널 테스트
./run-public.sh --ngrok

# 다른 디바이스에서 접속 테스트
```

### 행사 당일

#### 1시간 전

```bash
# 시스템 시작
./run-public.sh --ngrok

# URL 확인 및 QR 코드 준비
# 프로젝터로 QR 코드 표시
```

#### 행사 시작

1. **참가자 안내**
   - QR 코드 스캔 또는 URL 접속
   - 이름과 참가 코드 입력

2. **실시간 모니터링**
   - 관리자 대시보드로 진행 상황 확인
   - 도움 요청에 즉시 응답

3. **채팅 활용**
   - 공지사항 전달
   - 질문 답변
   - 1:1 지원

#### 행사 종료

```bash
# 피드백 수집 (자동)
# 데이터 백업
docker cp $(docker-compose ps -q backend):/app/data/sqlite.db ./backup_$(date +%Y%m%d).db

# 시스템 종료
docker-compose down
```

## 보안 고려사항

### ngrok 보안 설정

```bash
# Basic Auth 추가
ngrok http 5173 --basic-auth="user:password"

# IP 화이트리스트 (유료 플랜)
ngrok http 5173 --cidr-allow=192.168.1.0/24
```

### 임시 URL 사용

ngrok의 무료 플랜은 세션마다 새로운 URL 생성:

- 장점: 행사 후 자동으로 접근 불가
- 단점: URL 변경 시 다시 공유 필요

### HTTPS

ngrok은 자동으로 HTTPS 제공:

```
https://abc123.ngrok-free.app  ← 자동 SSL/TLS
```

## 대안 솔루션

### Cloudflare Tunnel

무료이며 더 안정적:

```bash
# cloudflared 설치
brew install cloudflare/cloudflare/cloudflared

# 터널 생성
cloudflared tunnel --url http://localhost:5173
```

### localtunnel

Node.js 기반 대안:

```bash
npm install -g localtunnel
lt --port 5173
```

### Tailscale

VPN 기반 접근 (더 안전):

```bash
# Tailscale 설치 및 설정
brew install tailscale
tailscale up

# 참가자도 Tailscale 설치 필요
```

## 성능 최적화

### ngrok 대역폭

무료 플랜 제한:

- 연결 수: 40/분
- 대역폭: 제한 없음
- 터널 수: 1개

대규모 행사 (100명 이상):

- ngrok Pro 플랜 권장
- 또는 Cloudflare Tunnel 사용

### 네트워크 안정성

```bash
# ngrok 재연결 설정
ngrok http 5173 --log=stdout --log-level=info
```

백그라운드 실행:

```bash
# systemd 서비스 (Linux)
cat > /etc/systemd/system/codelabs-tunnel.service << EOF
[Unit]
Description=Codelabs ngrok Tunnel
After=network.target

[Service]
Type=simple
User=your_user
WorkingDirectory=/path/to/open-codelabs
ExecStart=/usr/local/bin/ngrok http 5173
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable codelabs-tunnel
sudo systemctl start codelabs-tunnel
```

## 모니터링

### ngrok 대시보드

ngrok 실행 중 [http://localhost:4040](http://localhost:4040)에서 확인:

- 실시간 요청/응답
- 대역폭 사용량
- 에러 로그

### 시스템 리소스

```bash
# 실시간 모니터링
docker stats

# 로그 확인
docker-compose logs -f --tail=100
```

## 문제 해결

### ngrok 연결 실패

```bash
# ngrok 프로세스 확인
ps aux | grep ngrok

# 포트 사용 확인
lsof -i :4040
lsof -i :5173

# ngrok 재시작
killall ngrok
./run-public.sh --ngrok
```

### URL을 가져올 수 없음

```bash
# ngrok API 직접 확인
curl http://localhost:4040/api/tunnels | jq .

# 수동으로 URL 확인
open http://localhost:4040
```

### 느린 연결 속도

- 가까운 ngrok 리전 선택:

```bash
ngrok http 5173 --region=jp  # 일본
ngrok http 5173 --region=ap  # 아시아-태평양
```

## Firebase Hosting 배포

Firebase Hosting을 사용하면 프론트엔드를 Google의 글로벌 CDN에 배포할 수 있습니다.

### 1. Firebase CLI 설치

```bash
npm install -g firebase-tools
```

### 2. Firebase 프로젝트 설정

```bash
# Firebase 로그인
firebase login

# 프로젝트 초기화 (이미 설정되어 있다면 생략)
firebase init hosting
```

프로젝트의 `.firebaserc` 파일을 수정하여 Firebase 프로젝트 ID 설정:

```json
{
  "projects": {
    "default": "your-firebase-project-id"
  }
}
```

### 3. 프론트엔드 빌드

```bash
cd frontend
npm install
npm run build
```

빌드된 파일은 `frontend/build` 디렉토리에 생성됩니다.

### 4. Firebase 배포

```bash
# 프로젝트 루트에서 실행
firebase deploy --only hosting
```

배포 후 제공되는 URL 예시:

```
✔  Deploy complete!

Project Console: https://console.firebase.google.com/project/your-project/overview
Hosting URL: https://your-project.web.app
```

### 5. 커스텀 도메인 (선택사항)

Firebase Console에서 커스텀 도메인 연결:

1. [Firebase Console](https://console.firebase.google.com) 접속
2. Hosting 섹션으로 이동
3. "Add custom domain" 클릭
4. 도메인 입력 및 DNS 설정

### Firebase Hosting 특징

**장점:**

- 글로벌 CDN을 통한 빠른 속도
- 무료 SSL 인증서 자동 제공
- 무료 티어로도 충분한 용량 (10GB 저장공간, 월 10GB 전송량)
- 쉬운 롤백 및 버전 관리
- 자동 캐싱 및 압축

**제한사항:**

- 정적 파일만 호스팅 가능 (백엔드는 별도 배포 필요)
- 백엔드는 Firebase Functions, Cloud Run, 또는 별도 서버 필요

### 백엔드 배포 옵션

프론트엔드를 Firebase Hosting에 배포한 경우 백엔드 배포 옵션:

#### Option 1: Firebase Functions

```bash
# Functions 초기화
firebase init functions

# 배포
firebase deploy --only functions
```

#### Option 2: Google Cloud Run

```bash
# 백엔드 Docker 이미지 빌드
cd backend
docker build -t gcr.io/your-project/backend .

# Cloud Run에 배포
gcloud run deploy backend \
  --image gcr.io/your-project/backend \
  --platform managed \
  --region asia-northeast1
```

#### Option 3: 별도 서버

프론트엔드는 Firebase Hosting, 백엔드는 기존 서버 사용:

```bash
# .env 파일에 백엔드 URL 설정
VITE_API_URL=https://your-backend-server.com
```

### 배포 자동화

GitHub Actions를 사용한 자동 배포:

```yaml
# .github/workflows/deploy.yml
name: Deploy to Firebase

on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '18'

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
          projectId: your-firebase-project-id
```

### 프리뷰 채널

배포 전 테스트를 위한 프리뷰 URL 생성:

```bash
# 프리뷰 채널 생성
firebase hosting:channel:deploy preview

# 특정 기간 동안 유효한 프리뷰
firebase hosting:channel:deploy preview --expires 7d
```

### 비용 관리

Firebase Hosting 무료 티어:

- 저장공간: 10GB
- 전송량: 월 10GB (약 1만 사용자)
- 빌드 시간: 제한 없음

대규모 행사의 경우:

```bash
# 사용량 모니터링
firebase hosting:metrics
```

## 다음 단계

- [환경 변수 설정](environment.md) - 세부 설정
- [Docker 배포](docker.md) - 프로덕션 배포
- [FAQ](../faq.md) - 자주 묻는 질문
