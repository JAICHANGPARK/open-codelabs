<script lang="ts">
    let { renderedContent } = $props<{ renderedContent: string }>();
</script>

<div class="prose dark:prose-invert prose-blue max-w-none markdown-body">
    {@html renderedContent}
</div>
