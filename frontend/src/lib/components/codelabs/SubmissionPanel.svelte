<script lang="ts">
    import { t } from "svelte-i18n";
    import { FileUp, FileText, Trash2, Upload } from "lucide-svelte";

    let {
        submissions,
        submitting,
        totalSize,
        onUpload,
        onDelete
    } = $props<{
        submissions: { id: string; file_name: string }[];
        submitting: boolean;
        totalSize: number;
        onUpload: (e: Event) => void;
        onDelete: (submissionId: string) => void;
    }>();
</script>

<div class="mt-8 pt-8 border-t border-border dark:border-dark-border px-2 pb-8">
    <h3
        class="text-[11px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-widest mb-4 px-2 flex items-center gap-2"
    >
        <FileUp size={14} />
        {$t("submission_panel.title")}
    </h3>
    <div class="space-y-2 px-2">
        {#each submissions as sub}
            <div class="flex items-center justify-between p-2 rounded-lg bg-white dark:bg-white/5 border border-border dark:border-dark-border group">
        <div class="flex items-center gap-2 min-w-0">
            <FileText size={14} class="text-muted-foreground shrink-0" />
            <span class="text-xs truncate text-muted-foreground dark:text-dark-text-muted">{sub.file_name}</span>
        </div>
        <button 
            onclick={() => onDelete(sub.id)}
            class="p-1 text-red-500 opacity-0 group-hover:opacity-100 transition-opacity hover:bg-red-50 dark:hover:bg-red-500/10 rounded"
            aria-label={$t("submission_panel.delete")}
        >
            <Trash2 size={14} />
        </button>
            </div>
        {/each}

        <label class="flex flex-col items-center justify-center w-full p-4 border-2 border-dashed border-border dark:border-dark-border rounded-xl hover:bg-accent/60 dark:hover:bg-white/5 transition-all cursor-pointer group">
            <div class="flex flex-col items-center justify-center pt-2 pb-2">
                <Upload size={20} class="text-muted-foreground dark:text-dark-text-muted group-hover:text-primary transition-colors mb-2" />
                <p class="text-[10px] text-muted-foreground dark:text-dark-text-muted font-bold text-center">
                    {submitting ? $t("submission_panel.uploading") : $t("submission_panel.upload_btn")}
                </p>
                <p class="text-[9px] text-muted-foreground mt-1">
                    {(
                        totalSize / 1024 / 1024
                    ).toFixed(1)}MB / 10MB
                </p>
            </div>
            <input type="file" class="hidden" onchange={onUpload} disabled={submitting} />
        </label>
    </div>
</div>
