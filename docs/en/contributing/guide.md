# Contribution Guide

Thank you for contributing to the Open Codelabs project! This guide explains how to contribute to the project.

## How to Contribute

### 🐛 Bug Reports

Found a bug?

1. Check for duplicates in [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues).
2. If none exist, create a new Issue.
3. Include the following information:
   - **Environment**: OS, Docker version, browser, etc.
   - **Reproduction Steps**: Step-by-step description.
   - **Expected Behavior**: What should happen.
   - **Actual Behavior**: What actually happens.
   - **Screenshots**: Attach if possible.

**Issue Template**:

```markdown
## Environment
- OS: macOS 14.0
- Docker: 24.0.6
- Browser: Chrome 120

## Reproduction Steps
1. Login as Admin
2. Create Codelab
3. Attempt to add a Step

## Expected Behavior
Step should be successfully added

## Actual Behavior
500 error occurs

## Screenshots
[Attached]

## Logs
```
```
ERROR backend::handlers::codelabs: Failed to create step
```
```

### ✨ Feature Suggestions

Have an idea for a new feature?

1. Discuss in [GitHub Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions).
2. Collect community feedback.
3. Once approved, convert to an Issue.
4. Begin implementation.

**Suggestion Template**:

```markdown
## Background
Why is this feature needed?

## Proposal
What feature would you like to add?

## Use Case
How will it be used?

## Alternatives
Are there any other ways?

## Intent to Implement
Will you be implementing this yourself? (Yes/No)
```

### 📝 Documentation Improvements

Are there any typos or areas for improvement in the documentation?

1. Modify the `docs/` directory.
2. Submit a Pull Request.
3. Fast merge!

### 💻 Code Contributions

Want to write code?

1. Select (or create) an issue.
2. Fork & Clone.
3. Create a branch.
4. Write code.
5. Test your changes.
6. Submit a Pull Request.

For more details, please refer to the [Development Workflow](workflow.md).

## Getting Started

### 1. Fork & Clone

```bash
# Click the Fork button on GitHub

# Clone
git clone https://github.com/YOUR_USERNAME/open-codelabs.git
cd open-codelabs

# Add Upstream
git remote add upstream https://github.com/JAICHANGPARK/open-codelabs.git
```

### 2. Development Environment Setup

```bash
# Backend
cd backend
cp .env.example .env
cargo check

# Frontend
cd ../frontend
bun install
```

For more details, please refer to the [Local Development Environment](../self-hosting/local-development.md).

### 3. Creating a Branch

```bash
# Get the latest main
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feat/your-feature-name

# Or bug fix
git checkout -b fix/issue-number-description
```

**Branch Naming Conventions**:

- `feat/`: New feature
- `fix/`: Bug fix
- `docs/`: Documentation change
- `refactor/`: Code refactoring
- `test/`: Adding tests
- `chore/`: Miscellaneous (build, CI, etc.)

## Coding Guidelines

### Backend (Rust)

#### Code Style

```bash
# Formatting
cargo fmt

# Lint
cargo clippy

# Tests
cargo test
```

#### Naming Conventions

```rust
// Types: PascalCase
struct Codelab { ... }
enum Status { ... }

// Functions/Variables: snake_case
fn create_codelab() { ... }
let user_name = "Alice";

// Constants: SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: u32 = 100;
```

#### Error Handling

```rust
// ✅ Good example
async fn get_codelab(id: &str) -> Result<Codelab, StatusCode> {
    let codelab = sqlx::query_as::<_, Codelab>(
        "SELECT * FROM codelabs WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(codelab)
}

// ❌ Bad example
async fn get_codelab(id: &str) -> Codelab {
    sqlx::query_as::<_, Codelab>(
        "SELECT * FROM codelabs WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap()  // Potential panic!
}
```

#### Logging

```rust
use tracing::{info, warn, error, debug};

// Use appropriate levels
debug!("Parsing request body");
info!("Codelab created: {}", codelab.id);
warn!("Database connection slow");
error!("Failed to create codelab: {}", err);
```

### Frontend (TypeScript/Svelte)

#### Code Style

```bash
# Type check
bun run check

# Formatting (Prettier)
bunx prettier --write src
```

#### Naming Conventions

```typescript
// Components: PascalCase
export default CodelabList;

// Functions/Variables: camelCase
const codelabList = [];
function fetchCodelabs() { ... }

// Constants: SCREAMING_SNAKE_CASE
const API_BASE_URL = "...";

// Types/Interfaces: PascalCase
interface Codelab { ... }
type Status = 'pending' | 'resolved';
```

#### Svelte Component Structure

```svelte
<script lang="ts">
    // 1. Imports
    import { onMount } from 'svelte';
    import { listCodelabs } from '$lib/api';

    // 2. Types
    interface Codelab {
        id: string;
        title: string;
    }

    // 3. Props
    export let title: string;

    // 4. State
    let codelabs: Codelab[] = [];
    let loading = false;

    // 5. Functions
    async function loadCodelabs() {
        loading = true;
        try {
            codelabs = await listCodelabs();
        } catch (err) {
            console.error('Failed to load codelabs:', err);
        } finally {
            loading = false;
        }
    }

    // 6. Lifecycle
    onMount(() => {
        loadCodelabs();
    });
</script>

<!-- 7. Template -->
<div class="container">
    <h1>{title}</h1>

    {#if loading}
        <p>Loading...</p>
    {:else}
        {#each codelabs as codelab}
            <div>{codelab.title}</div>
        {/each}
    {/if}
</div>

<!-- 8. Styles (scoped) -->
<style>
    .container {
        padding: 1rem;
    }
</style>
```

#### API Calls

```typescript
// ✅ Good example
export async function listCodelabs(): Promise<Codelab[]> {
    const res = await fetch(`${API_URL}/codelabs`);

    if (!res.ok) {
        throw new Error(`Failed to fetch codelabs: ${res.status}`);
    }

    return res.json();
}

// ❌ Bad example
export async function listCodelabs() {
    const res = await fetch(`${API_URL}/codelabs`);
    return res.json();  // No error handling, no type specification
}
```

## Testing

### Backend Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_codelab() {
        let pool = setup_test_db().await;

        let payload = CreateCodelab {
            title: "Test".to_string(),
            description: "Test Desc".to_string(),
            author: "Tester".to_string(),
        };

        let result = create_codelab_handler(payload, &pool).await;
        assert!(result.is_ok());
    }
}
```

### Frontend Testing

```typescript
import { describe, it, expect } from 'bun:test';
import { render } from '@testing-library/svelte';
import CodelabList from './CodelabList.svelte';

describe('CodelabList', () => {
    it('renders title', () => {
        const { getByText } = render(CodelabList, {
            props: { title: 'My Codelabs' }
        });

        expect(getByText('My Codelabs')).toBeTruthy();
    });
});
```

## Commit Messages

### Guidelines

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation change
- `style`: Code formatting (no functional changes)
- `refactor`: Refactoring
- `test`: Adding/modifying tests
- `chore`: Build, CI, etc.

### Examples

```bash
# Good commit message
feat(backend): add feedback API endpoints

Implement POST and GET endpoints for codelab feedback:
- POST /api/codelabs/:id/feedback
- GET /api/codelabs/:id/feedback

Closes #42

# Simple bug fix
fix(frontend): correct typo in login form

# Documentation change
docs: update installation guide for M1 Mac
```

### Commit Size

- One commit should contain only one logical change.
- Split large commits into multiple ones.
- Squash WIP commits before PR.

```bash
# Good example
git commit -m "feat(backend): add feedback model"
git commit -m "feat(backend): add feedback API handlers"
git commit -m "feat(frontend): add feedback form component"

# Bad example
git commit -m "add feedback feature"  # Too large
git commit -m "fix typo"              # Too small (should be squashed into previous commit)
```

## Pull Requests

### Pre-PR Checklist

- [ ] Is the code formatted? (`cargo fmt`, `prettier`)
- [ ] Does Lint pass? (`cargo clippy`)
- [ ] Do tests pass? (`cargo test`, `bun test`)
- [ ] Is the documentation updated?
- [ ] Does the commit message follow the rules?

### PR Template

```markdown
## Changes
What did you change?

## Motivation
Why is this change necessary?

## Testing
How did you test it?

## Screenshots (for UI changes)
[Attached]

## Checklist
- [ ] Code formatting complete
- [ ] Lint passed
- [ ] Tests passed
- [ ] Documentation updated
- [ ] Self-review complete

Closes #issue_number
```

### PR Review

Maintainers will review for:

- Code quality
- Test coverage
- Documentation completeness
- Commit history

Responding to feedback:

```bash
# After modification
git add .
git commit -m "fix: address review comments"
git push

# Or modify existing commit
git commit --amend
git push --force-with-lease
```

## Code of Conduct

### What we strive for

- **Respect**: Respect all contributors.
- **Inclusivity**: Welcome diversity.
- **Collaboration**: Solve problems together.
- **Learning**: Mistakes are opportunities for learning.

### Prohibitions

- Harassment, discrimination, hate speech.
- Making private information public.
- Trolling, intentional disruption.
- Inappropriate content.

Upon violation:
1. Warning
2. Suspension
3. Permanent ban

## License

Contributed code follows the project's [Apache License 2.0](../license.md).

## Questions?

- [GitHub Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions)
- [Discord/Slack](Link)
- [Email](mailto:team@example.com)

## Thank You!

Check the list of all contributors:

- [Contributors](https://github.com/JAICHANGPARK/open-codelabs/graphs/contributors)

We look forward to your contributions! 🚀
