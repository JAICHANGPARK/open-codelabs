<script lang="ts">
    import { 
        ChevronLeft, 
        ExternalLink, 
        Github, 
        FileText, 
        Eye, 
        X, 
        Download, 
        Edit3, 
        Users, 
        MessageSquare, 
        Paperclip, 
        Sparkles, 
        Loader2, 
        CheckCircle2, 
        Save,
        Settings
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import type { Codelab } from "$lib/api";

    let { 
        id, 
        codelab, 
        loading, 
        mode = $bindable(), 
        isSaving, 
        saveSuccess,
        toggleVisibility,
        handleExport,
        handleSave
    } = $props<{
        id: string;
        codelab: Codelab | null;
        loading: boolean;
        mode: string;
        isSaving: boolean;
        saveSuccess: boolean;
        toggleVisibility: () => void;
        handleExport: () => void;
        handleSave: () => void;
    }>();
</script>

<header
    class="bg-white dark:bg-dark-surface border-b border-[#E8EAED] dark:border-dark-border py-3 sm:py-4 px-4 sm:px-8 sticky top-0 z-40 shadow-sm"
>
    <div class="max-w-screen-2xl mx-auto flex justify-between items-center gap-2 sm:gap-3">
        <div class="flex items-center gap-1 sm:gap-6 flex-1 min-w-0">
            <a
                href="/admin"
                class="text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text hover:bg-[#F1F3F4] dark:hover:bg-white/5 p-1.5 sm:p-2 rounded-full transition-all shrink-0"
                aria-label="Back to dashboard"
            >
                <ChevronLeft size={24} />
            </a>
            <div class="min-w-0 flex-1">
                {#if loading}
                    <div
                        class="h-5 sm:h-6 w-32 md:w-48 bg-[#F1F3F4] dark:bg-white/5 animate-pulse rounded"
                    ></div>
                {:else}
                    <h1
                        class="text-sm sm:text-lg md:text-xl font-bold text-[#202124] dark:text-dark-text flex items-center gap-2 truncate"
                    >
                        <span class="truncate">{codelab?.title}</span>
                        <a
                            href="/codelabs/{id}"
                            target="_blank"
                            class="text-[#4285F4] hover:text-[#1A73E8] shrink-0"
                            title={$t("editor.view_live")}
                        >
                            <ExternalLink size={16} />
                        </a>
                    </h1>
                    <p
                        class="text-[10px] sm:text-xs text-[#5F6368] dark:text-dark-text-muted font-medium mt-0.5 hidden xs:block"
                    >
                        ID: {id.split('-')[0]}... &bull; {$t("editor.facilitator_mode")}
                    </p>
                {/if}
            </div>
        </div>
        <div class="flex items-center gap-1 sm:gap-2 lg:gap-4 shrink-0">
            <div class="hidden md:flex items-center gap-2">
                <a
                    href="https://github.com/JAICHANGPARK/open-codelabs"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                    title={$t("common.github_repo")}
                >
                    <Github size={20} />
                </a>
                <a
                    href="https://jaichangpark.github.io/open-codelabs/"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                    title={$t("common.documentation")}
                >
                    <FileText size={20} />
                </a>
                <div class="w-px h-6 bg-[#E8EAED] dark:bg-dark-border mx-1"></div>
            </div>
            <button
                onclick={toggleVisibility}
                class="relative inline-flex h-7 w-12 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-[#4285F4] focus:ring-offset-2 {codelab?.is_public ? 'bg-[#34A853]' : 'bg-gray-200 dark:bg-dark-border'}"
                role="switch"
                aria-checked={codelab?.is_public != 0}
                title={codelab?.is_public ? $t("common.public") : $t("common.private")}
            >
                <span
                    class="pointer-events-none flex h-5 w-5 items-center justify-center rounded-full bg-white shadow-sm ring-0 transition-transform duration-200 {codelab?.is_public ? 'translate-x-6' : 'translate-x-1'}"
                >
                    {#if codelab?.is_public}
                        <Eye size={12} class="text-[#34A853]" />
                    {:else}
                        <X size={12} class="text-[#EA4335]" />
                    {/if}
                </span>
            </button>
            <div class="h-6 w-px bg-[#E8EAED] dark:bg-dark-border hidden sm:block"></div>
            <button
                onclick={handleExport}
                class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                title={$t("editor.export_codelab")}
            >
                <Download size={20} />
            </button>
            <div
                class="flex bg-[#F1F3F4] dark:bg-white/5 p-1 rounded-full border border-[#E8EAED] dark:border-dark-border"
            >
                <button
                    onclick={() => (mode = "edit")}
                    class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                    'edit'
                        ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                        : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                >
                    <Edit3 size={14} />
                    <span class="hidden sm:inline">{$t("editor.edit")}</span>
                </button>
                <button
                    onclick={() => (mode = "preview")}
                    class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                    'preview'
                        ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                        : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                >
                    <Eye size={14} />
                    <span class="hidden sm:inline">{$t("editor.preview")}</span>
                </button>
                <button
                    onclick={() => (mode = "live")}
                    class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                    'live'
                        ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                        : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                >
                    <Users size={14} />
                    <span class="hidden sm:inline">{$t("editor.live_tab")}</span>
                </button>
                <button
                    onclick={() => (mode = "feedback")}
                    class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                    'feedback'
                        ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                        : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                >
                    <MessageSquare size={14} />
                    <span class="hidden sm:inline">{$t("editor.feedback_tab")}</span>
                </button>
                <button
                    onclick={() => (mode = "materials")}
                    class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                    'materials'
                        ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                        : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                >
                    <Paperclip size={14} />
                    <span class="hidden sm:inline">{$t("editor.materials_tab")}</span>
                </button>
                <button
                    onclick={() => (mode = "quiz")}
                    class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                    'quiz'
                        ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                        : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                >
                    <Sparkles size={14} />
                    <span class="hidden sm:inline">{$t("editor.quiz_tab")}</span>
                </button>
                <button
                    onclick={() => (mode = "settings")}
                    class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                    'settings'
                        ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                        : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                >
                    <Settings size={14} />
                    <span class="hidden sm:inline">{$t("editor.settings_tab")}</span>
                </button>
            </div>
            <button
                onclick={handleSave}
                disabled={isSaving || mode !== "edit"}
                class="bg-[#4285F4] hover:bg-[#1A73E8] disabled:opacity-50 text-white p-2 sm:px-6 sm:py-2.5 rounded-full flex items-center gap-2 text-sm font-bold transition-all shadow-md active:scale-95 {saveSuccess
                    ? 'bg-[#1E8E3E]'
                    : ''}"
            >
                {#if isSaving}
                    <Loader2 size={18} class="animate-spin" />
                {:else if saveSuccess}
                    <CheckCircle2 size={18} />
                {:else}
                    <Save size={18} />
                {/if}
                <span class="hidden sm:inline">{$t("common.save")}</span>
            </button>
        </div>
    </div>
</header>
