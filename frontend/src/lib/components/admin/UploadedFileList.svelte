<script lang="ts">
    import { FileCode, Trash2 } from "lucide-svelte";
    import { t } from "svelte-i18n";
    let { files, onRemove } = $props<{
        files: { name: string; content: string }[];
        onRemove: (index: number) => void;
    }>();
</script>

{#if files.length > 0}
    <div
        class="flex flex-wrap gap-2 max-h-32 overflow-y-auto p-2 bg-white dark:bg-dark-surface rounded-xl border border-border dark:border-dark-border shadow-sm"
    >
        {#each files as file, i}
            <div
                class="flex items-center gap-2 px-3 py-1.5 bg-accent/60 dark:bg-dark-bg border border-border dark:border-dark-border rounded-lg text-xs font-medium text-foreground dark:text-dark-text shadow-sm group"
            >
                <FileCode size={14} class="text-primary" />
                <span class="max-w-[220px] truncate" title={file.name}>
                    {file.name}
                </span>
                <button
                    type="button"
                    onclick={() => onRemove(i)}
                    class="text-muted-foreground hover:text-red-500 transition-colors"
                    aria-label={$t("common.delete")}
                >
                    <Trash2 size={14} />
                </button>
            </div>
        {/each}
    </div>
{/if}
