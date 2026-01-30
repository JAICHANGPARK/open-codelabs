# Development Workflow

This document describes the development workflow for the Open Codelabs project.

## Git Workflow

### Branch Strategy

```
main (Production)
  ↓
develop (Development)
  ↓
feature/* (Features)
fix/* (Bug Fixes)
```

### Creating branches

```bash
# Get the latest code from main
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feat/add-feedback-export

# Bug fix branch
git checkout -b fix/websocket-reconnect
```

## Development Process

### 1. Create or Select an Issue

On GitHub Issues:
- Create a new Issue or
- Select an existing Issue
- Set yourself as the Assignee

### 2. Local Development

```bash
# Backend development
cd backend
cargo watch -x run

# Frontend development (in a separate terminal)
cd frontend
bun run dev

# Test in browser
open http://localhost:5173
```

### 3. Writing Code

#### Backend (Rust)

```rust
// src/handlers/new_feature.rs
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct NewFeatureRequest {
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct NewFeatureResponse {
    pub result: String,
}

pub async fn new_feature_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewFeatureRequest>,
) -> Result<Json<NewFeatureResponse>, StatusCode> {
    // Implementation
    Ok(Json(NewFeatureResponse {
        result: "success".to_string(),
    }))
}
```

Add routes (`main.rs`):

```rust
.route("/api/new-feature", post(new_feature_handler))
```

#### Frontend (Svelte)

```svelte
<!-- src/routes/new-feature/+page.svelte -->
<script lang="ts">
    import { onMount } from 'svelte';
    import { newFeatureApi } from '$lib/api';

    let data = '';
    let result = '';

    async function handleSubmit() {
        try {
            result = await newFeatureApi(data);
        } catch (err) {
            console.error('Failed:', err);
        }
    }
</script>

<form on:submit|preventDefault={handleSubmit}>
    <input bind:value={data} placeholder="Enter data" />
    <button type="submit">Submit</button>
</form>

{#if result}
    <p>Result: {result}</p>
{/if}
```

API Client (`lib/api.ts`):

```typescript
export async function newFeatureApi(data: string): Promise<string> {
    const res = await fetch(`${API_URL}/new-feature`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ data })
    });

    if (!res.ok) {
        throw new Error('API call failed');
    }

    const json = await res.json();
    return json.result;
}
```

### 4. Testing

#### Backend Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_feature() {
        let request = NewFeatureRequest {
            data: "test".to_string(),
        };

        let result = new_feature_handler(request).await;
        assert!(result.is_ok());
    }
}
```

Execution:

```bash
cargo test
```

#### Frontend Testing

```typescript
// src/routes/new-feature/+page.test.ts
import { describe, it, expect } from 'bun:test';
import { render } from '@testing-library/svelte';
import NewFeature from './+page.svelte';

describe('NewFeature', () => {
    it('renders form', () => {
        const { getByPlaceholderText } = render(NewFeature);
        expect(getByPlaceholderText('Enter data')).toBeTruthy();
    });
});
```

Execution:

```bash
bun test
```

### 5. Code Quality Check

```bash
# Backend
cd backend
cargo fmt --check
cargo clippy
cargo test

# Frontend
cd frontend
bun run check
bunx prettier --check src
```

## Commit Rules

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Examples

```bash
feat(backend): add feedback export API

Implement GET /api/codelabs/:id/feedback/export endpoint
that exports feedback data as CSV

Closes #123

---

fix(frontend): fix WebSocket reconnection

WebSocket now properly reconnects on connection loss
with exponential backoff

Fixes #124

---

docs: update installation guide

Add M1 Mac specific instructions

---

refactor(backend): simplify help request handler

Extract common logic into helper function
No functional changes
```

### Commit Checklist

- [ ] Contains only one logical change
- [ ] Clear commit message
- [ ] Tests passed
- [ ] Code formatted

## Pull Requests

### Creating a PR

```bash
# After committing
git push origin feat/add-feedback-export

# Click "Create Pull Request" on GitHub
```

### PR Title

Same format as commit message:

```
feat(backend): add feedback export API
```

### PR Description

Template:

```markdown
## Changes
- Added feedback export API
- Download available in CSV format

## Motivation
Administrators need to export feedback data to Excel for analysis.

## Testing
- [x] Unit tests added
- [x] Manual testing complete
- [x] Tested with 100 feedback data items

## Screenshots
![Export Button](screenshot.png)

## Checklist
- [x] Code formatting complete
- [x] Lint passed
- [x] Tests passed
- [x] Documentation updated
- [x] Self-review complete

Closes #123
```

### PR Review Process

1. **CI Check**: Automated build and tests pass
2. **Code Review**: Maintainer reviews
3. **Feedback Reflecting**: Request issues fixed
4. **Approval**: Approved
5. **Merge**: Squash and Merge

### Reflecting Review Feedback

```bash
# Commit fixes
git add .
git commit -m "fix: address review comments"
git push

# Or modify existing commit
git add .
git commit --amend --no-edit
git push --force-with-lease
```

## Release Process

### Version Management

Use [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH`
- e.g., `1.2.3`

### Preparing a Release

```bash
# Update versions
# backend/Cargo.toml
version = "1.2.0"

# frontend/package.json
"version": "1.2.0"

# Create tag
git tag -a v1.2.0 -m "Release version 1.2.0"
git push origin v1.2.0
```

### GitHub Release

1. Click "Releases" on GitHub.
2. Click "Draft a new release".
3. Select tag: `v1.2.0`.
4. Write release notes:

```markdown
## What's New in v1.2.0

### Features
- Add feedback export API (#123)
- Add dark mode support (#125)

### Bug Fixes
- Fix WebSocket reconnection (#124)
- Fix image upload on mobile (#126)

### Improvements
- Improve performance of Step rendering
- Update dependencies

## Breaking Changes
None

## Upgrade Guide
Just pull the latest image:
\`\`\`bash
docker compose pull
docker compose up -d
\`\`\`
```

## CI/CD (Future)

### GitHub Actions

`.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Check
        run: cargo check
      - name: Test
        run: cargo test
      - name: Clippy
        run: cargo clippy -- -D warnings

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: oven-sh/setup-bun@v1
      - name: Install
        run: bun install
      - name: Check
        run: bun run check
      - name: Test
        run: bun test
```

## Tips

### Efficient Development

```bash
# Multi-panel with Tmux
tmux new -s dev
# Ctrl-B % (vertical split)
# Ctrl-B " (horizontal split)

# Panel 1: Backend
cargo watch -x run

# Panel 2: Frontend
bun run dev

# Panel 3: Terminal
```

### Hot Reload

Backend:

```bash
cargo install cargo-watch
cargo watch -x run
```

Frontend handles this by default (Vite HMR).

### Debugging

Backend:

```rust
// Add logs
tracing::debug!("Processing request: {:?}", request);
tracing::info!("Created codelab: {}", codelab.id);
tracing::warn!("Slow query: {}ms", duration);
tracing::error!("Failed to save: {}", err);
```

Frontend:

```typescript
// Console logs
console.log('Data:', data);
console.error('Failed:', err);

// Use Svelte DevTools
```

## Next Steps

- [Contribution Guide](guide.md) - How to contribute
- [Code Style](code-style.md) - Coding rules
- [Local Development](../self-hosting/local-development.md) - Development environment
