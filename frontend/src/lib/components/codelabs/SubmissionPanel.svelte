<script lang="ts">
    import { t } from "svelte-i18n";
    import { FileUp, FileText, Trash2, Upload, Link2, ExternalLink } from "lucide-svelte";

    let {
        submissions,
        submitting,
        totalSize,
        onUpload,
        onDelete,
        onLinkSubmit,
        linkSubmitting = false
    } = $props<{
        submissions: { id: string; file_name: string; submission_type?: string; link_url?: string | null }[];
        submitting: boolean;
        totalSize: number;
        onUpload: (e: Event) => void;
        onDelete: (submissionId: string) => void;
        onLinkSubmit: (url: string) => void;
        linkSubmitting?: boolean;
    }>();
    let linkUrl = $state("");
    function handleLinkSubmit() {
        const trimmed = linkUrl.trim();
        if (!trimmed) return;
        onLinkSubmit(trimmed);
        linkUrl = "";
    }
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
                    {#if sub.submission_type === "link"}
                        <ExternalLink size={14} class="text-primary shrink-0" />
                    {:else}
                        <FileText size={14} class="text-muted-foreground shrink-0" />
                    {/if}
                    <span class="text-xs truncate text-muted-foreground dark:text-dark-text-muted">{sub.file_name}</span>
                </div>
                <div class="flex items-center gap-1">
                    {#if sub.submission_type === "link" && sub.link_url}
                        <a
                            href={sub.link_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="p-1 text-muted-foreground hover:text-primary transition-colors"
                            aria-label={$t("submission_panel.open_link")}
                        >
                            <ExternalLink size={14} />
                        </a>
                    {/if}
                    <button 
                        onclick={() => onDelete(sub.id)}
                        class="p-1 text-red-500 opacity-0 group-hover:opacity-100 transition-opacity hover:bg-red-50 dark:hover:bg-red-500/10 rounded"
                        aria-label={$t("submission_panel.delete")}
                    >
                        <Trash2 size={14} />
                    </button>
                </div>
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

        <div class="flex items-start gap-2 w-full">
            <div class="flex-1 relative">
                <Link2 size={14} class="absolute left-3 top-3 text-muted-foreground" />
                <textarea
                    rows="2"
                    bind:value={linkUrl}
                    placeholder={$t("submission_panel.link_placeholder")}
                    class="w-full pl-8 pr-3 py-2 rounded-xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface text-xs text-foreground dark:text-dark-text outline-none focus:border-primary resize-none"
                ></textarea>
            </div>
            <button
                type="button"
                class="px-3 py-2 rounded-xl bg-primary text-primary-foreground text-xs font-bold hover:bg-primary/90 transition-all disabled:opacity-50"
                disabled={linkSubmitting || !linkUrl.trim()}
                onclick={handleLinkSubmit}
            >
                {linkSubmitting ? $t("submission_panel.link_submitting") : $t("submission_panel.link_submit")}
            </button>
        </div>
    </div>
</div>
