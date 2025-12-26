# 프론트엔드 코드 예제

SvelteKit Frontend의 주요 코드 예제를 소개합니다.

## 컴포넌트 예제

### Codelab 목록

```svelte
<script lang="ts">
    import { onMount } from 'svelte';
    import { listCodelabs, type Codelab } from '$lib/api';

    let codelabs: Codelab[] = [];
    let loading = true;

    onMount(async () => {
        try {
            codelabs = await listCodelabs();
        } catch (err) {
            console.error('Failed to load:', err);
        } finally {
            loading = false;
        }
    });
</script>

{#if loading}
    <p>로딩 중...</p>
{:else}
    {#each codelabs as codelab}
        <div class="card">
            <h2>{codelab.title}</h2>
            <p>{codelab.description}</p>
        </div>
    {/each}
{/if}
```

## API 클라이언트

```typescript
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export async function listCodelabs(): Promise<Codelab[]> {
    const res = await fetch(`${API_URL}/api/codelabs`);
    if (!res.ok) throw new Error('Failed');
    return res.json();
}
```

## 다음 단계

- [API 사용 예제](api-usage.md)
- [아키텍처](../architecture/frontend.md)
