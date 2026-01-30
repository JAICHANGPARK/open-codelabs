# Quickstart

Use this guide to get Open Codelabs running in about 5 minutes.

## Prerequisites

Minimum requirements:

- [Docker](https://www.docker.com/get-started) (recommended), or
- [Bun](https://bun.sh/) + [Rust](https://www.rust-lang.org/) for local development

## Run with Docker (recommended)

This is the simplest option. You only need Docker installed.

### 1. Clone the repository

```bash
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs
```

### 2. Start with Docker Compose

```bash
docker compose up --build
```

The first run may take a few minutes while images build.

### 3. Open in your browser

Once the build is done:

- **Facilitator (admin)**: [http://localhost:5173/login](http://localhost:5173/login)
  - ID: `admin`
  - PW: `admin`
- **Attendee**: [http://localhost:5173](http://localhost:5173)

## Run locally for development

If you are developing, you can run everything locally.

### Run the backend

```bash
cd backend

# Set environment variables
cat > .env << EOF
DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123
EOF

# Create database directory
mkdir -p data

# Start the server
cargo run
```

The backend runs at `http://localhost:8080`.

### Run the frontend

In a new terminal:

```bash
cd frontend

# Install dependencies
bun install

# Start the dev server
bun run dev
```

The frontend runs at `http://localhost:5173`.

## Create your first codelab

### 1. Log in as admin

1. Go to [http://localhost:5173/login](http://localhost:5173/login)
2. Log in with the default credentials:
   - ID: `admin`
   - PW: `admin123`

### 2. Create a codelab

1. Click "Create New Codelab"
2. Fill in details:
   - **Title**: "My First Codelab"
   - **Description**: "Build a web server with Rust"
   - **Author**: "Jane Doe"
3. Click "Create"

### 3. Add steps

Open the newly created codelab card to edit:

1. Click "Add Step"
2. Enter step details:
   - **Title**: "Project Setup"
   - **Content**: Write in Markdown

   ```markdown
   # Project Setup

   Create a new Rust project:

   ```bash
   cargo new my-web-server
   cd my-web-server
   ```

   ## Add dependencies

   Add these dependencies to `Cargo.toml`:

   ```toml
   [dependencies]
   axum = "0.7"
   tokio = { version = "1.0", features = ["full"] }
   ```
   ```

3. Click "Save"

### 4. Test as an attendee

1. Open a new incognito window (or another browser) and visit [http://localhost:5173](http://localhost:5173)
2. Select the codelab
3. Enter your name and attendee code
4. Follow the steps

## Next steps

You have created your first codelab. Explore the following:

- [Installation Guide](installation.md) - detailed setup options
- [Create Your First Codelab](first-codelab.md) - advanced features
- [Public Deployment](../self-hosting/public-deployment.md) - expose locally via ngrok/bore/cloudflare
- [API Reference](../specification/api-reference.md) - automation and integrations

## Troubleshooting

### Docker containers do not start

```bash
# Stop existing containers
docker compose down

# Clean volumes
docker compose down -v

# Restart
docker compose up --build
```

### Port is already in use

Update the ports in `docker-compose.yml`:

```yaml
services:
  frontend:
    ports:
      - "3000:5173"  # Use 3000 instead of 5173
  backend:
    ports:
      - "3080:8080"  # Use 3080 instead of 8080
```

### Database errors

```bash
# Reset backend data
rm -rf backend/data/sqlite.db

# Restart
docker compose restart backend
```

For more troubleshooting, see the [FAQ](../faq.md).
