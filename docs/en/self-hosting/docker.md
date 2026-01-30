# Deploy with Docker

A complete guide to deploying Open Codelabs with Docker.

## Basic deployment

### docker-compose.yml structure

The project's `docker-compose.yml` file:

```yaml
services:
  backend:
    build: ./backend
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
      - ADMIN_ID=admin
      - ADMIN_PW=admin123
    volumes:
      - ./backend/data:/app/data

  frontend:
    build: ./frontend
    ports:
      - "5173:5173"
    environment:
      - VITE_API_URL=http://backend:8080
      - PORT=5173
      - HOST=0.0.0.0
    depends_on:
      - backend
```

### Basic run

```bash
# Build and run
docker compose up --build

# Run in the background
docker compose up -d

# View logs
docker compose logs -f

# Stop
docker compose down
```

!!! note
    Depending on your environment, you may need to use `docker-compose` instead of `docker compose`.

## Use GitHub Container Registry images

You can use GHCR images instead of building locally. Use `docker-compose.images.yml`.

### 1. Prepare environment variables

Create `.env` in the project root and set the values below.

```bash
IMAGE_REGISTRY=ghcr.io
IMAGE_NAMESPACE=open-codelabs
IMAGE_TAG=latest

DATA_VOLUME_PATH=./backend
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=admin123

FRONTEND_PORT=5173
FRONTEND_HOST=0.0.0.0
VITE_API_URL=http://localhost:8080
```

### 2. Run

```bash
docker compose -f docker-compose.images.yml up -d
```

!!! tip
    Set `IMAGE_TAG` to a release tag instead of `latest` to pin the version.

## Backend Dockerfile

`backend/Dockerfile`:

```dockerfile
# Multi-stage build for optimal image size
FROM rust:1.75 AS builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install required libraries
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /app/target/release/backend /app/backend

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Create data directory
RUN mkdir -p /app/data

# Expose port
EXPOSE 8080

# Run the application
CMD ["./backend"]
```

### Key points

- **Multi-stage build**: minimize final image size
- **Release mode**: optimized binary
- **Runtime-only dependencies**: no Rust compiler needed
- **Migrations included**: auto DB initialization

## Frontend Dockerfile

`frontend/Dockerfile`:

```dockerfile
FROM oven/bun:1 AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
RUN bun run build

# Runtime stage
FROM oven/bun:1-slim

WORKDIR /app

# Copy built files
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./

# Install production dependencies only
RUN bun install --production

EXPOSE 5173

# Run with bun
CMD ["bun", "run", "build/index.js"]
```

## Environment variables

### Use .env files

Reference env files from `docker-compose.yml`:

```yaml
services:
  backend:
    env_file:
      - backend/.env
    build: ./backend
    # ...
```

`backend/.env`:

```bash
DATABASE_URL=sqlite:/app/data/sqlite.db?mode=rwc
ADMIN_ID=admin
ADMIN_PW=SecurePassword123!
RUST_LOG=backend=info,tower_http=info
```

### Security recommendations

!!! danger "Production security"
    - Never use the default password (`admin123`)
    - Do not commit `.env` files to Git
    - Use a strong password (20+ characters recommended)

```bash
# Generate a strong password
openssl rand -base64 32
```

## Data persistence

### Volume setup

Use volumes to persist database data.

#### SQLite
```yaml
services:
  backend:
    volumes:
      - ./backend/data:/app/data          # host directory
      - backend_data:/app/data            # Docker volume (recommended)
    # ...

volumes:
  backend_data:
```

#### PostgreSQL (example)
```yaml
services:
  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=codelab
      - POSTGRES_PASSWORD=secure_password
      - POSTGRES_DB=open_codelabs
    volumes:
      - postgres_data:/var/lib/postgresql/data

  backend:
    environment:
      - DATABASE_URL=postgres://codelab:secure_password@db:5432/open_codelabs
    depends_on:
      - db
    # ...

volumes:
  postgres_data:
```

### Backup strategy

#### Database backup

```bash
# Backup the SQLite database
docker-compose exec backend sqlite3 /app/data/sqlite.db ".backup /app/data/backup.db"

# Copy to host
docker cp <container_id>:/app/data/backup.db ./backup.db
```

#### Automated backup script

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Database backup
docker-compose exec -T backend sqlite3 /app/data/sqlite.db ".backup /app/data/backup_$TIMESTAMP.db"
docker cp $(docker-compose ps -q backend):/app/data/backup_$TIMESTAMP.db $BACKUP_DIR/

echo "Backup created: $BACKUP_DIR/backup_$TIMESTAMP.db"

# Delete backups older than 30 days
find $BACKUP_DIR -name "backup_*.db" -mtime +30 -delete
```

```bash
# Make executable
chmod +x backup.sh

# Schedule daily with cron
0 2 * * * /path/to/backup.sh
```

## Reverse proxy setup

### Nginx proxy

Add nginx to `docker-compose.yml`:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - frontend
      - backend

  backend:
    # Replace ports with expose (do not expose publicly)
    expose:
      - "8080"

  frontend:
    expose:
      - "5173"
```

`nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    upstream frontend {
        server frontend:5173;
    }

    upstream backend {
        server backend:8080;
    }

    server {
        listen 80;
        server_name your-domain.com;

        # Frontend
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Backend API
        location /api {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # WebSocket support
        location /api/ws {
            proxy_pass http://backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
        }
    }
}
```

### HTTPS/SSL setup

Using Let's Encrypt:

```yaml
services:
  certbot:
    image: certbot/certbot
    volumes:
      - ./certs:/etc/letsencrypt
      - ./certbot-data:/var/www/certbot
    command: certonly --webroot --webroot-path=/var/www/certbot --email your@email.com --agree-tos --no-eff-email -d your-domain.com
```

HTTPS nginx configuration:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /etc/nginx/certs/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/nginx/certs/live/your-domain.com/privkey.pem;

    # SSL config
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # ... other settings
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

## Production optimizations

### Resource limits

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

### Health checks

```yaml
services:
  backend:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/codelabs"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  frontend:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5173"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Auto-restart

```yaml
services:
  backend:
    restart: unless-stopped

  frontend:
    restart: unless-stopped
```

## Docker Compose command reference

### Basic commands

```bash
# Build only
docker-compose build

# Build without cache
docker-compose build --no-cache

# Run in the background
docker-compose up -d

# Run a single service
docker-compose up backend

# Scaling (multiple instances)
docker-compose up --scale backend=3

# Stop
docker-compose stop

# Restart
docker-compose restart

# Full cleanup (including volumes)
docker-compose down -v
```

### Logs and monitoring

```bash
# All logs
docker-compose logs

# Follow logs
docker-compose logs -f

# Specific service logs
docker-compose logs -f backend

# Last 100 lines
docker-compose logs --tail=100

# Include timestamps
docker-compose logs -t
```

### Debugging

```bash
# Enter a container
docker-compose exec backend sh

# Run a command
docker-compose exec backend ls /app/data

# Copy files (container -> host)
docker cp <container_id>:/app/data/sqlite.db ./

# Copy files (host -> container)
docker cp ./config.toml <container_id>:/app/
```

### Cleanup

```bash
# Remove stopped containers
docker-compose rm

# Remove unused images
docker image prune

# Full system cleanup
docker system prune -a
```

## Troubleshooting

### Migration error (checksum mismatch)

Recent updates modified migrations for PostgreSQL/MySQL support. Existing SQLite users may see `Error: migration ... was previously applied but has been modified`. Try one of the following:

1. **Reset the database (recommended)**: if you do not need the existing data, delete the SQLite file and restart.
   ```bash
   rm backend/data/sqlite.db
   docker-compose up --build
   ```
2. **Switch to PostgreSQL/MySQL**: new databases avoid this problem. See [Environment Variables](environment.md) and set `DATABASE_URL`.

### Containers keep restarting

```bash
# Check logs
docker-compose logs backend

# Run manually to see errors
docker-compose run backend sh
```

### Port conflicts

```bash
# Check port usage
sudo lsof -i :8080

# Change port in docker-compose.yml
ports:
  - "8081:8080"  # host:container
```

### Low disk space

```bash
# Check Docker disk usage
docker system df

# Cleanup
docker system prune -a --volumes
```

### Network issues

```bash
# Recreate network
docker-compose down
docker network prune
docker-compose up
```

## Next steps

- [Local development](local-development.md) - set up a dev environment
- [Public deployment](public-deployment.md) - expose locally via ngrok/bore/cloudflare
- [Environment variables](environment.md) - detailed configuration

## References

- [Docker Compose documentation](https://docs.docker.com/compose/)
- [Docker security best practices](https://docs.docker.com/develop/security-best-practices/)
- [Multi-stage builds](https://docs.docker.com/develop/develop-images/multistage-build/)
