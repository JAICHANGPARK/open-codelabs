<script lang="ts">
    let { renderedContent }: { renderedContent: string } = $props();
</script>

<div class="prose dark:prose-invert prose-blue max-w-none markdown-body">
    {@html renderedContent}
</div>
