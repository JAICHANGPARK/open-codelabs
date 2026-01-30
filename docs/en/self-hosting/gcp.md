# Open Codelabs GCP (Google Cloud Platform) Deployment Guide

This document describes several ways to deploy Open Codelabs on GCP.

## GCP deployment options

Depending on your architecture, there are three primary approaches.

### 1. Firebase (recommended, easiest serverless option)
Firebase is a developer-friendly suite of GCP services. The project already supports Firebase mode, so you can deploy without managing servers.

- **Hosting**: frontend (SvelteKit)
- **Firestore**: codelab data and user info
- **Realtime Database**: chat and progress sharing
- **Storage**: image uploads
- **Auth**: Google login

See the detailed guide: [DEPLOY_FIREBASE.md](../deploy/DEPLOY_FIREBASE.md).

---

### 2. Cloud Run (container-based)
Build the backend (Rust) and frontend as containers and deploy them separately. This is suitable when you want more control over the infrastructure.

#### Architecture
- **Frontend**: SvelteKit (Node.js/Bun) -> Cloud Run service 1
- **Backend**: Rust Axum -> Cloud Run service 2
- **Database**:
  - Simple: SQLite on a Cloud Run local volume.
    Note: Cloud Run is stateless, so the SQLite file resets on restart. For non-temporary usage, use the recommended option below.
  - Recommended: switch to Cloud SQL (PostgreSQL/MySQL) or use **Firebase mode**.

#### Deployment steps

**1. Configure the GCP project and login**
```bash
gcloud auth login
gcloud config set project [YOUR_PROJECT_ID]
```

**2. Create Artifact Registry**
```bash
gcloud artifacts repositories create open-codelabs \
    --repository-format=docker --location=asia-northeast3
```

**3. Build and push images**
```bash
# Backend build
cd backend
gcloud builds submit --tag asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/backend:latest .

# Frontend build
cd ../frontend
gcloud builds submit --tag asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/frontend:latest .
```

**4. Deploy Cloud Run services**
- **Backend**:
    ```bash
    gcloud run deploy backend \
        --image asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/backend:latest \
        --platform managed --region asia-northeast3 --allow-unauthenticated \
        --set-env-vars "DATABASE_URL=sqlite:data/sqlite.db?mode=rwc,ADMIN_ID=admin,ADMIN_PW=admin123"
    ```
- **Frontend**:
    ```bash
    gcloud run deploy frontend \
        --image asia-northeast3-docker.pkg.dev/[PROJECT_ID]/open-codelabs/frontend:latest \
        --platform managed --region asia-northeast3 --allow-unauthenticated \
        --set-env-vars "VITE_API_URL=[BACKEND_SERVICE_URL]"
    ```

---

### 3. Compute Engine (traditional VM)
Deploy with Docker Compose on a GCP VM.

1. Create a GCE instance (Debian or Ubuntu)
2. Install Docker and Docker Compose
3. Clone the repo and run `docker-compose.yml`
    ```bash
    docker-compose up -d --build
    ```

## Which option should you choose?

| Feature | Firebase (Serverless) | Cloud Run (Containers) | Compute Engine (VM) |
| :--- | :--- | :--- | :--- |
| Difficulty | Very low | Medium | Medium |
| Cost | Usage based (generous free tier) | Usage based | Fixed (instance time) |
| Realtime | Firebase RTDB (very stable) | WebSocket (sticky session required) | WebSocket (full control) |
| Maintenance | Almost none | Low | High |

Conclusion: For a first deployment or small workshops, **Firebase** is strongly recommended. If you need custom server behavior or want to go deeper with GCP, consider **Cloud Run**.

## Related links

- [GCP Console](https://console.cloud.google.com/)
- [Firebase Console](https://console.firebase.google.com/)
- [Cloud Run Documentation](https://cloud.google.com/run/docs)
