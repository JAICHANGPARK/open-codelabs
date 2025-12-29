# Open Codelabs GCP(Google Cloud Platform) 배포 가이드

이 문서는 Open Codelabs 프로젝트를 GCP 인프라를 활용하여 배포하는 다양한 방법을 설명합니다.

## 🚀 GCP 배포 옵션

Open Codelabs는 아키텍처에 따라 크게 두 가지 방식으로 GCP에 배포할 수 있습니다.

### 1. Firebase (추천 - 가장 쉬운 서버리스 방식)
Firebase는 GCP의 개발자 친화적인 하위 서비스군입니다. 프로젝트의 코드가 이미 Firebase 모드를 지원하도록 설계되어 있어, 별도의 서버 관리 없이 가장 빠르게 배포할 수 있습니다.

*   **Hosting**: 프런트엔드 (SvelteKit) 배포
*   **Firestore**: 코드랩 데이터 및 사용자 정보 저장
*   **Realtime Database**: 실시간 채팅 및 진행 상황 공유
*   **Storage**: 이미지 업로드
*   **Auth**: 구글 로그인 연동

👉 상세 가이드: [DEPLOY_FIREBASE.md](DEPLOY_FIREBASE.md) 를 참고하세요.

---

### 2. Cloud Run (컨테이너 기반 방식)
백엔드(Rust)와 프런트엔드를 각각 컨테이너로 빌드하여 배포하는 방식입니다. 인프라를 더 세밀하게 제어하고 싶을 때 적합합니다.

#### 🏗 아키텍처
*   **Frontend**: SvelteKit (Node.js/Bun) -> Cloud Run 서비스 1
*   **Backend**: Rust Axum -> Cloud Run 서비스 2
*   **Database**:
    *   간이 배포: Cloud Run의 로컬 볼륨에 SQLite 저장 (재시작 시 데이터 초기화 위험 있음)
    *   권장: Cloud SQL (PostgreSQL/MySQL)로 전환 또는 Cloud Firestore 사용

#### 🛠 배포 단계

**1. GCP 프로젝트 설정 및 CLI 로그인**
```bash
gcloud auth login
gcloud config set project [YOUR_PROJECT_ID]
```

**2. Artifact Registry 생성**
```bash
gcloud artifacts repositories create open-codelabs \
    --repository-format=docker --location=asia-northeast3
```

**3. 이미지 빌드 및 푸시**
```bash
# Backend 빌드
cd backend
gcloud builds submit --tag asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/backend:latest .

# Frontend 빌드
cd ../frontend
gcloud builds submit --tag asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/frontend:latest .
```

**4. Cloud Run 서비스 배포**
*   **Backend 배포**:
    ```bash
    gcloud run deploy backend \
        --image asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/backend:latest \
        --platform managed --region asia-northeast3 --allow-unauthenticated \
        --set-env-vars "DATABASE_URL=sqlite:data/sqlite.db?mode=rwc,ADMIN_ID=admin,ADMIN_PW=admin123"
    ```
*   **Frontend 배포**:
    ```bash
    gcloud run deploy frontend \
        --image asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/frontend:latest \
        --platform managed --region asia-northeast3 --allow-unauthenticated \
        --set-env-vars "VITE_API_URL=[BACKEND_SERVICE_URL]"
    ```

---

### 3. Compute Engine (전통적인 방식)
GCP의 가상 머신(VM)에 직접 Docker Compose를 사용하여 배포하는 방식입니다.

1.  **GCE 인스턴스 생성** (Debian 또는 Ubuntu)
2.  **Docker & Docker Compose 설치**
3.  **저장소 클론 및 `docker-compose.yml` 실행**
    ```bash
    docker-compose up -d --build
    ```

## 💡 어떤 방식을 선택해야 하나요?

| 특징 | Firebase (Serverless) | Cloud Run (Containers) | Compute Engine (VM) |
| :--- | :--- | :--- | :--- |
| **난이도** | 매우 낮음 | 보통 | 보통 |
| **비용** | 사용량 기반 (무료 티어 넉넉함) | 사용량 기반 | 고정 비용 (인스턴스 시간당) |
| **실시간 기능** | Firebase RTDB (매우 안정적) | WebSocket (Sticky Session 고려 필요) | WebSocket (직접 제어 가능) |
| **유지보수** | 거의 없음 | 낮음 | 높음 |

**결론**: 처음 시작하거나 소규모 세션을 운영한다면 **Firebase** 방식을 강력히 추천합니다. GCP를 깊게 공부하거나 커스텀 서버 기능이 필요하다면 **Cloud Run**을 고려해 보세요.

## 🔗 관련 링크
*   [GCP Console](https://console.cloud.google.com/)
*   [Firebase Console](https://console.firebase.google.com/)
*   [Cloud Run Documentation](https://cloud.google.com/run/docs)
