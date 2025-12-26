# 개발 워크플로우

Open Codelabs 프로젝트의 개발 워크플로우를 설명합니다.

## Git 워크플로우

### 브랜치 전략

```
main (프로덕션)
  ↓
develop (개발)
  ↓
feature/* (기능)
fix/* (버그 수정)
```

### 브랜치 생성

```bash
# main에서 최신 코드 가져오기
git checkout main
git pull upstream main

# 기능 브랜치 생성
git checkout -b feat/add-feedback-export

# 버그 수정 브랜치
git checkout -b fix/websocket-reconnect
```

## 개발 프로세스

### 1. Issue 생성 또는 선택

GitHub Issues에서:
- 새 Issue 생성 또는
- 기존 Issue 선택
- 자신을 Assignee로 지정

### 2. 로컬 개발

```bash
# Backend 개발
cd backend
cargo watch -x run

# Frontend 개발 (별도 터미널)
cd frontend
bun run dev

# 브라우저에서 테스트
open http://localhost:5173
```

### 3. 코드 작성

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
    // 구현
    Ok(Json(NewFeatureResponse {
        result: "success".to_string(),
    }))
}
```

라우트 추가 (`main.rs`):

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

API 클라이언트 (`lib/api.ts`):

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

### 4. 테스트

#### Backend 테스트

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

실행:

```bash
cargo test
```

#### Frontend 테스트

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

실행:

```bash
bun test
```

### 5. 코드 품질 확인

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

## 커밋 규칙

### 커밋 메시지 형식

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 예시

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

### 커밋 체크리스트

- [ ] 하나의 논리적 변경만 포함
- [ ] 커밋 메시지가 명확함
- [ ] 테스트가 통과함
- [ ] 코드가 포맷팅됨

## Pull Request

### PR 생성

```bash
# 커밋 후
git push origin feat/add-feedback-export

# GitHub에서 "Create Pull Request" 클릭
```

### PR 제목

커밋 메시지와 동일한 형식:

```
feat(backend): add feedback export API
```

### PR 설명

템플릿:

```markdown
## 변경 사항
- Feedback export API 추가
- CSV 형식으로 다운로드 가능

## 동기
관리자가 피드백 데이터를 분석하기 위해 Excel로 가져올 필요가 있음

## 테스트
- [x] Unit 테스트 추가
- [x] 수동 테스트 완료
- [x] 100개 피드백 데이터로 테스트

## 스크린샷
![Export 버튼](screenshot.png)

## 체크리스트
- [x] 코드 포맷팅 완료
- [x] Lint 통과
- [x] 테스트 통과
- [x] 문서 업데이트
- [x] Self-review 완료

Closes #123
```

### PR 리뷰 프로세스

1. **CI 확인**: 자동 빌드 및 테스트 통과
2. **Code Review**: 메인테이너가 리뷰
3. **피드백 반영**: 요청사항 수정
4. **승인**: Approve
5. **머지**: Squash and Merge

### 리뷰 피드백 반영

```bash
# 수정사항 커밋
git add .
git commit -m "fix: address review comments"
git push

# 또는 기존 커밋 수정
git add .
git commit --amend --no-edit
git push --force-with-lease
```

## 릴리스 프로세스

### 버전 관리

[Semantic Versioning](https://semver.org/) 사용:

- `MAJOR.MINOR.PATCH`
- 예: `1.2.3`

### 릴리스 준비

```bash
# 버전 업데이트
# backend/Cargo.toml
version = "1.2.0"

# frontend/package.json
"version": "1.2.0"

# 태그 생성
git tag -a v1.2.0 -m "Release version 1.2.0"
git push origin v1.2.0
```

### GitHub Release

1. GitHub에서 "Releases" 클릭
2. "Draft a new release" 클릭
3. 태그 선택: `v1.2.0`
4. 릴리스 노트 작성:

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
docker-compose pull
docker-compose up -d
\`\`\`
```

## CI/CD (향후)

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

## 팁

### 효율적인 개발

```bash
# Tmux로 멀티 패널
tmux new -s dev
# Ctrl-B % (수직 분할)
# Ctrl-B " (수평 분할)

# 패널 1: Backend
cargo watch -x run

# 패널 2: Frontend
bun run dev

# 패널 3: 터미널
```

### Hot Reload

Backend:

```bash
cargo install cargo-watch
cargo watch -x run
```

Frontend는 기본 지원 (Vite HMR)

### 디버깅

Backend:

```rust
// 로그 추가
tracing::debug!("Processing request: {:?}", request);
tracing::info!("Created codelab: {}", codelab.id);
tracing::warn!("Slow query: {}ms", duration);
tracing::error!("Failed to save: {}", err);
```

Frontend:

```typescript
// Console 로그
console.log('Data:', data);
console.error('Failed:', err);

// Svelte DevTools 사용
```

## 다음 단계

- [기여 가이드](guide.md) - 기여 방법
- [코드 스타일](code-style.md) - 코딩 규칙
- [로컬 개발](../self-hosting/local-development.md) - 개발 환경
