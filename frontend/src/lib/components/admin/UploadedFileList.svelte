<script lang="ts">
    import { FileCode, Trash2 } from "lucide-svelte";
    let { files, onRemove } = $props<{
        files: { name: string; content: string }[];
        onRemove: (index: number) => void;
    }>();
</script>

{#if files.length > 0}
    <div
        class="flex flex-wrap gap-2 max-h-32 overflow-y-auto p-2 bg-white/50 dark:bg-dark-surface/50 rounded-xl border border-dashed border-[#DADCE0] dark:border-dark-border"
    >
        {#each files as file, i}
            <div
                class="flex items-center gap-2 px-3 py-1.5 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-lg text-xs font-medium text-[#3C4043] dark:text-dark-text shadow-sm group"
            >
                <FileCode size={14} class="text-[#4285F4]" />
                <span class="max-w-[150px] truncate">{file.name}</span>
                <button
                    onclick={() => onRemove(i)}
                    class="text-[#9AA0A6] hover:text-[#EA4335] transition-colors"
                    aria-label="Remove file"
                >
                    <Trash2 size={14} />
                </button>
            </div>
        {/each}
    </div>
{/if}
