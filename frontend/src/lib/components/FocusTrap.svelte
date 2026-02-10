<script lang="ts">
    import type { Snippet } from "svelte";
    let { active = true, children } = $props<{
        active?: boolean;
        children: Snippet;
    }>();
    let container: HTMLElement;

    function handleKeydown(event: KeyboardEvent) {
        if (!active || event.key !== "Tab") return;
        const focusable = Array.from(
            container.querySelectorAll<HTMLElement>(
                'a[href], button, textarea, input, select, [tabindex]:not([tabindex="-1"])',
            ),
        ).filter((el) => !el.hasAttribute("disabled") && el.tabIndex !== -1);

        if (focusable.length === 0) return;

        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        const current = document.activeElement;

        if (event.shiftKey && current === first) {
            event.preventDefault();
            last.focus();
        } else if (!event.shiftKey && current === last) {
            event.preventDefault();
            first.focus();
        }
    }
</script>

<div bind:this={container} onkeydown={handleKeydown} role="presentation">
    {@render children()}
</div>
