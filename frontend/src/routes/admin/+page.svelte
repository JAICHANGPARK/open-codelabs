<script lang="ts">
    import { onMount } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import {
        listCodelabs,
        getMyCodelabs,
        onAuthChange,
        isServerlessMode,
        createCodelab,
        importCodelab,
        deleteCodelab,
        getUpdateStatus,
        type Codelab,
    } from "$lib/api";
    import {
        Plus,
        BookOpen,
        Headset,
        User,
        Clock,
        LayoutDashboard,
        Download,
        FileUp,
        Trash2,
        Share2,
        Check,
        Eye,
        Settings,
        Sparkles,
        Github,
        ShieldCheck,
        FileText,
        X,
        Copy,
        Loader2,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";
    import { encrypt, decrypt } from "$lib/crypto";
    import AiCodelabGenerator from "$lib/components/admin/AiCodelabGenerator.svelte";
    import FacilitatorConsultant from "$lib/components/admin/FacilitatorConsultant.svelte";
    import {
        copyCodelab as apiCopyCodelab,
        exportBackup,
        inspectBackup,
        restoreBackup,
        saveAdminSettings,
    } from "$lib/api";

    let codelabs: Codelab[] = $state([]);
    let loading = $state(true);
    let showCreateModal = $state(false);
    let newCodelab = $state({
        title: "",
        description: "",
        author: "",
        is_public: true,
    });
    let copyTarget: string | null = $state(null);

    async function handleCopy(id: string) {
        try {
            loading = true;
            const copied = await apiCopyCodelab(id);
            codelabs = [copied, ...codelabs];
        } catch (e: any) {
            alert(`${$t("common.error")}: ${e.message || e}`);
        } finally {
            loading = false;
        }
    }

    // Grouping logic
    let groupedCodelabs = $derived.by(() => {
        const groups: Record<string, Codelab[]> = {};
        const sorted = [...codelabs].sort((a, b) => {
            const dateA = a.created_at ? new Date(a.created_at).getTime() : 0;
            const dateB = b.created_at ? new Date(b.created_at).getTime() : 0;
            return dateB - dateA; // Newest first
        });

        sorted.forEach((c) => {
            const date = c.created_at
                ? new Date(c.created_at).toLocaleDateString($locale || "en", {
                      year: "numeric",
                      month: "long",
                      day: "numeric",
                  })
                : $t("dashboard.unknown_date");

            if (!groups[date]) {
                groups[date] = [];
            }
            groups[date].push(c);
        });
        return Object.entries(groups);
    });

    // AI & Settings State
    let showSettingsModal = $state(false);
    let isSavingSettings = $state(false);
    let showAiGenerator = $state(false);
    let geminiApiKey = $state("");
    let apiKeySaved = $state(false);
    let showConsultant = $state(false);
    let initialGeneratorContext = $state("");
    let backupFileInput = $state<HTMLInputElement | null>(null);

    let user = $state<any>(null);
    let updateStatus = $state<any | null>(null);
    let updateError = $state("");
    let checkingUpdates = $state(false);

    onMount(async () => {
        onAuthChange((u) => {
            user = u;
            if (isServerlessMode() && u) {
                loadMyCodelabs();
                if (u.displayName) newCodelab.author = u.displayName;
            }
        });

        try {
            if (!isServerlessMode()) {
                checkingUpdates = true;
                updateStatus = await getUpdateStatus();
            }
            if (isServerlessMode()) {
                // If in firebase mode, listCodelabs might return everything (if admin)
                // but we might prefer showing only 'My Codelabs' by default.
                // For now, let's load all if we can, but also provide 'My Codelabs'.
                codelabs = await listCodelabs();
            } else {
                codelabs = await listCodelabs();
            }

            // Load API Key
            const encryptedKey = localStorage.getItem("gemini_api_key");
            if (encryptedKey) {
                const decrypted = decrypt(encryptedKey);
                if (decrypted) {
                    geminiApiKey = decrypted;
                    apiKeySaved = true;
                }
            }
        } catch (e) {
            console.error(e);
            updateError = (e as any)?.message || "";
        } finally {
            checkingUpdates = false;
            loading = false;
        }
    });

    async function loadMyCodelabs() {
        try {
            const mine = await getMyCodelabs();
            // If we want to strictly show only 'My' codelabs in Firebase mode
            if (isServerlessMode()) {
                codelabs = mine;
            }
        } catch (e) {
            console.error(e);
        }
    }

    async function saveSettings() {
        isSavingSettings = true;
        try {
            // 1. Save to server (for proxy use)
            await saveAdminSettings({ gemini_api_key: geminiApiKey.trim() });

            // 2. Save locally
            if (geminiApiKey.trim()) {
                const encrypted = encrypt(geminiApiKey.trim());
                localStorage.setItem("gemini_api_key", encrypted);
                apiKeySaved = true;
                showSettingsModal = false;
            } else {
                localStorage.removeItem("gemini_api_key");
                apiKeySaved = false;
            }
            alert($t("dashboard.settings.save_success"));
        } catch (e: any) {
            console.error(e);
            if (e.message === "ENCRYPTION_PASSWORD_MISSING") {
                const pw = prompt(
                    $t("login.password") + " required for encryption:",
                );
                if (pw) {
                    sessionStorage.setItem("adminPassword", pw);
                    saveSettings();
                    return;
                }
            } else {
                alert($t("dashboard.settings.save_failed"));
            }
        } finally {
            isSavingSettings = false;
        }
    }

    async function handleCreate() {
        if (!newCodelab.title) return;
        try {
            const created = await createCodelab(newCodelab);
            codelabs = [created, ...codelabs];
            showCreateModal = false;
            newCodelab = {
                title: "",
                description: "",
                author: "",
                is_public: true,
            };
        } catch (e) {
            console.error(e);
        }
    }

    let fileInput: HTMLInputElement;

    async function handleImport(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            loading = true;
            try {
                const imported = await importCodelab(target.files[0]);
                codelabs = [imported, ...codelabs];
            } catch (e: any) {
                alert(`${$t("common.error")}: ${e.message || e}`);
            } finally {
                loading = false;
                target.value = "";
            }
        }
    }

    async function handleBackupExport() {
        try {
            await exportBackup();
        } catch (e: any) {
            alert(`${$t("common.error")}: ${e.message || e}`);
        }
    }

    async function handleBackupRestore(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            try {
                const summary = await inspectBackup(target.files[0]);
                const summaryText = $t("dashboard.backup.restore_summary", {
                    values: {
                        codelabs: summary.codelabs,
                        steps: summary.steps,
                        attendees: summary.attendees,
                        help_requests: summary.help_requests,
                        chat_messages: summary.chat_messages,
                        feedback: summary.feedback,
                        materials: summary.materials,
                        quizzes: summary.quizzes,
                        quiz_submissions: summary.quiz_submissions,
                        submissions: summary.submissions,
                        ai_conversations: summary.ai_conversations,
                        ai_threads: summary.ai_threads,
                        ai_messages: summary.ai_messages,
                        audit_logs: summary.audit_logs,
                        codeserver_workspaces: summary.codeserver_workspaces,
                        uploads_files: summary.uploads_files,
                        workspaces_files: summary.workspaces_files,
                    },
                });
                const confirmed = confirm(
                    `${$t("dashboard.backup.restore_warning")}\n\n${summaryText}`,
                );
                if (!confirmed) return;
                await restoreBackup(target.files[0]);
                alert($t("dashboard.backup.restore_success"));
                window.location.reload();
            } catch (e: any) {
                alert(`${$t("common.error")}: ${e.message || e}`);
            } finally {
                target.value = "";
            }
        }
    }

    async function handleDelete(id: string) {
        if (!confirm($t("dashboard.confirm_delete"))) return;
        try {
            await deleteCodelab(id);
            codelabs = codelabs.filter((c) => c.id !== id);
        } catch (e: any) {
            console.error(e);
            alert(`${$t("common.error")}: ${e.message || e}`);
        }
    }

    async function copyLink(id: string) {
        const url = `${window.location.origin}/codelabs/${id}`;
        try {
            // Check if clipboard API is available
            if (navigator.clipboard && navigator.clipboard.writeText) {
                await navigator.clipboard.writeText(url);
            } else {
                // Fallback for browsers without clipboard API
                const input = document.createElement("input");
                input.value = url;
                input.style.position = "fixed";
                input.style.opacity = "0";
                document.body.appendChild(input);
                input.select();
                document.execCommand("copy");
                document.body.removeChild(input);
            }

            copyTarget = id;
            setTimeout(() => {
                if (copyTarget === id) copyTarget = null;
            }, 2000);
        } catch (err) {
            console.error("Failed to copy!", err);
            alert($t("editor.copy_url") + " " + $t("common.failed"));
        }
    }

    function handleGeneratorClose() {
        showAiGenerator = false;
        initialGeneratorContext = "";
    }
</script>

<div
    class="min-h-screen bg-muted dark:bg-dark-bg transition-colors duration-200"
>
    {#if updateStatus && (updateStatus.frontend?.update_available || updateStatus.backend?.update_available)}
        <div
            class="bg-amber-50 dark:bg-amber-500/10 border-b border-amber-200 dark:border-amber-500/20"
        >
            <div
                class="max-w-6xl mx-auto px-4 sm:px-8 lg:px-12 py-3 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2"
            >
                <div
                    class="text-sm text-amber-800 dark:text-amber-200 font-medium"
                >
                    {$t("dashboard.update_available") || "Update available"}:
                    {#if updateStatus.frontend?.update_available}
                        <span class="font-bold">Frontend</span>
                        {#if updateStatus.frontend?.current}
                            (current {updateStatus.frontend.current})
                        {/if}
                        {#if updateStatus.frontend?.latest}
                            → {updateStatus.frontend.latest}
                        {/if}
                    {/if}
                    {#if updateStatus.backend?.update_available}
                        {#if updateStatus.frontend?.update_available}
                            |
                        {/if}
                        <span class="font-bold">Backend</span>
                        {#if updateStatus.backend?.current}
                            (current {updateStatus.backend.current})
                        {/if}
                        {#if updateStatus.backend?.latest}
                            → {updateStatus.backend.latest}
                        {/if}
                    {/if}
                </div>
                <div class="text-xs text-amber-700 dark:text-amber-300">
                    {$t("dashboard.update_hint") ||
                        "Pull the latest Docker images to update."}
                </div>
            </div>
        </div>
    {/if}
    <div class="max-w-6xl mx-auto p-4 sm:p-8 lg:p-12">
        <header
            class="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-6 mb-8 sm:mb-12"
        >
            <div class="w-full lg:w-auto">
                <div class="flex items-center gap-3 mb-2">
                    <div
                        class="w-10 h-10 bg-primary rounded-lg flex items-center justify-center text-white shadow-sm shrink-0"
                    >
                        <LayoutDashboard size={24} />
                    </div>
                    <h1
                        class="text-2xl sm:text-3xl font-bold text-foreground dark:text-dark-text truncate"
                    >
                        {$t("dashboard.title")}
                    </h1>
                </div>
                <p
                    class="text-muted-foreground dark:text-dark-text-muted text-base sm:text-lg"
                >
                    {$t("dashboard.subtitle")}
                </p>
            </div>
            <div
                class="flex flex-wrap items-center gap-2 sm:gap-4 w-full lg:w-auto"
            >
                <a
                    href="/admin/audit-logs"
                    class="p-2.5 hover:bg-white dark:hover:bg-dark-surface rounded-full text-muted-foreground dark:text-dark-text-muted transition-all border border-transparent hover:border-border dark:hover:border-dark-border"
                    title="Audit Logs"
                    aria-label="Audit Logs"
                >
                    <ShieldCheck size={20} />
                </a>
                <button
                    onclick={() => (showSettingsModal = true)}
                    class="p-2.5 hover:bg-white dark:hover:bg-dark-surface rounded-full text-muted-foreground dark:text-dark-text-muted transition-all border border-transparent hover:border-border dark:hover:border-dark-border"
                    title={$t("dashboard.settings.title")}
                    aria-label={$t("dashboard.settings.title")}
                >
                    <Settings
                        size={20}
                        class={apiKeySaved ? "text-emerald-600" : ""}
                    />
                </button>
                <div
                    class="h-6 w-px bg-border dark:bg-dark-border hidden sm:block"
                ></div>

                <div class="hidden lg:flex items-center gap-1">
                    <a
                        href="https://github.com/JAICHANGPARK/open-codelabs"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="p-2 text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:bg-muted dark:hover:bg-white/5 rounded-full transition-all"
                        title="GitHub Repository"
                        aria-label="GitHub Repository"
                    >
                        <Github size={20} />
                    </a>
                    <a
                        href="https://jaichangpark.github.io/open-codelabs/"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="p-2 text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:bg-muted dark:hover:bg-white/5 rounded-full transition-all"
                        title="Documentation"
                        aria-label="Documentation"
                    >
                        <FileText size={20} />
                    </a>
                    <div
                        class="h-6 w-px bg-border dark:bg-dark-border mx-1"
                    ></div>
                </div>

                <input
                    type="file"
                    accept=".zip"
                    bind:this={fileInput}
                    onchange={handleImport}
                    class="hidden"
                />
                <button
                    onclick={() => fileInput.click()}
                    class="bg-white dark:bg-dark-surface hover:bg-muted dark:hover:bg-white/5 text-muted-foreground dark:text-dark-text-muted px-3 sm:px-4 py-2 sm:py-2.5 rounded-full flex items-center gap-2 transition-all border border-border dark:border-dark-border font-bold text-xs sm:text-sm"
                >
                    <FileUp size={18} />
                    <span class="hidden xs:inline">{$t("common.import")}</span>
                </button>
                <button
                    onclick={() => (showConsultant = true)}
                    class="bg-white dark:bg-dark-surface hover:bg-muted dark:hover:bg-white/5 text-muted-foreground dark:text-dark-text-muted px-3 sm:px-4 py-2 sm:py-2.5 rounded-full flex items-center gap-2 transition-all border border-border dark:border-dark-border font-bold text-xs sm:text-sm"
                >
                    <Headset size={18} />
                    <span class="hidden xs:inline"
                        >{$t("admin.consultant.button") || "Consultant"}</span
                    >
                </button>
                <button
                    onclick={() => (showAiGenerator = true)}
                    class="bg-white dark:bg-dark-surface hover:bg-muted dark:hover:bg-white/5 text-primary px-3 sm:px-4 py-2 sm:py-2.5 rounded-full flex items-center gap-2 transition-all border border-border dark:border-dark-border font-bold text-xs sm:text-sm"
                >
                    <Sparkles size={18} />
                    <span class="hidden xs:inline"
                        >{$t("dashboard.create_with_ai")}</span
                    >
                </button>
                <button
                    onclick={() => (showCreateModal = true)}
                    class="bg-primary hover:bg-primary/90 text-white px-4 sm:px-5 py-2 sm:py-2.5 rounded-full flex items-center gap-2 transition-all shadow-md hover:shadow-lg font-bold text-xs sm:text-sm ml-auto lg:ml-0"
                >
                    <Plus size={20} />
                    <span>{$t("dashboard.new_codelab")}</span>
                </button>
            </div>
        </header>

        {#if loading}
            <div class="flex justify-center items-center py-20" in:fade>
                <div
                    class="animate-spin rounded-full h-12 w-12 border-4 border-border dark:border-dark-border border-t-primary dark:border-t-primary"
                ></div>
            </div>
        {:else if codelabs.length === 0}
            <div
                class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl p-8 sm:p-16 text-center shadow-sm"
                in:fly={{ y: 20, duration: 500 }}
            >
                <div
                    class="bg-muted dark:bg-white/5 w-16 sm:w-20 h-16 sm:h-20 rounded-full flex items-center justify-center mx-auto mb-6"
                >
                    <BookOpen
                        size={32}
                        class="text-muted-foreground/60 dark:text-dark-text-muted"
                    />
                </div>
                <h3
                    class="text-xl font-bold text-foreground dark:text-dark-text"
                >
                    {$t("dashboard.no_codelabs")}
                </h3>
                <p
                    class="text-muted-foreground dark:text-dark-text-muted mt-2 text-base sm:text-lg"
                >
                    {$t("dashboard.get_started")}
                </p>
                <button
                    onclick={() => (showCreateModal = true)}
                    class="mt-8 text-primary font-bold hover:text-primary flex items-center gap-2 mx-auto px-6 py-2 rounded-full border border-border dark:border-dark-border hover:bg-accent/70 dark:hover:bg-primary/10 transition-all"
                >
                    {$t("dashboard.create_first")}
                    <Plus size={18} />
                </button>
            </div>
        {:else}
            <div class="space-y-12">
                {#each groupedCodelabs as [date, list]}
                    <section>
                        <div class="flex items-center gap-2 mb-6">
                            <div class="h-8 w-1 bg-primary rounded-full"></div>
                            <h2
                                class="text-xl font-bold text-foreground dark:text-dark-text"
                            >
                                {date}
                            </h2>
                            <span
                                class="bg-accent/60 dark:bg-white/10 text-muted-foreground dark:text-dark-text-muted px-2 py-0.5 rounded-md text-xs font-bold"
                            >
                                {list.length}
                            </span>
                        </div>
                        <div
                            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8"
                        >
                            {#each list as codelab, i}
                                <div
                                    in:fly={{
                                        y: 20,
                                        delay: i * 50,
                                        duration: 500,
                                    }}
                                >
                                    <a
                                        href="/admin/{codelab.id}"
                                        class="group block bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl p-6 sm:p-8 hover:shadow-xl transition-all duration-300 hover:border-primary dark:hover:border-primary relative overflow-hidden h-full flex flex-col"
                                    >
                                        <div
                                            class="flex items-start justify-between gap-4 mb-4"
                                        >
                                            <h3
                                                class="text-xl font-bold text-foreground dark:text-dark-text group-hover:text-primary transition-colors flex flex-wrap items-center gap-2 flex-1"
                                            >
                                                <span>{codelab.title}</span>
                                                {#if !codelab.is_public}
                                                    <span
                                                        class="bg-gray-100 dark:bg-white/10 text-gray-500 dark:text-dark-text-muted text-[10px] px-2 py-0.5 rounded-full border dark:border-dark-border flex items-center gap-1 font-bold shrink-0"
                                                    >
                                                        <X size={10} />
                                                        {$t("common.private")}
                                                    </span>
                                                {/if}
                                            </h3>
                                            <div
                                                class="flex items-center gap-1 sm:gap-2 shrink-0"
                                            >
                                                <button
                                                    onclick={(e) => {
                                                        e.preventDefault();
                                                        copyLink(codelab.id);
                                                    }}
                                                    class="p-2 transition-all rounded-full {copyTarget ===
                                                    codelab.id
                                                        ? 'bg-emerald-50 dark:bg-emerald-600/20 text-emerald-700 dark:text-emerald-600'
                                                        : 'bg-muted dark:bg-white/5 text-muted-foreground dark:text-dark-text-muted hover:bg-accent/70 dark:hover:bg-primary/10 hover:text-primary'}"
                                                    title={$t(
                                                        "dashboard.share_link",
                                                    )}
                                                    aria-label={$t(
                                                        "dashboard.share_link",
                                                    )}
                                                >
                                                    {#if copyTarget === codelab.id}
                                                        <Check size={18} />
                                                    {:else}
                                                        <Share2 size={18} />
                                                    {/if}
                                                </button>
                                                <button
                                                    onclick={(e) => {
                                                        e.preventDefault();
                                                        handleCopy(codelab.id);
                                                    }}
                                                    class="p-2 bg-muted dark:bg-white/5 text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:bg-accent/70 dark:hover:bg-primary/10 rounded-full transition-all"
                                                    title={$t("common.copy") ||
                                                        "Copy"}
                                                    aria-label={$t(
                                                        "common.copy",
                                                    ) || "Copy"}
                                                >
                                                    <Copy size={18} />
                                                </button>
                                                <button
                                                    onclick={(e) => {
                                                        e.preventDefault();
                                                        handleDelete(
                                                            codelab.id,
                                                        );
                                                    }}
                                                    class="p-2 bg-muted dark:bg-white/5 text-muted-foreground dark:text-dark-text-muted hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-full transition-all"
                                                    title={$t(
                                                        "common.delete",
                                                    ) || "Delete"}
                                                    aria-label={$t(
                                                        "common.delete",
                                                    ) || "Delete"}
                                                >
                                                    <Trash2 size={18} />
                                                </button>
                                            </div>
                                        </div>

                                        <p
                                            class="text-muted-foreground dark:text-dark-text-muted text-base line-clamp-2 mb-8 flex-1"
                                        >
                                            {codelab.description}
                                        </p>
                                        <div
                                            class="flex items-center justify-between border-t border-border dark:border-dark-border pt-6"
                                        >
                                            <div
                                                class="flex items-center gap-2 text-muted-foreground dark:text-dark-text-muted text-sm font-medium min-w-0"
                                            >
                                                <div
                                                    class="w-6 h-6 rounded-full bg-accent/60 dark:bg-white/5 flex items-center justify-center shrink-0"
                                                >
                                                    <User size={14} />
                                                </div>
                                                <span class="truncate"
                                                    >{codelab.author}</span
                                                >
                                            </div>
                                            <div
                                                class="flex items-center gap-1.5 text-muted-foreground/80 dark:text-dark-text-muted text-[10px] sm:text-xs font-medium uppercase tracking-wider shrink-0"
                                            >
                                                <Clock size={14} />
                                                <span class="hidden xs:inline">
                                                    {new Date(
                                                        codelab.created_at ||
                                                            "",
                                                    ).toLocaleTimeString(
                                                        $locale || "en",
                                                        {
                                                            hour: "2-digit",
                                                            minute: "2-digit",
                                                        },
                                                    )}
                                                </span>
                                            </div>
                                        </div>
                                    </a>
                                </div>
                            {/each}
                        </div>
                    </section>
                {/each}
            </div>
        {/if}
    </div>
</div>

{#if showCreateModal}
    <div
        class="fixed inset-0 bg-foreground/60 dark:bg-black/80 backdrop-blur-sm flex items-center justify-center p-4 z-50"
        transition:fade={{ duration: 200 }}
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-modal-title"
    >
        <div
            class="bg-white dark:bg-dark-surface rounded-3xl shadow-2xl w-full max-w-lg overflow-hidden"
            in:fly={{ y: 40, duration: 400 }}
        >
            <div class="bg-primary p-6 sm:p-8 text-white">
                <h2
                    id="create-modal-title"
                    class="text-xl sm:text-2xl font-bold mb-2"
                >
                    {$t("dashboard.create_new_title")}
                </h2>
                <p class="opacity-80 text-sm sm:text-base">
                    {$t("dashboard.design_experience")}
                </p>
            </div>

            <div class="p-6 sm:p-8 space-y-6">
                <div>
                    <label
                        for="new-title"
                        class="block text-xs font-bold text-muted-foreground dark:text-dark-text-muted mb-2 uppercase tracking-wide"
                        >{$t("dashboard.codelab_title")}</label
                    >
                    <input
                        id="new-title"
                        type="text"
                        bind:value={newCodelab.title}
                        placeholder={$t("dashboard.placeholder_title")}
                        class="w-full bg-muted dark:bg-dark-bg border-2 border-border dark:border-dark-border rounded-xl px-4 py-3 focus:border-primary dark:focus:border-primary outline-none transition-all placeholder:text-muted-foreground/60 text-foreground dark:text-dark-text"
                    />
                </div>
                <div>
                    <label
                        for="new-desc"
                        class="block text-xs font-bold text-muted-foreground dark:text-dark-text-muted mb-2 uppercase tracking-wide"
                        >{$t("dashboard.codelab_desc")}</label
                    >
                    <textarea
                        id="new-desc"
                        bind:value={newCodelab.description}
                        placeholder={$t("dashboard.placeholder_desc")}
                        class="w-full bg-muted dark:bg-dark-bg border-2 border-border dark:border-dark-border rounded-xl px-4 py-3 focus:border-primary dark:focus:border-primary outline-none h-32 resize-none transition-all placeholder:text-muted-foreground/60 text-foreground dark:text-dark-text"
                    ></textarea>
                </div>
                <div>
                    <label
                        for="new-author"
                        class="block text-xs font-bold text-muted-foreground dark:text-dark-text-muted mb-2 uppercase tracking-wide"
                        >{$t("dashboard.codelab_author")}</label
                    >
                    <input
                        id="new-author"
                        type="text"
                        bind:value={newCodelab.author}
                        placeholder={$t("dashboard.placeholder_author")}
                        class="w-full bg-muted dark:bg-dark-bg border-2 border-border dark:border-dark-border rounded-xl px-4 py-3 focus:border-primary dark:focus:border-primary outline-none transition-all placeholder:text-muted-foreground/60 text-foreground dark:text-dark-text"
                    />
                </div>
                <div
                    class="flex items-center justify-between p-4 bg-muted dark:bg-dark-bg rounded-xl border-2 border-border dark:border-dark-border"
                >
                    <span
                        class="text-sm font-bold text-muted-foreground dark:text-dark-text-muted"
                        >{$t("common.visibility")}</span
                    >
                    <button
                        onclick={() =>
                            (newCodelab.is_public = !newCodelab.is_public)}
                        class="relative inline-flex h-7 w-12 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 {newCodelab.is_public
                            ? 'bg-emerald-600'
                            : 'bg-gray-200 dark:bg-dark-border'}"
                        role="switch"
                        aria-checked={newCodelab.is_public}
                        title={newCodelab.is_public
                            ? $t("common.public")
                            : $t("common.private")}
                    >
                        <span
                            class="pointer-events-none flex h-5 w-5 items-center justify-center rounded-full bg-white shadow-sm ring-0 transition-transform duration-200 {newCodelab.is_public
                                ? 'translate-x-6'
                                : 'translate-x-1'}"
                        >
                            {#if newCodelab.is_public}
                                <Eye size={12} class="text-emerald-600" />
                            {:else}
                                <X size={12} class="text-red-500" />
                            {/if}
                        </span>
                    </button>
                </div>
            </div>

            <div class="px-6 sm:px-8 pb-8 flex justify-end gap-3 sm:gap-4">
                <button
                    onclick={() => (showCreateModal = false)}
                    class="px-5 sm:px-6 py-2.5 text-muted-foreground dark:text-dark-text-muted font-bold hover:bg-muted dark:hover:bg-white/5 rounded-full transition-all"
                >
                    {$t("common.cancel")}
                </button>
                <button
                    onclick={handleCreate}
                    class="px-6 sm:px-8 py-2.5 bg-primary text-white rounded-full font-bold hover:bg-primary/90 shadow-md transition-all active:scale-95"
                >
                    {$t("common.create")}
                </button>
            </div>
        </div>
    </div>
{/if}

{#if showSettingsModal}
    <div
        class="fixed inset-0 bg-foreground/60 dark:bg-black/80 backdrop-blur-sm flex items-center justify-center p-4 z-50"
        transition:fade={{ duration: 200 }}
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-modal-title"
    >
        <div
            class="bg-white dark:bg-dark-surface rounded-2xl shadow-2xl w-full max-w-md overflow-hidden"
            in:fly={{ y: 20, duration: 300 }}
        >
            <div
                class="px-6 py-4 border-b border-border dark:border-dark-border flex items-center justify-between bg-muted dark:bg-white/5"
            >
                <h3
                    id="settings-modal-title"
                    class="font-bold text-foreground dark:text-dark-text"
                >
                    {$t("dashboard.settings.title")}
                </h3>
                <button
                    onclick={() => (showSettingsModal = false)}
                    class="text-muted-foreground dark:text-dark-text-muted hover:bg-accent/70 dark:hover:bg-white/10 p-1 rounded-full transition-colors"
                    aria-label="Close settings"><X size={18} /></button
                >
            </div>
            <div class="p-6 space-y-4">
                <div>
                    <label
                        for="api-key"
                        class="block text-sm font-bold text-muted-foreground dark:text-dark-text-muted mb-2"
                        >{$t("dashboard.settings.gemini_api_key")}</label
                    >
                    <input
                        id="api-key"
                        type="password"
                        bind:value={geminiApiKey}
                        placeholder={$t(
                            "dashboard.settings.api_key_placeholder",
                        )}
                        class="w-full bg-muted dark:bg-dark-bg border-2 border-border dark:border-dark-border rounded-lg px-4 py-2.5 focus:border-primary dark:focus:border-primary outline-none transition-all placeholder:text-muted-foreground/60 text-sm font-mono text-foreground dark:text-dark-text"
                    />
                    <p class="text-xs text-muted-foreground/80 mt-2">
                        {$t("dashboard.settings.api_key_desc")}
                    </p>
                </div>
                <div
                    class="pt-4 border-t border-border dark:border-dark-border"
                >
                    <div class="flex items-center justify-between mb-2">
                        <h4
                            class="text-sm font-bold text-foreground dark:text-dark-text"
                        >
                            {$t("dashboard.backup.title")}
                        </h4>
                    </div>
                    <p class="text-xs text-muted-foreground/80 mb-3">
                        {$t("dashboard.backup.note")}
                    </p>
                    <div class="flex flex-wrap gap-2">
                        <button
                            onclick={handleBackupExport}
                            class="px-4 py-2 text-muted-foreground dark:text-dark-text-muted font-bold hover:bg-accent/70 dark:hover:bg-white/10 rounded-lg text-sm transition-all border border-border dark:border-dark-border flex items-center gap-2"
                        >
                            <Download size={16} />
                            {$t("dashboard.backup.export_button")}
                        </button>
                        <button
                            onclick={() => backupFileInput?.click()}
                            class="px-4 py-2 text-muted-foreground dark:text-dark-text-muted font-bold hover:bg-accent/70 dark:hover:bg-white/10 rounded-lg text-sm transition-all border border-border dark:border-dark-border flex items-center gap-2"
                        >
                            <FileUp size={16} />
                            {$t("dashboard.backup.restore_button")}
                        </button>
                        <input
                            type="file"
                            accept=".zip"
                            bind:this={backupFileInput}
                            onchange={handleBackupRestore}
                            class="hidden"
                        />
                    </div>
                </div>
            </div>
            <div
                class="px-6 py-4 border-t border-border dark:border-dark-border flex justify-end gap-3 bg-muted dark:bg-white/5"
            >
                <button
                    onclick={() => (showSettingsModal = false)}
                    class="px-4 py-2 text-muted-foreground dark:text-dark-text-muted font-bold hover:bg-accent/70 dark:hover:bg-white/10 rounded-lg text-sm transition-all"
                >
                    {$t("common.cancel")}
                </button>
                <button
                    onclick={saveSettings}
                    disabled={isSavingSettings}
                    class="px-6 py-2 bg-primary text-white rounded-lg font-bold hover:bg-primary/90 text-sm transition-all shadow-sm flex items-center gap-2"
                >
                    {#if isSavingSettings}
                        <Loader2 size={16} class="animate-spin" />
                    {/if}
                    {$t("common.save")}
                </button>
            </div>
        </div>
    </div>
{/if}

{#if showAiGenerator}
    <AiCodelabGenerator
        apiKey={geminiApiKey}
        initialContext={initialGeneratorContext}
        onClose={handleGeneratorClose}
        onCodelabCreated={(codelab) => {
            codelabs = [codelab, ...codelabs];
            handleGeneratorClose();
        }}
    />
{/if}

{#if showConsultant}
    <FacilitatorConsultant
        onClose={() => (showConsultant = false)}
        onGenerateCodelab={(transcript: string) => {
            initialGeneratorContext = transcript;
            showConsultant = false;
            showAiGenerator = true;
        }}
    />
{/if}
