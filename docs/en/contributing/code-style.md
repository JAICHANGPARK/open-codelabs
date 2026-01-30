# Code Style

This is the coding style guide for the Open Codelabs project.

## Rust (Backend)

### Formatting

```bash
cargo fmt
```

### Lint

```bash
cargo clippy -- -D warnings
```

### Naming

- Types: `PascalCase`
- Functions/Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

### Example

```rust
// Good example
struct Codelab { }
fn create_codelab() { }
const MAX_SIZE: usize = 100;

// Bad example
struct codelab { }
fn CreateCodelab() { }
const maxSize: usize = 100;
```

## TypeScript (Frontend)

### Formatting

```bash
bunx prettier --write src
```

### Naming

- Components: `PascalCase`
- Functions/Variables: `camelCase`
- Constants: `SCREAMING_SNAKE_CASE`

### Example

```typescript
// Good example
function fetchCodelabs() { }
const API_URL = "...";

// Bad example
function FetchCodelabs() { }
const apiUrl = "...";
```

## Next Steps

- [Contribution Guide](guide.md)
- [Development Workflow](workflow.md)
