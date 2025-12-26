# 코드 스타일

Open Codelabs 프로젝트의 코딩 스타일 가이드입니다.

## Rust (Backend)

### 포맷팅

```bash
cargo fmt
```

### Lint

```bash
cargo clippy -- -D warnings
```

### 네이밍

- 타입: `PascalCase`
- 함수/변수: `snake_case`
- 상수: `SCREAMING_SNAKE_CASE`

### 예제

```rust
// 좋은 예
struct Codelab { }
fn create_codelab() { }
const MAX_SIZE: usize = 100;

// 나쁜 예
struct codelab { }
fn CreateCodelab() { }
const maxSize: usize = 100;
```

## TypeScript (Frontend)

### 포맷팅

```bash
bunx prettier --write src
```

### 네이밍

- 컴포넌트: `PascalCase`
- 함수/변수: `camelCase`
- 상수: `SCREAMING_SNAKE_CASE`

### 예제

```typescript
// 좋은 예
function fetchCodelabs() { }
const API_URL = "...";

// 나쁜 예
function FetchCodelabs() { }
const apiUrl = "...";
```

## 다음 단계

- [기여 가이드](guide.md)
- [개발 워크플로우](workflow.md)
