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
        Settings,
        Info,
        FileUp,
        Trophy,
        FolderGit2,
        Award,
    } from "lucide-svelte";
    import * as NavigationMenu from "$lib/components/ui/navigation-menu/index.js";
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
        handleSave,
        handleDownloadWorkspace,
        handleBrowseWorkspace,
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
        handleDownloadWorkspace?: () => void;
        handleBrowseWorkspace?: () => void;
    }>();

    const tabGroups = [
        {
            labelKey: "editor.tab_groups.authoring",
            items: [
                {
                    id: "edit",
                    labelKey: "editor.edit",
                    descriptionKey: "editor.tab_descriptions.edit",
                    icon: Edit3,
                },
                {
                    id: "preview",
                    labelKey: "editor.preview",
                    descriptionKey: "editor.tab_descriptions.preview",
                    icon: Eye,
                },
                {
                    id: "guide",
                    labelKey: "editor.guide_tab",
                    descriptionKey: "editor.tab_descriptions.guide",
                    icon: Info,
                },
                {
                    id: "materials",
                    labelKey: "editor.materials_tab",
                    descriptionKey: "editor.tab_descriptions.materials",
                    icon: Paperclip,
                },
            ],
        },
        {
            labelKey: "editor.tab_groups.operations",
            items: [
                {
                    id: "live",
                    labelKey: "editor.live_tab",
                    descriptionKey: "editor.tab_descriptions.live",
                    icon: Users,
                },
                {
                    id: "feedback",
                    labelKey: "editor.feedback_tab",
                    descriptionKey: "editor.tab_descriptions.feedback",
                    icon: MessageSquare,
                },
                {
                    id: "raffle",
                    labelKey: "editor.raffle_tab",
                    descriptionKey: "editor.tab_descriptions.raffle",
                    icon: Trophy,
                },
            ],
        },
        {
            labelKey: "editor.tab_groups.assessment",
            items: [
                {
                    id: "quiz",
                    labelKey: "editor.quiz_tab",
                    descriptionKey: "editor.tab_descriptions.quiz",
                    icon: Sparkles,
                },
                {
                    id: "submissions",
                    labelKey: "submission.title",
                    descriptionKey: "editor.tab_descriptions.submissions",
                    icon: FileUp,
                },
                {
                    id: "certificate",
                    labelKey: "editor.certificate_tab",
                    descriptionKey: "editor.tab_descriptions.certificate",
                    icon: Award,
                },
            ],
        },
        {
            labelKey: "editor.tab_groups.admin",
            items: [
                {
                    id: "workspace",
                    labelKey: "editor.workspace_tab",
                    descriptionKey: "editor.tab_descriptions.workspace",
                    icon: FolderGit2,
                },
                {
                    id: "settings",
                    labelKey: "editor.settings_tab",
                    descriptionKey: "editor.tab_descriptions.settings",
                    icon: Settings,
                },
                {
                    id: "ai",
                    label: "AI",
                    descriptionKey: "editor.tab_descriptions.ai",
                    icon: Sparkles,
                },
            ],
        },
    ];
</script>

<header
    class="bg-white dark:bg-dark-surface border-b border-[#E8EAED] dark:border-dark-border py-2 sm:py-4 px-3 sm:px-8 sticky top-0 z-40 shadow-sm"
>
    <div class="max-w-screen-2xl mx-auto flex flex-col gap-3">
        <div class="flex justify-between items-center gap-2 sm:gap-3">
            <div class="flex items-center gap-1 sm:gap-6 flex-1 min-w-0">
                <a
                    href="/admin"
                    class="text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text hover:bg-[#F1F3F4] dark:hover:bg-white/5 p-1.5 sm:p-2 rounded-full transition-all shrink-0"
                    aria-label={$t("editor.back")}
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
                            class="text-xs sm:text-lg md:text-xl font-bold text-[#202124] dark:text-dark-text flex items-center gap-1 sm:gap-2 truncate"
                        >
                            <span class="truncate">{codelab?.title}</span>
                            <a
                                href="/codelabs/{id}"
                                target="_blank"
                                class="text-[#4285F4] hover:text-[#1A73E8] shrink-0"
                                title={$t("editor.view_live")}
                                rel="noopener noreferrer"
                                aria-label={$t("editor.view_live")}
                            >
                                <ExternalLink size={14} class="sm:w-4 sm:h-4" />
                            </a>
                        </h1>
                        <p
                            class="text-[9px] sm:text-xs text-[#5F6368] dark:text-dark-text-muted font-medium mt-0.5 hidden xs:block"
                        >
                            ID: {id.split("-")[0]}... &bull; {$t(
                                "editor.facilitator_mode",
                            )}
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
                        aria-label={$t("common.github_repo")}
                    >
                        <Github size={20} />
                    </a>
                    <a
                        href="https://jaichangpark.github.io/open-codelabs/"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                        title={$t("common.documentation")}
                        aria-label={$t("common.documentation")}
                    >
                        <FileText size={20} />
                    </a>
                    <div
                        class="w-px h-6 bg-[#E8EAED] dark:bg-dark-border mx-1"
                    ></div>
                </div>
                <button
                    type="button"
                    onclick={toggleVisibility}
                    class="relative inline-flex h-6 w-10 sm:h-7 sm:w-12 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-[#4285F4] focus:ring-offset-2 {codelab?.is_public
                        ? 'bg-[#34A853]'
                        : 'bg-gray-200 dark:bg-dark-border'}"
                    role="switch"
                    aria-checked={codelab?.is_public != 0}
                    aria-label={$t("common.visibility")}
                    title={codelab?.is_public
                        ? $t("common.public")
                        : $t("common.private")}
                >
                    <span
                        class="pointer-events-none flex h-4 w-4 sm:h-5 sm:w-5 items-center justify-center rounded-full bg-white shadow-sm ring-0 transition-transform duration-200 {codelab?.is_public
                            ? 'translate-x-5 sm:translate-x-6'
                            : 'translate-x-1'}"
                    >
                        {#if codelab?.is_public}
                            <Eye
                                size={10}
                                class="text-[#34A853] sm:w-3 sm:h-3"
                            />
                        {:else}
                            <X size={10} class="text-[#EA4335] sm:w-3 sm:h-3" />
                        {/if}
                    </span>
                </button>
                <div
                    class="h-6 w-px bg-[#E8EAED] dark:bg-dark-border hidden sm:block"
                ></div>
                <button
                    type="button"
                    onclick={handleExport}
                    class="p-1.5 sm:p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                    title={$t("editor.export_codelab")}
                    aria-label={$t("editor.export_codelab")}
                >
                    <Download size={18} class="sm:w-5 sm:h-5" />
                </button>

                {#if handleBrowseWorkspace}
                    <button
                        type="button"
                        onclick={handleBrowseWorkspace}
                        class="p-1.5 sm:p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                        title={$t("workspace.actions.browse")}
                        aria-label={$t("workspace.actions.browse")}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="sm:w-5 sm:h-5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                    </button>
                {/if}

                {#if handleDownloadWorkspace}
                    <button
                        type="button"
                        onclick={handleDownloadWorkspace}
                        class="p-1.5 sm:p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                        title={$t("workspace.actions.download")}
                        aria-label={$t("workspace.actions.download")}
                    >
                        <FileUp size={18} class="sm:w-5 sm:h-5" />
                    </button>
                {/if}

                <button
                    onclick={handleSave}
                    disabled={isSaving ||
                        (mode !== "edit" &&
                            mode !== "quiz" &&
                            mode !== "settings")}
                    class="bg-[#4285F4] hover:bg-[#1A73E8] disabled:opacity-50 text-white p-1.5 sm:px-6 sm:py-2.5 rounded-full flex items-center gap-1 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all shadow-md active:scale-95 {saveSuccess
                        ? 'bg-[#1E8E3E]'
                        : ''}"
                >
                    {#if isSaving}
                        <Loader2
                            size={16}
                            class="animate-spin sm:w-4.5 sm:h-4.5"
                        />
                    {:else if saveSuccess}
                        <CheckCircle2 size={16} class="sm:w-4.5 sm:h-4.5" />
                    {:else}
                        <Save size={16} class="sm:w-4.5 sm:h-4.5" />
                    {/if}
                    <span class="hidden xs:inline">{$t("common.save")}</span>
                </button>
            </div>
        </div>

        <!-- Mode Switcher - Navigation Menu -->
        <NavigationMenu.Root
            viewport={false}
            class="w-full !max-w-full justify-start"
        >
            <NavigationMenu.List
                class="flex flex-1 flex-wrap items-center justify-start gap-2 pb-1"
            >
                {#each tabGroups as group}
                    <NavigationMenu.Item>
                        <NavigationMenu.Trigger
                            class="!h-7 sm:!h-8 !px-3 sm:!px-4 !rounded-full text-[9px] sm:text-[10px] font-semibold uppercase tracking-wider border border-[#E8EAED] dark:border-dark-border bg-[#F1F3F4] dark:bg-white/5 text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text data-[state=open]:bg-white data-[state=open]:text-[#4285F4] data-[state=open]:shadow-sm dark:data-[state=open]:bg-dark-surface {group.items.some((item) => item.id === mode)
                                ? '!bg-white dark:!bg-dark-surface text-[#4285F4] shadow-sm'
                                : ''}"
                        >
                            {$t(group.labelKey)}
                        </NavigationMenu.Trigger>
                        <NavigationMenu.Content
                            class="z-50 rounded-2xl border border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface shadow-xl p-2 w-[min(520px,calc(100vw-32px))] md:w-[520px]"
                        >
                            <div class="grid gap-2 sm:grid-cols-2">
                                {#each group.items as item}
                                    <NavigationMenu.Link
                                        href="#"
                                        onclick={(event) => {
                                            event.preventDefault();
                                            mode = item.id;
                                        }}
                                        aria-current={mode === item.id ? "page" : undefined}
                                        class="flex flex-row items-start gap-2 rounded-lg px-3 py-2 text-[11px] sm:text-sm font-semibold transition-colors {mode ===
                                        item.id
                                            ? 'bg-[#E8F0FE] dark:bg-[#4285F4]/10 text-[#4285F4]'
                                            : 'text-[#5F6368] dark:text-dark-text-muted hover:bg-[#F8F9FA] dark:hover:bg-white/5 hover:text-[#202124] dark:hover:text-dark-text'}"
                                    >
                                        <svelte:component
                                            this={item.icon}
                                            size={14}
                                            class="mt-0.5"
                                        />
                                        <span class="flex-1">
                                            <span class="block">
                                                {item.labelKey
                                                    ? $t(item.labelKey)
                                                    : item.label}
                                            </span>
                                            <span class="block text-[10px] font-normal text-[#9AA0A6] dark:text-dark-text-muted">
                                                {$t(item.descriptionKey)}
                                            </span>
                                        </span>
                                    </NavigationMenu.Link>
                                {/each}
                            </div>
                        </NavigationMenu.Content>
                    </NavigationMenu.Item>
                {/each}
            </NavigationMenu.List>
        </NavigationMenu.Root>
    </div>
</header>
