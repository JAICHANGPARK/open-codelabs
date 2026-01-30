# FAQ (Frequently Asked Questions)

This is a list of frequently asked questions and answers for using Open Codelabs.

## Installation and Execution

### Q: Can I run it without Docker?

**A:** Yes, it is possible. You can install Rust and Bun and run them locally.

```bash
# Backend
cd backend
cargo run

# Frontend
cd frontend
bun install && bun run dev
```

For more details, please refer to the [Local Development Environment](self-hosting/local-development.md).

### Q: Docker build is slow on M1/M2 Mac.

**A:** It can be slow on Apple Silicon due to emulation through Rosetta 2.

Solution:
1. In Docker Desktop settings, enable "Use Rosetta for x86/amd64 emulation".
2. Use native build:

```yaml
# docker-compose.yml
services:
  backend:
    platform: linux/arm64  # Add this
```

### Q: Ports 8080 and 5173 are already in use.

**A:** Change the ports in `docker-compose.yml`:

```yaml
services:
  backend:
    ports:
      - "3080:8080"  # host:container

  frontend:
    ports:
      - "3000:5173"
```

Alternatively, terminate the existing processes:

```bash
# Check port usage
lsof -i :8080
lsof -i :5173

# Terminate process
kill -9 <PID>
```

### Q: It won't run on Windows.

**A:** We recommend using WSL2:

1. Install WSL2.
2. Install Ubuntu.
3. Install Docker Desktop for Windows and enable WSL2 integration.
4. Clone and run the project within WSL2 Ubuntu.

## Usage

### Q: I forgot the administrator password.

**A:** Change the environment variable and restart:

```bash
# Docker
docker compose down
# Change ADMIN_PW in docker-compose.yml
docker compose up

# Local
# Change ADMIN_PW in backend/.env
cargo run
```

### Q: What if participant names are duplicated?

**A:** Names cannot be duplicated within the same Codelab. Please instruct participants to use unique names:

- "User_1", "User_2"
- Using full names or initials
- Using email: "hong@example.com"

### Q: Can I delete a step?

**A:** Currently, there is no direct step deletion feature. If you rewrite and save the step list, the existing steps will be replaced.

Alternatively, delete directly from the database:

```bash
sqlite3 backend/data/sqlite.db
sqlite> DELETE FROM steps WHERE id = 'step_id';
```

### Q: Images are not showing in Markdown.

**A:** Please check the image path:

1. Upload the image on the admin page.
2. Copy the automatically generated URL.
3. Insert into Markdown:

```markdown
![description](http://localhost:8080/assets/images/xxx.png)
```

External image URLs can also be used:

```markdown
![description](https://example.com/image.png)
```

### Q: Chat messages disappear.

**A:** If the WebSocket connection is lost, real-time messages may be lost. However, all messages are saved in the database.

Refreshing the page will load the existing chat history.

## Public Deployment

### Q: Is the ngrok free plan sufficient?

**A:** It is sufficient for small workshops (~40 people).

ngrok free plan:
- Connections: 40/min
- Bandwidth: Unlimited
- Tunnels: 1

If there are more participants:
- Use ngrok Pro plan.
- Use Cloudflare Tunnel.
- Deploy on your own server.

### Q: The ngrok URL changes every time.

**A:** In the free plan, a new URL is generated for each session.

If you need a fixed URL:
1. ngrok paid plan (Reserved Domain)
2. Cloudflare Tunnel
3. Deploy on your own domain

### Q: Participants see an "ngrok warning page".

**A:** The ngrok free plan displays a warning page upon the first access.

Clicking the "Visit Site" button will grant access.

To remove the warning page:
- Use a paid ngrok plan.
- Use other tunneling services (e.g., Cloudflare Tunnel).

## Performance and Scaling

### Q: How many users are supported?

**A:** Test results:

- **Concurrent users**: 100 people (Stable)
- **Maximum tested**: 200 people (Incremental CPU usage)

Limiting factors:
- SQLite concurrent writes
- Single-server WebSocket

For more users:
- Migrate to PostgreSQL.
- Use Redis as a WebSocket message broker.
- Add a load balancer.

### Q: Memory usage is high.

**A:** Check items:

```bash
# Check container resources
docker stats

# Check log size
docker compose logs backend | wc -l
```

Optimization:
1. Lower log level: `RUST_LOG=info`
2. Set resource limits:

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          memory: 512M
```

### Q: The database file keeps growing.

**A:** Run VACUUM regularly:

```bash
sqlite3 backend/data/sqlite.db "VACUUM;"
```

Or delete old data:

```sql
-- Delete Codelabs older than 30 days
DELETE FROM codelabs WHERE created_at < datetime('now', '-30 days');
```

## Troubleshooting

### Q: "Database is locked" error occurs.

**A:** SQLite does not support concurrent writes.

Solution:
1. Check `?mode=rwc`: `DATABASE_URL=sqlite:data/sqlite.db?mode=rwc`
2. Reduce connection pool size:

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(1)  // Default is 5
    .connect(&database_url)
    .await?;
```

3. Consider migrating to PostgreSQL.

### Q: WebSocket connection keeps dropping.

**A:** Causes:
- Firewall/Proxy
- Network instability
- Server restart

Debugging:

```javascript
// Check reconnection logic in Frontend
const ws = new WebSocket(wsUrl);

ws.onclose = (event) => {
    console.log('WebSocket closed:', event.code, event.reason);

    // Reconnect
    setTimeout(() => reconnect(), 3000);
};
```

### Q: Image upload fails.

**A:** Check items:

1. File size: Maximum 10MB (default)
2. File format: PNG, JPG, GIF, WebP
3. Directory permissions:

```bash
# Docker
docker compose exec backend ls -la /app/static/assets/images

# If there is a permission issue
docker compose exec backend chmod 755 /app/static/assets
```

### Q: API call fails in Frontend.

**A:** This might be a CORS issue.

Check Backend logs:

```bash
docker compose logs backend | grep CORS
```

Verify CORS settings in `main.rs`:

```rust
.layer(tower_http::cors::CorsLayer::permissive())
```

## Others

### Q: Is multi-language supported?

**A:** Currently, we officially support Korean and English. The i18n structure is ready for more languages.

How to contribute:
1. Add language files to `frontend/src/lib/i18n/`.
2. Submit a Pull Request.

### Q: Can I use it on mobile?

**A:** Yes, it supports mobile and tablets with a responsive design.

Tested environments:
- iOS Safari
- Android Chrome
- iPad

### Q: Can I export a Codelab to PDF?

**A:** It is not directly supported yet, but:

1. Download ZIP via Export.
2. Convert Markdown to PDF using Pandoc:

```bash
pandoc step_1.md -o step_1.pdf
```

Or use the browser's print function (Ctrl/Cmd+P).

### Q: I want to contribute to the project.

**A:** Welcome! Please refer to the [Contribution Guide](contributing/guide.md).

Simple ways to contribute:
- Bug reports: [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues)
- Feature suggestions: [Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions)
- Pull Request: [Development Workflow](contributing/workflow.md)

### Q: Can I use it commercially?

**A:** Yes, you can freely use, modify, and distribute it under the MIT license.

However, you must include the following:
- Original copyright notice
- MIT license text

For more details, please refer to the [License](license.md).

## Additional Help

Couldn't find an answer above?

- [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues) - Bug Reports
- [GitHub Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions) - Questions & Discussions
- [Documentation](index.md) - Explore full documentation
