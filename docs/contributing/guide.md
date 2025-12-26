# 기여 가이드

Open Codelabs 프로젝트에 기여해주셔서 감사합니다! 이 가이드는 프로젝트에 기여하는 방법을 설명합니다.

## 기여 방법

### 🐛 버그 리포트

버그를 발견하셨나요?

1. [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues)에서 중복 확인
2. 없다면 새 Issue 생성
3. 다음 정보 포함:
   - **환경**: OS, Docker 버전, 브라우저 등
   - **재현 방법**: 단계별 설명
   - **예상 동작**: 어떻게 되어야 하는지
   - **실제 동작**: 실제로 어떻게 되는지
   - **스크린샷**: 가능하면 첨부

**Issue 템플릿**:

```markdown
## 환경
- OS: macOS 14.0
- Docker: 24.0.6
- 브라우저: Chrome 120

## 재현 방법
1. 관리자 로그인
2. Codelab 생성
3. Step 추가 시도

## 예상 동작
Step이 성공적으로 추가되어야 함

## 실제 동작
500 에러 발생

## 스크린샷
[첨부]

## 로그
```
ERROR backend::handlers::codelabs: Failed to create step
```
```

### ✨ 기능 제안

새로운 기능 아이디어가 있으신가요?

1. [GitHub Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions)에서 논의
2. 커뮤니티 피드백 수집
3. 승인되면 Issue로 전환
4. 구현 시작

**제안 템플릿**:

```markdown
## 제안 배경
왜 이 기능이 필요한가요?

## 제안 내용
어떤 기능을 추가하고 싶나요?

## 사용 예시
어떻게 사용될까요?

## 대안
다른 방법은 없나요?

## 구현 의향
직접 구현하실 건가요? (Yes/No)
```

### 📝 문서 개선

문서에 오타나 개선 사항이 있나요?

1. `docs/` 디렉토리 수정
2. Pull Request 제출
3. 빠른 머지!

### 💻 코드 기여

코드를 작성하고 싶으신가요?

1. 이슈 선택 (또는 생성)
2. Fork & Clone
3. 브랜치 생성
4. 코드 작성
5. 테스트
6. Pull Request

자세한 내용은 [개발 워크플로우](workflow.md)를 참조하세요.

## 시작하기

### 1. Fork & Clone

```bash
# GitHub에서 Fork 버튼 클릭

# Clone
git clone https://github.com/YOUR_USERNAME/open-codelabs.git
cd open-codelabs

# Upstream 추가
git remote add upstream https://github.com/JAICHANGPARK/open-codelabs.git
```

### 2. 개발 환경 설정

```bash
# Backend
cd backend
cp .env.example .env
cargo check

# Frontend
cd ../frontend
bun install
```

자세한 내용은 [로컬 개발 환경](../self-hosting/local-development.md)을 참조하세요.

### 3. 브랜치 생성

```bash
# 최신 main 가져오기
git checkout main
git pull upstream main

# 기능 브랜치 생성
git checkout -b feat/your-feature-name

# 또는 버그 수정
git checkout -b fix/issue-number-description
```

**브랜치 네이밍 규칙**:

- `feat/`: 새 기능
- `fix/`: 버그 수정
- `docs/`: 문서 수정
- `refactor/`: 코드 리팩토링
- `test/`: 테스트 추가
- `chore/`: 기타 (빌드, CI 등)

## 코딩 가이드라인

### Backend (Rust)

#### 코드 스타일

```bash
# 포맷팅
cargo fmt

# Lint
cargo clippy

# 테스트
cargo test
```

#### 네이밍 규칙

```rust
// 타입: PascalCase
struct Codelab { ... }
enum Status { ... }

// 함수/변수: snake_case
fn create_codelab() { ... }
let user_name = "Alice";

// 상수: SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: u32 = 100;
```

#### 에러 처리

```rust
// ✅ 좋은 예
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

// ❌ 나쁜 예
async fn get_codelab(id: &str) -> Codelab {
    sqlx::query_as::<_, Codelab>(
        "SELECT * FROM codelabs WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap()  // panic 가능!
}
```

#### 로깅

```rust
use tracing::{info, warn, error, debug};

// 적절한 레벨 사용
debug!("Parsing request body");
info!("Codelab created: {}", codelab.id);
warn!("Database connection slow");
error!("Failed to create codelab: {}", err);
```

### Frontend (TypeScript/Svelte)

#### 코드 스타일

```bash
# 타입 체크
bun run check

# 포맷팅 (Prettier)
bunx prettier --write src
```

#### 네이밍 규칙

```typescript
// 컴포넌트: PascalCase
export default CodelamList;

// 함수/변수: camelCase
const codelabList = [];
function fetchCodelabs() { ... }

// 상수: SCREAMING_SNAKE_CASE
const API_BASE_URL = "...";

// 타입/인터페이스: PascalCase
interface Codelab { ... }
type Status = 'pending' | 'resolved';
```

#### Svelte 컴포넌트 구조

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

#### API 호출

```typescript
// ✅ 좋은 예
export async function listCodelabs(): Promise<Codelab[]> {
    const res = await fetch(`${API_URL}/codelabs`);

    if (!res.ok) {
        throw new Error(`Failed to fetch codelabs: ${res.status}`);
    }

    return res.json();
}

// ❌ 나쁜 예
export async function listCodelabs() {
    const res = await fetch(`${API_URL}/codelabs`);
    return res.json();  // 에러 처리 없음, 타입 명시 없음
}
```

## 테스트

### Backend 테스트

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

### Frontend 테스트

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

## 커밋 메시지

### 규칙

[Conventional Commits](https://www.conventionalcommits.org/) 사용:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**타입**:
- `feat`: 새 기능
- `fix`: 버그 수정
- `docs`: 문서 변경
- `style`: 코드 포맷팅 (기능 변경 없음)
- `refactor`: 리팩토링
- `test`: 테스트 추가/수정
- `chore`: 빌드, CI 등

### 예시

```bash
# 좋은 커밋 메시지
feat(backend): add feedback API endpoints

Implement POST and GET endpoints for codelab feedback:
- POST /api/codelabs/:id/feedback
- GET /api/codelabs/:id/feedback

Closes #42

# 간단한 버그 수정
fix(frontend): correct typo in login form

# 문서 수정
docs: update installation guide for M1 Mac
```

### 커밋 크기

- 하나의 커밋에는 하나의 논리적 변경만
- 너무 큰 커밋은 여러 개로 분리
- WIP 커밋은 PR 전에 squash

```bash
# 좋은 예
git commit -m "feat(backend): add feedback model"
git commit -m "feat(backend): add feedback API handlers"
git commit -m "feat(frontend): add feedback form component"

# 나쁜 예
git commit -m "add feedback feature"  # 너무 큼
git commit -m "fix typo"              # 너무 작음 (이전 커밋에 squash)
```

## Pull Request

### PR 생성 전 체크리스트

- [ ] 코드가 포맷팅되었나? (`cargo fmt`, `prettier`)
- [ ] Lint가 통과하나? (`cargo clippy`)
- [ ] 테스트가 통과하나? (`cargo test`, `bun test`)
- [ ] 문서가 업데이트되었나?
- [ ] 커밋 메시지가 규칙을 따르나?

### PR 템플릿

```markdown
## 변경 사항
무엇을 변경했나요?

## 동기
왜 이 변경이 필요한가요?

## 테스트
어떻게 테스트했나요?

## 스크린샷 (UI 변경 시)
[첨부]

## 체크리스트
- [ ] 코드 포맷팅 완료
- [ ] Lint 통과
- [ ] 테스트 통과
- [ ] 문서 업데이트
- [ ] Self-review 완료

Closes #issue_number
```

### PR 리뷰

메인테이너가 리뷰합니다:

- 코드 품질
- 테스트 커버리지
- 문서 완성도
- 커밋 히스토리

피드백에 대응:

```bash
# 수정 후
git add .
git commit -m "fix: address review comments"
git push

# 또는 기존 커밋 수정
git commit --amend
git push --force-with-lease
```

## 행동 강령

### 우리가 지향하는 것

- **존중**: 모든 기여자 존중
- **포용**: 다양성 환영
- **협력**: 함께 문제 해결
- **학습**: 실수는 학습 기회

### 금지 사항

- 괴롭힘, 차별, 혐오 발언
- 개인 정보 공개
- 트롤링, 의도적 방해
- 부적절한 콘텐츠

위반 시:
1. 경고
2. 일시 정지
3. 영구 차단

## 라이선스

기여한 코드는 프로젝트의 [MIT License](../license.md)를 따릅니다.

## 질문이 있으신가요?

- [GitHub Discussions](https://github.com/JAICHANGPARK/open-codelabs/discussions)
- [Discord/Slack](링크)
- [이메일](mailto:team@example.com)

## 감사합니다!

모든 기여자 목록:

- [Contributors](https://github.com/JAICHANGPARK/open-codelabs/graphs/contributors)

당신의 기여를 기다립니다! 🚀
