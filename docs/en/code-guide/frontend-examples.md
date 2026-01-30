# Frontend Code Examples

This document introduces key code examples for the SvelteKit Frontend.

## Component Example

### Codelab List

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
    <p>Loading...</p>
{:else}
    {#each codelabs as codelab}
        <div class="card">
            <h2>{codelab.title}</h2>
            <p>{codelab.description}</p>
        </div>
    {/each}
{/if}
```

## API Client

```typescript
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export async function listCodelabs(): Promise<Codelab[]> {
    const res = await fetch(`${API_URL}/api/codelabs`);
    if (!res.ok) throw new Error('Failed');
    return res.json();
}
```

## Next Steps

- [API Usage Examples](api-usage.md)
- [Architecture](../architecture/frontend.md)
