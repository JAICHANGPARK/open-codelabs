<script lang="ts">
    import { 
        Plus, 
        Trash2, 
        Sparkles, 
        Loader2, 
        Check, 
        Save,
        CheckCircle2
    } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { t } from "svelte-i18n";
    import type { Quiz, Codelab } from "$lib/api";

    let {
        quizzes = $bindable(),
        numQuizToGenerate = $bindable(),
        isQuizGenerating,
        isSaving,
        saveSuccess,
        generateQuizWithAi,
        addEmptyQuiz,
        removeQuiz,
        handleQuizSave
    } = $props<{
        quizzes: Quiz[];
        numQuizToGenerate: number;
        isQuizGenerating: boolean;
        isSaving: boolean;
        saveSuccess: boolean;
        generateQuizWithAi: () => void;
        addEmptyQuiz: () => void;
        removeQuiz: (index: number) => void;
        handleQuizSave: () => void;
    }>();
</script>

<div
    class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
    in:fade
>
    <div class="p-6 sm:p-8 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/30 dark:bg-white/5 flex flex-col sm:flex-row justify-between items-center gap-4">
        <div>
            <h3 class="text-xl font-bold text-[#202124] dark:text-dark-text">{$t("editor.quiz_tab")}</h3>
            <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">{$t("editor.quiz_settings")}</p>
        </div>
        <div class="flex items-center gap-3">
            <div class="flex items-center gap-2 bg-white dark:bg-dark-surface border border-[#DADCE0] dark:border-dark-border px-3 py-1.5 rounded-xl shadow-sm">
                <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted">{$t("editor.num_questions")}</span>
                <input type="number" bind:value={numQuizToGenerate} min="1" max="10" class="w-12 text-center font-bold outline-none bg-transparent" />
            </div>
            <button
                onclick={generateQuizWithAi}
                disabled={isQuizGenerating}
                class="bg-[#4285F4] hover:bg-[#1A73E8] disabled:opacity-50 text-white px-4 py-2 rounded-xl text-sm font-bold transition-all shadow-md flex items-center gap-2"
            >
                {#if isQuizGenerating}
                    <Loader2 size={16} class="animate-spin" />
                {:else}
                    <Sparkles size={16} />
                {/if}
                {$t("editor.generate_quiz")}
            </button>
        </div>
    </div>

    <div class="p-6 sm:p-8 flex-1 overflow-y-auto space-y-8">
        <div class="space-y-6">
            {#each quizzes as quiz, qIndex}
                <div class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-sm overflow-hidden group">
                    <div class="p-4 bg-[#F8F9FA] dark:bg-white/5 border-b border-[#F1F3F4] dark:border-dark-border flex justify-between items-center">
                        <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-tighter">Question {qIndex + 1}</span>
                        <button onclick={() => removeQuiz(qIndex)} class="p-1.5 text-[#5F6368] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg transition-all">
                            <Trash2 size={16} />
                        </button>
                    </div>
                    <div class="p-6 space-y-6">
                        <div class="space-y-2">
                            <label class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.quiz_question")}</label>
                            <input type="text" bind:value={quiz.question} placeholder="Enter your question here..." class="w-full text-lg font-bold outline-none bg-transparent border-b-2 border-transparent focus:border-[#4285F4] transition-all" />
                        </div>
                        <div class="space-y-3">
                            <label class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.quiz_options")}</label>
                            <div class="grid grid-cols-1 gap-2">
                                {#each quiz.options as option, oIndex}
                                    <div class="flex items-center gap-3 group/opt">
                                        <button 
                                            onclick={() => quiz.correct_answer = oIndex}
                                            class="w-6 h-6 rounded-full flex items-center justify-center border-2 transition-all {quiz.correct_answer === oIndex ? 'bg-[#34A853] border-[#34A853] text-white' : 'border-[#DADCE0] dark:border-dark-border text-transparent hover:border-[#34A853]'}"
                                        >
                                            <Check size={14} />
                                        </button>
                                        <input 
                                            type="text" 
                                            bind:value={quiz.options[oIndex]} 
                                            placeholder="Option {oIndex + 1}"
                                            class="flex-1 bg-[#F8F9FA] dark:bg-white/5 border border-transparent focus:border-[#DADCE0] dark:focus:border-dark-border rounded-xl px-4 py-2 text-sm transition-all {quiz.correct_answer === oIndex ? 'font-bold text-[#137333] dark:text-green-400' : ''}"
                                        />
                                    </div>
                                {/each}
                            </div>
                        </div>
                    </div>
                </div>
            {/each}
            
            <button
                onclick={addEmptyQuiz}
                class="w-full py-4 border-2 border-dashed border-[#DADCE0] dark:border-dark-border rounded-2xl text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:border-[#4285F4] hover:bg-[#F8F9FA] dark:hover:bg-white/5 transition-all flex items-center justify-center gap-2 font-bold"
            >
                <Plus size={20} />
                {$t("editor.add_quiz")}
            </button>
        </div>
    </div>
    
    <div class="p-6 border-t border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface flex justify-end">
                                                <button
                                                    onclick={handleQuizSave}
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
                                                </button>    </div>
</div>
