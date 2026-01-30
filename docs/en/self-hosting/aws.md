# Open Codelabs AWS (Amazon Web Services) Deployment Guide

This document describes several ways to deploy Open Codelabs on AWS infrastructure.

## AWS deployment options

Depending on your architecture, there are three primary approaches.

### 1. AWS App Runner (recommended, easiest container option)
Similar to Google Cloud Run, App Runner is a fully managed service that deploys directly from source or container images. It is the easiest way to run both frontend and backend without managing infrastructure.

#### Architecture
- **Frontend**: SvelteKit -> App Runner service 1
- **Backend**: Rust Axum -> App Runner service 2
- **Database**:
  - **Simple**: SQLite inside the App Runner instance (data resets on restart)
  - **Recommended**: AWS RDS (PostgreSQL/MySQL)

#### Deployment steps
1. **Create ECR repositories**
    ```bash
    aws ecr create-repository --repository-name open-codelabs/backend
    aws ecr create-repository --repository-name open-codelabs/frontend
    ```
2. **Build and push images**
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
3. **Create App Runner services**
    - In the AWS Console, open **App Runner** and click "Create service".
    - Select the images from ECR and configure environment variables.
      - Backend: `DATABASE_URL`, `ADMIN_ID`, `ADMIN_PW`
      - Frontend: `VITE_API_URL` (backend App Runner URL)

---

### 2. AWS EC2 (traditional)
Deploy with Docker Compose on a virtual machine. This can be the cheapest option (especially with Free Tier).

1. **Create an EC2 instance** (Amazon Linux 2023 or Ubuntu recommended)
2. **Configure security groups**: open ports 80, 443 (HTTPS), 5173 (frontend), 8080 (backend)
3. **Install Docker and Docker Compose**
4. **Clone and run**
    ```bash
    git clone https://github.com/JAICHANGPARK/open-codelabs.git
    cd open-codelabs
    docker-compose up -d --build
    ```

---

### 3. AWS Amplify (frontend only)
Use this if you are running Firebase mode (`VITE_USE_FIREBASE=true`) or only want to host the frontend on AWS.

1. Connect the GitHub repo in the **Amplify Console**
2. Configure the build settings (SvelteKit is auto-detected)
3. Add env vars such as `VITE_USE_FIREBASE` and `VITE_FIREBASE_API_KEY`

---

## Database and storage (production)

For production, use persistent services for data durability.

### 1. Database (RDS)
Set the backend `DATABASE_URL` to your RDS instance.
- **PostgreSQL**: `postgres://user:password@host:port/dbname`
- **MySQL**: `mysql://user:password@host:port/dbname`

### 2. Storage (S3)
You can use S3 for uploads.
- The project currently supports local filesystem or Firebase Storage by default.
- For S3, you will need to implement an additional SDK integration. A local volume mount or Firebase mode is recommended for container deployments.

## Which option should you choose?

| Feature | App Runner | EC2 (Docker) | Amplify (Frontend) |
| :--- | :--- | :--- | :--- |
| Difficulty | Low | Medium | Very low |
| Cost | Usage based | Fixed (instance) | Usage based |
| Ops overhead | Very low | Medium | Low |
| Best for | Quick prototyping, auto scaling | Lowest cost, full control | Firebase mode |

Conclusion: If you are comfortable with containers, choose **App Runner**. If cost is the priority, **EC2** is a good fit.

## Related links

- [AWS Management Console](https://aws.amazon.com/console/)
- [AWS App Runner Documentation](https://docs.aws.amazon.com/apprunner/)
- [AWS EC2 Documentation](https://docs.aws.amazon.com/ec2/)
