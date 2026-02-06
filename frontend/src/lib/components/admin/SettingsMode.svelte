<script lang="ts">
    import { 
        Loader2, 
        CheckCircle2, 
        Save,
        Settings,
        Key,
        FileEdit
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { saveAdminSettings, type Codelab } from "$lib/api";
    import { onMount } from "svelte";
    import { decrypt, encrypt } from "$lib/crypto";
    import { fade } from "svelte/transition";

    let {
        codelab = $bindable(),
        isSaving,
        saveSuccess,
        handleSave
    } = $props<{
        codelab: Codelab | null;
        isSaving: boolean;
        saveSuccess: boolean;
        handleSave: () => void;
    }>();

    let geminiApiKey = $state("");
    let isSavingAdminSettings = $state(false);
    let adminSaveSuccess = $state(false);

    onMount(() => {
        const storedKey = localStorage.getItem("gemini_api_key");
        if (storedKey) {
            const decrypted = decrypt(storedKey);
            if (decrypted) geminiApiKey = decrypted;
        }
    });

    async function handleSaveAdminSettings() {
        isSavingAdminSettings = true;
        try {
            // 1. Save to server (for backend proxy use)
            await saveAdminSettings({ gemini_api_key: geminiApiKey });
            
            // 2. Save locally encrypted for UI persistence
            if (geminiApiKey.trim()) {
                localStorage.setItem("gemini_api_key", encrypt(geminiApiKey.trim()));
            } else {
                localStorage.removeItem("gemini_api_key");
            }

            adminSaveSuccess = true;
            setTimeout(() => adminSaveSuccess = false, 3000);
        } catch (e: any) {
            console.error(e);
            if (e.message === 'ENCRYPTION_PASSWORD_MISSING') {
                const pw = prompt($t("login.password") + " required for encryption:");
                if (pw) {
                    sessionStorage.setItem("adminPassword", pw);
                    handleSaveAdminSettings();
                    return;
                }
            } else {
                alert("Failed to save admin settings");
            }
        } finally {
            isSavingAdminSettings = false;
        }
    }
</script>

<div
    class="bg-white dark:bg-dark-surface rounded-2xl border border-border dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
    in:fade
>
    <div class="p-6 sm:p-8 border-b border-border dark:border-dark-border bg-accent/40 dark:bg-white/5 flex flex-col sm:flex-row justify-between items-center gap-4">
        <div class="flex items-center gap-3">
            <div class="p-2 bg-primary/10 rounded-lg text-primary">
                <Settings size={24} />
            </div>
            <div>
                <h3 class="text-xl font-bold text-foreground dark:text-dark-text">{$t("editor.settings_tab")}</h3>
                <p class="text-sm text-muted-foreground dark:text-dark-text-muted">{$t("editor.settings_description")}</p>
            </div>
        </div>
    </div>

    <div class="p-6 sm:p-8 flex-1 overflow-y-auto space-y-12">
        {#if codelab}
            <div class="max-w-3xl space-y-12">
                <!-- Codelab Info Section -->
                <section class="space-y-6">
                    <h4 class="text-sm font-bold text-foreground dark:text-dark-text border-l-4 border-primary pl-3 flex items-center gap-2">
                        <FileEdit size={16} />
                        {$t("dashboard.design_experience") || "Codelab Information"}
                    </h4>
                    
                    <div class="grid grid-cols-1 gap-6 bg-accent/60 dark:bg-white/5 p-6 rounded-2xl border border-border dark:border-dark-border">
                        <div class="space-y-2">
                            <label for="title" class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">{$t("dashboard.codelab_title")}</label>
                            <input
                                id="title"
                                type="text"
                                bind:value={codelab.title}
                                class="w-full bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-xl px-4 py-3 focus:border-primary outline-none transition-all dark:text-dark-text font-bold"
                            />
                        </div>
                        <div class="space-y-2">
                            <label for="description" class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">{$t("dashboard.codelab_desc")}</label>
                            <textarea
                                id="description"
                                bind:value={codelab.description}
                                rows="3"
                                class="w-full bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-xl px-4 py-3 focus:border-primary outline-none transition-all dark:text-dark-text"
                            ></textarea>
                        </div>
                        <div class="space-y-2">
                            <label for="author" class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">{$t("dashboard.codelab_author")}</label>
                            <input
                                id="author"
                                type="text"
                                bind:value={codelab.author}
                                class="w-full bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-xl px-4 py-3 focus:border-primary outline-none transition-all dark:text-dark-text"
                            />
                        </div>
                    </div>
                </section>

                <!-- Requirements Section -->
                <section class="space-y-6">
                    <h4 class="text-sm font-bold text-foreground dark:text-dark-text border-l-4 border-emerald-500 pl-3">
                        {$t("editor.cert_requirements")}
                    </h4>
                    
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <div class="bg-accent/60 dark:bg-white/5 p-6 rounded-2xl border border-border dark:border-dark-border group hover:border-primary transition-all">
                            <label class="flex items-center justify-between cursor-pointer">
                                <div class="space-y-1">
                                    <span class="text-base font-bold text-foreground dark:text-dark-text group-hover:text-primary transition-colors">
                                        {$t("editor.require_quiz")}
                                    </span>
                                    <p class="text-[11px] text-muted-foreground dark:text-dark-text-muted">
                                        {$t("editor.require_quiz_desc")}
                                    </p>
                                </div>
                                <div class="relative flex items-center">
                                    <input type="checkbox" bind:checked={codelab.require_quiz} class="peer sr-only" />
                                    <div class="h-6 w-11 rounded-full bg-border after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:bg-white after:transition-all peer-checked:bg-primary peer-checked:after:translate-x-full peer-focus:ring-2 peer-focus:ring-primary/20"></div>
                                </div>
                            </label>
                        </div>

                        <div class="bg-accent/60 dark:bg-white/5 p-6 rounded-2xl border border-border dark:border-dark-border group hover:border-emerald-500 transition-all">
                            <label class="flex items-center justify-between cursor-pointer">
                                <div class="space-y-1">
                                    <span class="text-base font-bold text-foreground dark:text-dark-text group-hover:text-emerald-600 transition-colors">
                                        {$t("editor.require_feedback")}
                                    </span>
                                    <p class="text-[11px] text-muted-foreground dark:text-dark-text-muted">
                                        {$t("editor.require_feedback_desc")}
                                    </p>
                                </div>
                                <div class="relative flex items-center">
                                    <input type="checkbox" bind:checked={codelab.require_feedback} class="peer sr-only" />
                                    <div class="h-6 w-11 rounded-full bg-border after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:bg-white after:transition-all peer-checked:bg-emerald-500 peer-checked:after:translate-x-full peer-focus:ring-2 peer-focus:ring-emerald-500/20"></div>
                                </div>
                            </label>
                        </div>
                    </div>
                </section>

                <!-- Admin Global Settings Section -->
                <section class="space-y-6 pt-6 border-t border-border dark:border-dark-border">
                    <div class="flex items-center justify-between">
                        <h4 class="text-sm font-bold text-foreground dark:text-dark-text border-l-4 border-violet-600 pl-3 flex items-center gap-2">
                            <Key size={16} />
                            {$t("dashboard.settings.title") || "Facilitator Settings"}
                        </h4>
                    </div>

                    <div class="bg-accent/60 dark:bg-white/5 p-6 rounded-2xl border border-border dark:border-dark-border space-y-4">
                        <div class="space-y-2">
                            <label for="gemini-key" class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">{$t("dashboard.settings.gemini_api_key")}</label>
                            <div class="flex gap-2">
                                <input
                                    id="gemini-key"
                                    type="password"
                                    bind:value={geminiApiKey}
                                    placeholder={$t("dashboard.settings.api_key_placeholder")}
                                    class="flex-1 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-xl px-4 py-2.5 focus:border-violet-600 outline-none transition-all dark:text-dark-text font-mono text-sm"
                                />
                                <button
                                    onclick={handleSaveAdminSettings}
                                    disabled={isSavingAdminSettings}
                                    class="bg-violet-600 hover:bg-violet-700 disabled:opacity-50 text-white px-6 py-2.5 rounded-xl font-bold transition-all shadow-sm flex items-center gap-2"
                                >
                                    {#if isSavingAdminSettings}
                                        <Loader2 size={16} class="animate-spin" />
                                    {:else if adminSaveSuccess}
                                        <CheckCircle2 size={16} />
                                    {:else}
                                        <Save size={16} />
                                    {/if}
                                    {$t("common.save")}
                                </button>
                            </div>
                            <p class="text-[10px] text-muted-foreground mt-1 italic">
                                * {$t("dashboard.settings.api_key_desc")}
                            </p>
                        </div>
                    </div>
                </section>
            </div>
        {/if}
    </div>
    
    <div class="p-6 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface flex justify-end">
        <button
            onclick={handleSave}
            disabled={isSaving}
            class="bg-primary hover:bg-primary/90 disabled:opacity-50 text-primary-foreground px-8 py-3 rounded-full font-bold shadow-lg transition-all active:scale-95 flex items-center gap-2"
        >
            {#if isSaving}
                <Loader2 size={20} class="animate-spin" />
            {:else if saveSuccess}
                <CheckCircle2 size={20} />
            {:else}
                <Save size={20} />
            {/if}
            {$t("editor.save_changes") || $t("editor.save_content")}
        </button>
    </div>
</div>
