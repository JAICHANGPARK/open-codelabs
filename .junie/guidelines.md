# Open Codelabs Development Guidelines

This document provides essential information for developing, configuring, and testing the Open Codelabs project.

## 🛠 Build and Configuration

### 1. Backend (Rust)
The backend is built with Rust (Axum framework) and uses SQLite for data storage.

- **Configuration**: Create a `.env` file in the `backend/` directory.
  ```bash
  DATABASE_URL=sqlite:data/sqlite.db?mode=rwc
  ADMIN_ID=admin
  ADMIN_PW=admin123
  RUST_LOG=backend=debug,tower_http=debug
  ```
- **Run**:
  ```bash
  cd backend
  cargo run
  ```

### 2. Frontend (SvelteKit)
The frontend is built with SvelteKit 5 and Bun.

- **Configuration**: Create a `.env` file in the `frontend/` directory.
  ```bash
  VITE_API_URL=http://localhost:8080
  ```
- **Run**:
  ```bash
  cd frontend
  bun install
  bun run dev
  ```

### 3. Docker (Recommended for production/staging)
You can run the entire system using Docker Compose.
```bash
docker-compose up --build
```

---

## 🧪 Testing

### Backend Testing
Backend tests use the standard Rust testing framework.

- **Run all tests**:
  ```bash
  cd backend
  cargo test
  ```
- **Example test (Unit test)**:
  ```rust
  #[cfg(test)]
  mod tests {
      use crate::models::CreateCodelab;

      #[test]
      fn test_create_codelab_serialization() {
          let codelab = CreateCodelab {
              title: "Test Codelab".to_string(),
              description: "Description".to_string(),
              author: "Author".to_string(),
          };
          let json = serde_json::to_string(&codelab).unwrap();
          assert!(json.contains("Test Codelab"));
      }
  }
  ```

### Frontend Testing
Frontend tests use `bun test`.

- **Run all tests**:
  ```bash
  cd frontend
  bun test
  ```
- **Example test (`src/example.test.ts`)**:
  ```typescript
  import { expect, test } from "bun:test";

  test("simple assertion", () => {
    expect(2 + 2).toBe(4);
  });
  ```

### Adding New Tests
- **Backend**: Add `#[cfg(test)] mod tests { ... }` blocks in your Rust files or create a `tests/` directory for integration tests.
- **Frontend**: Create files ending in `.test.ts` or `.spec.ts` anywhere in the `src/` directory.

---

## 📝 Development Standards

### Code Style
Follow the project's established code style guidelines.

#### Backend (Rust)
- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy -- -D warnings`
- **Naming**: `PascalCase` for types, `snake_case` for functions/variables, `SCREAMING_SNAKE_CASE` for constants.

#### Frontend (TypeScript/Svelte)
- **Type Checking**: `bun run check`
- **Formatting**: `bunx prettier --write src`
- **Naming**: `PascalCase` for components, `camelCase` for functions/variables, `SCREAMING_SNAKE_CASE` for constants.

### Architecture
- **Backend**: Uses a handler-based architecture in `backend/src/handlers/`. State is shared via `AppState`.
- **Frontend**: SvelteKit routes are located in `frontend/src/routes/`. Components are in `frontend/src/lib/components/`.

---

## 🚀 Deployment
Use the `run-public.sh` script to expose your local server via `ngrok` or `bore` for public testing or small-scale sessions.
```bash
./run-public.sh --ngrok
```
