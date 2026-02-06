<script lang="ts">
    import { 
        Plus, 
        Trash2, 
        Sparkles, 
        Loader2, 
        Check, 
        Save,
        CheckCircle2,
        X,
        ListChecks,
        FileText,
        BarChart3,
        Users2,
        Trophy,
        Clock,
        Settings2
    } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { t } from "svelte-i18n";
    import type { Quiz, QuizSubmissionWithAttendee } from "$lib/api";

    let {
        quizzes = $bindable(),
        quizSubmissions = [],
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
        quizSubmissions: QuizSubmissionWithAttendee[];
        numQuizToGenerate: number;
        isQuizGenerating: boolean;
        isSaving: boolean;
        saveSuccess: boolean;
        generateQuizWithAi: () => void;
        addEmptyQuiz: () => void;
        removeQuiz: (index: number) => void;
        handleQuizSave: () => void;
    }>();

    let activeTab = $state<"edit" | "results">("edit");

    const attendeeStats = $derived(() => {
        const stats = new Map<string, { id: string, name: string, correct: number, total: number, lastSubmitted: string }>();
        quizSubmissions.forEach((sub: QuizSubmissionWithAttendee) => {
            if (!stats.has(sub.attendee_id)) {
                stats.set(sub.attendee_id, { id: sub.attendee_id, name: sub.attendee_name, correct: 0, total: 0, lastSubmitted: sub.created_at || "" });
            }
            const s = stats.get(sub.attendee_id)!;
            s.total++;
            if (sub.is_correct) s.correct++;
            if (sub.created_at && (!s.lastSubmitted || sub.created_at > s.lastSubmitted)) {
                s.lastSubmitted = sub.created_at;
            }
        });
        return Array.from(stats.values()).sort((a, b) => b.lastSubmitted.localeCompare(a.lastSubmitted));
    });

    function addOption(qIndex: number) {
        if (quizzes[qIndex].options.length < 10) {
            quizzes[qIndex].options = [...quizzes[qIndex].options, ""];
        }
    }

    function removeOption(qIndex: number, oIndex: number) {
        if (quizzes[qIndex].options.length > 2) {
            const newOptions = [...quizzes[qIndex].options];
            newOptions.splice(oIndex, 1);
            quizzes[qIndex].options = newOptions;
            
            // Adjust correct_answer if necessary
            if (quizzes[qIndex].correct_answer >= newOptions.length) {
                quizzes[qIndex].correct_answer = newOptions.length - 1;
            } else if (quizzes[qIndex].correct_answer === oIndex) {
                quizzes[qIndex].correct_answer = 0;
            } else if (quizzes[qIndex].correct_answer > oIndex) {
                quizzes[qIndex].correct_answer--;
            }
        }
    }

    function getAttendeeSubmissions(attendeeId: string) {
        return quizSubmissions.filter((s: QuizSubmissionWithAttendee) => s.attendee_id === attendeeId);
    }
</script>

<div
    class="bg-white dark:bg-dark-surface rounded-2xl border border-border dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
    in:fade
>
    <!-- Header with Tabs -->
    <div class="p-6 sm:p-8 border-b border-border dark:border-dark-border bg-muted/30 dark:bg-white/5 flex flex-col sm:flex-row justify-between items-center gap-4">
        <div class="flex items-center gap-6">
            <div>
                <h3 class="text-xl font-bold text-foreground dark:text-dark-text">{$t("editor.quiz_tab")}</h3>
                <p class="text-sm text-muted-foreground dark:text-dark-text-muted">{$t("editor.quiz_settings")}</p>
            </div>
            
            <div class="flex bg-accent/60 dark:bg-white/5 p-1 rounded-xl border border-border dark:border-dark-border ml-4">
                <button 
                    onclick={() => activeTab = 'edit'}
                    class="px-4 py-1.5 rounded-lg flex items-center gap-2 text-xs font-bold transition-all {activeTab === 'edit' ? 'bg-white dark:bg-dark-surface shadow-sm text-primary' : 'text-muted-foreground dark:text-dark-text-muted hover:text-foreground'}"
                >
                    <Settings2 size={14} />
                    {$t("editor.quiz_edit_tab")}
                </button>
                <button 
                    onclick={() => activeTab = 'results'}
                    class="px-4 py-1.5 rounded-lg flex items-center gap-2 text-xs font-bold transition-all {activeTab === 'results' ? 'bg-white dark:bg-dark-surface shadow-sm text-primary' : 'text-muted-foreground dark:text-dark-text-muted hover:text-foreground'}"
                >
                    <BarChart3 size={14} />
                    {$t("editor.quiz_results_tab")}
                    {#if quizSubmissions.length > 0}
                        <span class="bg-primary text-white text-[10px] px-1.5 py-0.5 rounded-full">{attendeeStats().length}</span>
                    {/if}
                </button>
            </div>
        </div>

        {#if activeTab === 'edit'}
            <div class="flex items-center gap-3" in:fade>
                <div class="flex items-center gap-2 bg-white dark:bg-dark-surface border border-border dark:border-dark-border px-3 py-1.5 rounded-xl shadow-sm">
                    <span class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted">{$t("editor.num_questions")}</span>
                    <input type="number" bind:value={numQuizToGenerate} min="1" max="10" aria-label={$t("editor.num_questions")} class="w-12 text-center font-bold outline-none bg-transparent" />
                </div>
                <button
                    onclick={generateQuizWithAi}
                    disabled={isQuizGenerating}
                    class="bg-primary hover:bg-primary/90 disabled:opacity-50 text-white px-4 py-2 rounded-xl text-sm font-bold transition-all shadow-md flex items-center gap-2"
                >
                    {#if isQuizGenerating}
                        <Loader2 size={16} class="animate-spin" />
                    {:else}
                        <Sparkles size={16} />
                    {/if}
                    {$t("editor.generate_quiz")}
                </button>
            </div>
        {/if}
    </div>

    <div class="p-6 sm:p-8 flex-1 overflow-y-auto">
        {#if activeTab === 'edit'}
            <div class="space-y-6" in:fade>
                {#each quizzes as quiz, qIndex}
                    <div class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-sm overflow-hidden group">
                        <div class="p-4 bg-muted dark:bg-white/5 border-b border-border dark:border-dark-border flex justify-between items-center">
                            <div class="flex items-center gap-4">
                                <span class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-tighter">Question {qIndex + 1}</span>
                                <div class="flex bg-gray-200 dark:bg-white/10 p-0.5 rounded-lg">
                                    <button 
                                        onclick={() => quiz.quiz_type = 'multiple_choice'}
                                        class="px-2 py-1 rounded-md text-[10px] font-bold flex items-center gap-1 transition-all {quiz.quiz_type !== 'descriptive' ? 'bg-white dark:bg-dark-surface text-primary shadow-sm' : 'text-muted-foreground dark:text-dark-text-muted hover:text-foreground'}"
                                    >
                                        <ListChecks size={12} />
                                        {$t("editor.multiple_choice")}
                                    </button>
                                    <button 
                                        onclick={() => quiz.quiz_type = 'descriptive'}
                                        class="px-2 py-1 rounded-md text-[10px] font-bold flex items-center gap-1 transition-all {quiz.quiz_type === 'descriptive' ? 'bg-white dark:bg-dark-surface text-primary shadow-sm' : 'text-muted-foreground dark:text-dark-text-muted hover:text-foreground'}"
                                    >
                                        <FileText size={12} />
                                        {$t("editor.descriptive")}
                                    </button>
                                </div>
                            </div>
                            <button type="button" onclick={() => removeQuiz(qIndex)} class="p-1.5 text-muted-foreground hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg transition-all" aria-label={$t("common.delete")}>
                                <Trash2 size={16} />
                            </button>
                        </div>
                        <div class="p-6 space-y-6">
                            <div class="space-y-2">
                                <label for={`quiz-question-${qIndex}`} class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.quiz_question")}</label>
                                <input id={`quiz-question-${qIndex}`} type="text" bind:value={quiz.question} placeholder="Enter your question here..." class="w-full text-lg font-bold outline-none bg-transparent border-b-2 border-transparent focus:border-primary transition-all" />
                            </div>
                            
                            {#if quiz.quiz_type !== 'descriptive'}
                                <div class="space-y-3">
                                    <div class="flex justify-between items-center">
                                        <span id={`quiz-options-label-${qIndex}`} class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.quiz_options")}</span>
                                        <button 
                                            onclick={() => addOption(qIndex)}
                                            class="text-[10px] font-bold text-primary hover:bg-accent/70 dark:hover:bg-primary/10 px-2 py-1 rounded-lg transition-all flex items-center gap-1"
                                        >
                                            <Plus size={12} />
                                            {$t("editor.add_option")}
                                        </button>
                                    </div>
                                    <div class="grid grid-cols-1 gap-2" id={`quiz-options-${qIndex}`} role="group" aria-labelledby={`quiz-options-label-${qIndex}`}>
                                        {#each quiz.options as option, oIndex}
                                            <div class="flex items-center gap-3 group/opt">
                                                <button 
                                                    onclick={() => quiz.correct_answer = oIndex}
                                                    aria-label={$t("editor.quiz_correct")}
                                                    class="w-6 h-6 rounded-full flex items-center justify-center border-2 transition-all {quiz.correct_answer === oIndex ? 'bg-emerald-600 border-emerald-600 text-white' : 'border-border dark:border-dark-border text-transparent hover:border-emerald-600'}"
                                                >
                                                    <Check size={14} />
                                                </button>
                                                <input 
                                                    type="text" 
                                                    bind:value={quiz.options[oIndex]} 
                                                    placeholder={$t("editor.quiz_options") + " " + (oIndex + 1)}
                                                    aria-label={$t("editor.quiz_options") + " " + (oIndex + 1)}
                                                    class="flex-1 bg-muted dark:bg-white/5 border border-transparent focus:border-border dark:focus:border-dark-border rounded-xl px-4 py-2 text-sm transition-all {quiz.correct_answer === oIndex ? 'font-bold text-emerald-700 dark:text-green-400' : ''}"
                                                />
                                                {#if quiz.options.length > 2}
                                                    <button 
                                                        type="button"
                                                        onclick={() => removeOption(qIndex, oIndex)}
                                                        class="p-1.5 text-muted-foreground hover:text-red-500 opacity-0 group-hover/opt:opacity-100 transition-all"
                                                        aria-label={$t("common.delete")}
                                                    >
                                                        <X size={14} />
                                                    </button>
                                                {/if}
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {:else}
                                <div class="p-6 border-2 border-dashed border-border dark:border-dark-border rounded-2xl bg-muted/50 dark:bg-white/5 flex flex-col items-center justify-center text-center space-y-2">
                                    <FileText size={24} class="text-muted-foreground/80" />
                                    <p class="text-sm font-medium text-muted-foreground dark:text-dark-text-muted">
                                        {$t("editor.descriptive_hint")}
                                    </p>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
                
                <button
                    onclick={addEmptyQuiz}
                    class="w-full py-4 border-2 border-dashed border-border dark:border-dark-border rounded-2xl text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:border-primary hover:bg-muted dark:hover:bg-white/5 transition-all flex items-center justify-center gap-2 font-bold"
                >
                    <Plus size={20} />
                    {$t("editor.add_quiz")}
                </button>
            </div>
        {:else}
            <!-- Results View -->
            <div class="space-y-8" in:fade>
                {#if attendeeStats().length === 0}
                    <div class="flex flex-col items-center justify-center py-20 text-center space-y-4">
                        <div class="w-20 h-20 bg-accent/60 dark:bg-white/5 rounded-full flex items-center justify-center">
                            <Users2 size={40} class="text-muted-foreground/60" />
                        </div>
                        <div>
                            <h4 class="text-lg font-bold text-foreground dark:text-dark-text">{$t("editor.no_quiz_submissions")}</h4>
                            <p class="text-sm text-muted-foreground dark:text-dark-text-muted">Wait for participants to complete the quiz.</p>
                        </div>
                    </div>
                {:else}
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div class="bg-accent/70 dark:bg-primary/10 p-6 rounded-2xl border border-primary/20 dark:border-primary/20">
                            <div class="flex items-center gap-3 mb-2">
                                <Users2 size={20} class="text-primary" />
                                <span class="text-xs font-bold text-primary uppercase tracking-wider">{$t("live.participants")}</span>
                            </div>
                            <div class="text-3xl font-black text-primary dark:text-primary">{attendeeStats().length}</div>
                        </div>
                        <div class="bg-emerald-50 dark:bg-green-500/10 p-6 rounded-2xl border border-emerald-200 dark:border-green-500/20">
                            <div class="flex items-center gap-3 mb-2">
                                <Trophy size={20} class="text-emerald-700" />
                                <span class="text-xs font-bold text-emerald-700 uppercase tracking-wider">{$t("editor.score")} (Avg)</span>
                            </div>
                            <div class="text-3xl font-black text-emerald-700 dark:text-green-400">
                                {Math.round(attendeeStats().reduce((acc, s) => acc + (s.correct / s.total), 0) / attendeeStats().length * 100)}%
                            </div>
                        </div>
                        <div class="bg-muted dark:bg-white/5 p-6 rounded-2xl border border-border dark:border-dark-border">
                            <div class="flex items-center gap-3 mb-2">
                                <Clock size={20} class="text-muted-foreground" />
                                <span class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.submitted_at")} (Latest)</span>
                            </div>
                            <div class="text-lg font-bold text-foreground dark:text-dark-text truncate">
                                {new Date(attendeeStats()[0].lastSubmitted).toLocaleTimeString()}
                            </div>
                        </div>
                    </div>

                    <div class="overflow-x-auto border border-border dark:border-dark-border rounded-2xl shadow-sm">
                        <table class="w-full text-left border-collapse">
                            <thead>
                                <tr class="bg-muted dark:bg-white/5 border-b border-border dark:border-dark-border">
                                    <th class="px-6 py-4 text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">{$t("editor.attendee_name")}</th>
                                    <th class="px-6 py-4 text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">{$t("editor.score")}</th>
                                    <th class="px-6 py-4 text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">Status</th>
                                    <th class="px-6 py-4 text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">{$t("editor.submitted_at")}</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-border dark:divide-dark-border">
                                {#each attendeeStats() as stat}
                                    <tr class="hover:bg-muted dark:hover:bg-white/5 transition-colors">
                                        <td class="px-6 py-4">
                                            <div class="font-bold text-foreground dark:text-dark-text">{stat.name}</div>
                                        </td>
                                        <td class="px-6 py-4">
                                            <div class="flex items-center gap-2">
                                                <div class="w-24 h-2 bg-gray-100 dark:bg-white/10 rounded-full overflow-hidden">
                                                    <div 
                                                        class="h-full bg-emerald-600 rounded-full" 
                                                        style="width: {(stat.correct / stat.total) * 100}%"
                                                    ></div>
                                                </div>
                                                <span class="text-sm font-bold text-emerald-700 dark:text-green-400">{stat.correct}/{stat.total}</span>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4">
                                            {#if stat.correct === stat.total}
                                                <span class="bg-emerald-50 text-emerald-700 text-[10px] px-2 py-1 rounded-full font-bold">Passed</span>
                                            {:else}
                                                <span class="bg-gray-100 text-gray-500 text-[10px] px-2 py-1 rounded-full font-bold">Completed</span>
                                            {/if}
                                        </td>
                                        <td class="px-6 py-4 text-sm text-muted-foreground dark:text-dark-text-muted">
                                            {new Date(stat.lastSubmitted).toLocaleString()}
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>

                    <!-- Detailed Submission View -->
                    <div class="space-y-6">
                        <h4 class="text-lg font-bold text-foreground dark:text-dark-text flex items-center gap-2">
                            <ListChecks size={20} />
                            Detailed Responses
                        </h4>
                        <div class="space-y-4">
                            {#each attendeeStats() as stat}
                                <div class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl overflow-hidden shadow-sm">
                                    <div class="p-4 bg-muted dark:bg-white/5 border-b border-border dark:border-dark-border font-bold text-sm">
                                        {stat.name}'s Answers
                                    </div>
                                    <div class="p-6 space-y-4">
                                        {#each getAttendeeSubmissions(stat.id) as sub, idx}
                                            <div class="flex gap-4">
                                                <div class="mt-1">
                                                    {#if sub.is_correct}
                                                        <CheckCircle2 size={18} class="text-emerald-600" />
                                                    {:else}
                                                        <X size={18} class="text-red-500" />
                                                    {/if}
                                                </div>
                                                <div class="flex-1 space-y-1">
                                                    <div class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase">Q{idx+1}</div>
                                                    <div class="text-sm font-medium text-foreground dark:text-dark-text">
                                                        {sub.answer}
                                                    </div>
                                                </div>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}
            </div>
        {/if}
    </div>
    
    {#if activeTab === 'edit'}
        <div class="p-6 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface flex justify-end">
            <button
                onclick={handleQuizSave}
                disabled={isSaving}
                class="bg-emerald-700 hover:bg-emerald-800 disabled:opacity-50 text-white px-8 py-3 rounded-full font-bold shadow-lg transition-all active:scale-95 flex items-center gap-2"
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
    {/if}
</div>
