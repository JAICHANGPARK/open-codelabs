<script lang="ts">
    import { 
        Loader2, 
        CheckCircle2, 
        Save,
        Settings
    } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { t } from "svelte-i18n";
    import type { Codelab } from "$lib/api";

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
</script>

<div
    class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
    in:fade
>
    <div class="p-6 sm:p-8 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/30 dark:bg-white/5 flex flex-col sm:flex-row justify-between items-center gap-4">
        <div class="flex items-center gap-3">
            <div class="p-2 bg-[#4285F4]/10 rounded-lg text-[#4285F4]">
                <Settings size={24} />
            </div>
            <div>
                <h3 class="text-xl font-bold text-[#202124] dark:text-dark-text">{$t("editor.settings_tab")}</h3>
                <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">{$t("editor.settings_description")}</p>
            </div>
        </div>
    </div>

    <div class="p-6 sm:p-8 flex-1 overflow-y-auto space-y-8">
        {#if codelab}
            <div class="max-w-2xl">
                <section class="space-y-6">
                    <h4 class="text-sm font-bold text-[#202124] dark:text-dark-text border-l-4 border-[#4285F4] pl-3">
                        {$t("editor.cert_requirements")}
                    </h4>
                    
                    <div class="grid grid-cols-1 gap-4">
                        <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-2xl border border-[#E8EAED] dark:border-dark-border group hover:border-[#4285F4] transition-all">
                            <label class="flex items-center justify-between cursor-pointer">
                                <div class="space-y-1">
                                    <span class="text-base font-bold text-[#3C4043] dark:text-dark-text group-hover:text-[#4285F4] transition-colors">
                                        {$t("editor.require_quiz")}
                                    </span>
                                    <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                                        {$t("editor.require_quiz_desc")}
                                    </p>
                                </div>
                                <div class="relative flex items-center">
                                    <input type="checkbox" bind:checked={codelab.require_quiz} class="peer sr-only" />
                                    <div class="h-6 w-11 rounded-full bg-[#DADCE0] after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:bg-white after:transition-all peer-checked:bg-[#4285F4] peer-checked:after:translate-x-full peer-focus:ring-2 peer-focus:ring-[#4285F4]/20"></div>
                                </div>
                            </label>
                        </div>

                        <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-2xl border border-[#E8EAED] dark:border-dark-border group hover:border-[#34A853] transition-all">
                            <label class="flex items-center justify-between cursor-pointer">
                                <div class="space-y-1">
                                    <span class="text-base font-bold text-[#3C4043] dark:text-dark-text group-hover:text-[#34A853] transition-colors">
                                        {$t("editor.require_feedback")}
                                    </span>
                                    <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                                        {$t("editor.require_feedback_desc")}
                                    </p>
                                </div>
                                <div class="relative flex items-center">
                                    <input type="checkbox" bind:checked={codelab.require_feedback} class="peer sr-only" />
                                    <div class="h-6 w-11 rounded-full bg-[#DADCE0] after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:bg-white after:transition-all peer-checked:bg-[#34A853] peer-checked:after:translate-x-full peer-focus:ring-2 peer-focus:ring-[#34A853]/20"></div>
                                </div>
                            </label>
                        </div>
                    </div>
                </section>
            </div>
        {/if}
    </div>
    
    <div class="p-6 border-t border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface flex justify-end">
        <button
            onclick={handleSave}
            disabled={isSaving}
            class="bg-[#1E8E3E] hover:bg-[#137333] disabled:opacity-50 text-white px-8 py-3 rounded-full font-bold shadow-lg transition-all active:scale-95 flex items-center gap-2"
        >
            {#if isSaving}
                <Loader2 size={20} class="animate-spin" />
            {:else if saveSuccess}
                <CheckCircle2 size={20} />
            {:else}
                <Save size={20} />
            {/if}
            {$t("editor.save_content")}
        </button>
    </div>
</div>
