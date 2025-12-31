<script lang="ts">
    import { 
        FileText, 
        Download, 
        User, 
        Calendar, 
        Search,
        ExternalLink,
        Trash2,
        FileUp,
        HardDrive
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { ASSET_URL, type SubmissionWithAttendee } from "$lib/api";
    import { fade, fly } from "svelte/transition";

    let { 
        submissions = [], 
        onDelete
    } = $props<{
        submissions: SubmissionWithAttendee[];
        onDelete: (attendeeId: string, submissionId: string) => void;
    }>();

    $effect(() => {
        console.log("SubmissionsMode received submissions:", submissions);
    });

    let searchTerm = $state("");
    let filteredSubmissions = $derived(
        submissions.filter(s => 
            s.attendee_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            s.file_name.toLowerCase().includes(searchTerm.toLowerCase())
        )
    );

    function formatSize(bytes: number) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    function isImage(fileName: string) {
        const ext = fileName.split('.').pop()?.toLowerCase();
        return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext || '');
    }

    function formatDate(dateStr?: string) {
        if (!dateStr) return "";
        try {
            return new Date(dateStr).toLocaleString();
        } catch (e) {
            return dateStr;
        }
    }
</script>

<div class="space-y-6" in:fade>
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-2xl border border-[#E8EAED] dark:border-dark-border">
        <div>
            <h2 class="text-xl font-bold text-[#202124] dark:text-dark-text flex items-center gap-2">
                <FileUp size={24} class="text-[#4285F4]" />
                {$t("submission.facilitator_title")}
            </h2>
            <p class="text-sm text-[#5F6368] dark:text-dark-text-muted mt-1">
                {submissions.length} {$t("submission.total_count")}
            </p>
        </div>

        <div class="relative max-w-xs w-full">
            <Search size={18} class="absolute left-3 top-1/2 -translate-y-1/2 text-[#9AA0A6]" />
            <input
                type="text"
                bind:value={searchTerm}
                placeholder={$t("submission.search_placeholder")}
                class="w-full pl-10 pr-4 py-2 bg-white dark:bg-dark-surface border border-[#DADCE0] dark:border-dark-border rounded-full text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 focus:border-[#4285F4] transition-all"
            />
        </div>
    </div>

    {#if filteredSubmissions.length === 0}
        <div class="flex flex-col items-center justify-center py-20 text-center bg-white dark:bg-dark-surface rounded-3xl border-2 border-dashed border-[#DADCE0] dark:border-dark-border">
            <div class="w-20 h-20 bg-[#F1F3F4] dark:bg-white/5 rounded-full flex items-center justify-center mb-6">
                <FileUp size={40} class="text-[#BDC1C6]" />
            </div>
            <h3 class="text-lg font-bold text-[#202124] dark:text-dark-text">{$t("submission.no_submissions")}</h3>
            <p class="text-sm text-[#5F6368] dark:text-dark-text-muted mt-2">
                {searchTerm ? $t("submission.no_search_results") : $t("submission.waiting_submissions")}
            </p>
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
            {#each filteredSubmissions as sub, i}
                <div 
                    class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl p-5 hover:shadow-md transition-all group relative overflow-hidden flex flex-col"
                    in:fly={{ y: 20, delay: i * 50 }}
                >
                    {#if isImage(sub.file_name)}
                        <div class="aspect-video w-full mb-4 rounded-xl overflow-hidden bg-[#F1F3F4] dark:bg-white/5 border border-[#E8EAED] dark:border-dark-border relative group/img">
                            <img 
                                src={`${ASSET_URL}${sub.file_path}`} 
                                alt={sub.file_name}
                                class="w-full h-full object-cover transition-transform duration-500 group-hover/img:scale-110"
                            />
                            <div class="absolute inset-0 bg-black/40 opacity-0 group-hover/img:opacity-100 transition-opacity flex items-center justify-center">
                                <a 
                                    href={`${ASSET_URL}${sub.file_path}`} 
                                    target="_blank"
                                    class="bg-white text-black p-2 rounded-full shadow-lg"
                                >
                                    <ExternalLink size={20} />
                                </a>
                            </div>
                        </div>
                    {/if}
                    
                    <div class="flex items-start justify-between mb-4">
                        <div class="flex items-center gap-3 min-w-0">
                            <div class="w-10 h-10 bg-[#E8F0FE] dark:bg-[#4285F4]/10 rounded-xl flex items-center justify-center shrink-0">
                                <FileText size={20} class="text-[#4285F4]" />
                            </div>
                            <div class="min-w-0">
                                <h4 class="font-bold text-[#202124] dark:text-dark-text truncate text-sm" title={sub.file_name}>
                                    {sub.file_name}
                                </h4>
                                <div class="flex items-center gap-2 text-[11px] text-[#5F6368] dark:text-dark-text-muted mt-0.5">
                                    <span class="flex items-center gap-1 font-medium">
                                        <User size={12} />
                                        {sub.attendee_name}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-2.5 mb-5">
                        <div class="flex items-center justify-between text-[11px]">
                            <span class="text-[#9AA0A6] flex items-center gap-1">
                                <HardDrive size={12} />
                                {$t("submission.file_size")}
                            </span>
                            <span class="font-medium text-[#5F6368] dark:text-dark-text-muted">{formatSize(sub.file_size)}</span>
                        </div>
                        <div class="flex items-center justify-between text-[11px]">
                            <span class="text-[#9AA0A6] flex items-center gap-1">
                                <Calendar size={12} />
                                {$t("submission.submitted_at")}
                            </span>
                            <span class="font-medium text-[#5F6368] dark:text-dark-text-muted">{formatDate(sub.created_at)}</span>
                        </div>
                    </div>

                    <div class="flex items-center gap-2 mt-auto">
                        <a 
                            href={`${ASSET_URL}${sub.file_path}`} 
                            target="_blank"
                            download={sub.file_name}
                            class="flex-1 bg-[#F8F9FA] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] py-2 rounded-xl text-xs font-bold transition-all flex items-center justify-center gap-2 border border-transparent hover:border-[#D2E3FC] dark:hover:border-[#4285F4]/30"
                        >
                            <Download size={14} />
                            {$t("common.download")}
                        </a>
                        <a 
                            href={`${ASSET_URL}${sub.file_path}`} 
                            target="_blank"
                            class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-xl transition-all border border-transparent hover:border-[#D2E3FC] dark:hover:border-[#4285F4]/30"
                            title={$t("common.open_new_tab")}
                        >
                            <ExternalLink size={16} />
                        </a>
                        <button 
                            onclick={() => onDelete(sub.attendee_id, sub.id)}
                            class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#EA4335] hover:bg-red-50 dark:hover:bg-red-500/10 rounded-xl transition-all border border-transparent hover:border-red-100 dark:hover:border-red-500/30"
                            title={$t("common.delete")}
                        >
                            <Trash2 size={16} />
                        </button>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>