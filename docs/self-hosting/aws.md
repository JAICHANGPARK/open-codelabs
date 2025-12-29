# Open Codelabs AWS(Amazon Web Services) 배포 가이드

이 문서는 Open Codelabs 프로젝트를 AWS 인프라를 활용하여 배포하는 다양한 방법을 설명합니다.

## 🚀 AWS 배포 옵션

Open Codelabs는 아키텍처에 따라 크게 세 가지 방식으로 AWS에 배포할 수 있습니다.

### 1. AWS App Runner (추천 - 가장 쉬운 컨테이너 방식)
Google Cloud Run과 유사하게, 소스 코드나 Docker 이미지를 직접 배포할 수 있는 풀 관리형 서비스입니다. 인프라 관리 없이 백엔드와 프런트엔드를 실행하기에 가장 적합합니다.

#### 🏗 아키텍처
*   **Frontend**: SvelteKit -> App Runner 서비스 1
*   **Backend**: Rust Axum -> App Runner 서비스 2
*   **Database**:
    *   **간이 배포**: App Runner 인스턴스 내 SQLite 사용 (재시작 시 데이터 초기화 주의)
    *   **권장**: **AWS RDS (PostgreSQL/MySQL)** 사용

#### 🛠 배포 단계
1.  **ECR (Elastic Container Registry) 생성**
    ```bash
    aws ecr create-repository --repository-name open-codelabs/backend
    aws ecr create-repository --repository-name open-codelabs/frontend
    ```
2.  **이미지 빌드 및 푸시**
    ```bash
    # Backend
    cd backend
    docker build -t [ACCOUNT_ID].dkr.ecr.[REGION].amazonaws.com/open-codelabs/backend:latest .
    docker push [ACCOUNT_ID].dkr.ecr.[REGION].amazonaws.com/open-codelabs/backend:latest

    # Frontend
    cd ../frontend
    docker build -t [ACCOUNT_ID].dkr.ecr.[REGION].amazonaws.com/open-codelabs/frontend:latest .
    docker push [ACCOUNT_ID].dkr.ecr.[REGION].amazonaws.com/open-codelabs/frontend:latest
    ```
3.  **App Runner 서비스 생성**
    *   AWS Console에서 **App Runner**로 이동하여 '서비스 생성'을 클릭합니다.
    *   ECR에 푸시한 이미지를 선택하고, 환경 변수를 설정합니다.
        *   Backend: `DATABASE_URL`, `ADMIN_ID`, `ADMIN_PW`
        *   Frontend: `VITE_API_URL` (백엔드 App Runner URL)

---

### 2. AWS EC2 (전통적인 방식)
가상 머신(VM)에 직접 Docker Compose를 사용하여 배포하는 방식입니다. 가장 저렴하게(Free Tier 활용 시) 운영할 수 있는 방법입니다.

1.  **EC2 인스턴스 생성** (Amazon Linux 2023 또는 Ubuntu 추천)
2.  **보안 그룹 설정**: 80, 443(HTTPS), 5173(Frontend), 8080(Backend) 포트 개방
3.  **Docker & Docker Compose 설치**
4.  **저장소 클론 및 실행**
    ```bash
    git clone https://github.com/JAICHANGPARK/open-codelabs.git
    cd open-codelabs
    docker-compose up -d --build
    ```

---

### 3. AWS Amplify (프런트엔드 전용)
Firebase 모드(`VITE_USE_FIREBASE=true`)를 사용하거나, 프런트엔드만 AWS에서 호스팅하고 싶을 때 적합합니다.

1.  **Amplify Console** 접속 및 GitHub 저장소 연결
2.  **빌드 설정**: SvelteKit 설정을 자동으로 감지합니다.
3.  **환경 변수 설정**: `VITE_USE_FIREBASE`, `VITE_FIREBASE_API_KEY` 등 필요한 설정을 추가합니다.

---

## 💾 데이터베이스 및 스토리지 설정 (Production)

운영 환경에서는 데이터 영속성을 위해 다음 서비스를 연동하는 것을 권장합니다.

### 1. Database (RDS)
백엔드 `DATABASE_URL` 환경 변수에 RDS 주소를 입력합니다.
*   **PostgreSQL**: `postgres://user:password@host:port/dbname`
*   **MySQL**: `mysql://user:password@host:port/dbname`

### 2. Storage (S3)
이미지 업로드를 위해 AWS S3를 사용할 수 있습니다.
*   현재 프로젝트는 로컬 파일 시스템 또는 Firebase Storage를 기본으로 지원합니다.
*   AWS S3 연동이 필요한 경우 별도의 SDK 구현이 필요합니다. (기본적으로는 컨테이너 모드에서 로컬 볼륨 마운트나 Firebase 모드를 권장합니다.)

## 💡 어떤 방식을 선택해야 하나요?

| 특징 | App Runner | EC2 (Docker) | Amplify (Frontend) |
| :--- | :--- | :--- | :--- |
| **난이도** | 낮음 | 보통 | 매우 낮음 |
| **비용** | 사용량 기반 | 고정 비용 (인스턴스) | 사용량 기반 |
| **관리 부담** | 매우 낮음 | 보통 | 낮음 |
| **적합한 사례** | 빠른 프로토타이핑, 자동 스케일링 필요 시 | 가장 저렴한 운영, 전체 제어 필요 시 | Firebase 모드 사용 시 |

**결론**: 컨테이너 기술에 익숙하다면 **AWS App Runner**를, 비용 최적화가 중요하다면 **EC2**를 추천합니다.

## 🔗 관련 링크
*   [AWS Management Console](https://aws.amazon.com/console/)
*   [AWS App Runner Documentation](https://docs.aws.amazon.com/apprunner/)
*   [AWS EC2 Documentation](https://docs.aws.amazon.com/ec2/)
