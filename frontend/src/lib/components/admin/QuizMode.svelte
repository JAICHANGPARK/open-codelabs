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
    class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
    in:fade
>
    <!-- Header with Tabs -->
    <div class="p-6 sm:p-8 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/30 dark:bg-white/5 flex flex-col sm:flex-row justify-between items-center gap-4">
        <div class="flex items-center gap-6">
            <div>
                <h3 class="text-xl font-bold text-[#202124] dark:text-dark-text">{$t("editor.quiz_tab")}</h3>
                <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">{$t("editor.quiz_settings")}</p>
            </div>
            
            <div class="flex bg-[#F1F3F4] dark:bg-white/5 p-1 rounded-xl border border-[#E8EAED] dark:border-dark-border ml-4">
                <button 
                    onclick={() => activeTab = 'edit'}
                    class="px-4 py-1.5 rounded-lg flex items-center gap-2 text-xs font-bold transition-all {activeTab === 'edit' ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]' : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124]'}"
                >
                    <Settings2 size={14} />
                    {$t("editor.quiz_edit_tab")}
                </button>
                <button 
                    onclick={() => activeTab = 'results'}
                    class="px-4 py-1.5 rounded-lg flex items-center gap-2 text-xs font-bold transition-all {activeTab === 'results' ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]' : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124]'}"
                >
                    <BarChart3 size={14} />
                    {$t("editor.quiz_results_tab")}
                    {#if quizSubmissions.length > 0}
                        <span class="bg-[#4285F4] text-white text-[10px] px-1.5 py-0.5 rounded-full">{attendeeStats().length}</span>
                    {/if}
                </button>
            </div>
        </div>

        {#if activeTab === 'edit'}
            <div class="flex items-center gap-3" in:fade>
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
        {/if}
    </div>

    <div class="p-6 sm:p-8 flex-1 overflow-y-auto">
        {#if activeTab === 'edit'}
            <div class="space-y-6" in:fade>
                {#each quizzes as quiz, qIndex}
                    <div class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-sm overflow-hidden group">
                        <div class="p-4 bg-[#F8F9FA] dark:bg-white/5 border-b border-[#F1F3F4] dark:border-dark-border flex justify-between items-center">
                            <div class="flex items-center gap-4">
                                <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-tighter">Question {qIndex + 1}</span>
                                <div class="flex bg-gray-200 dark:bg-white/10 p-0.5 rounded-lg">
                                    <button 
                                        onclick={() => quiz.quiz_type = 'multiple_choice'}
                                        class="px-2 py-1 rounded-md text-[10px] font-bold flex items-center gap-1 transition-all {quiz.quiz_type !== 'descriptive' ? 'bg-white dark:bg-dark-surface text-[#4285F4] shadow-sm' : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124]'}"
                                    >
                                        <ListChecks size={12} />
                                        {$t("editor.multiple_choice")}
                                    </button>
                                    <button 
                                        onclick={() => quiz.quiz_type = 'descriptive'}
                                        class="px-2 py-1 rounded-md text-[10px] font-bold flex items-center gap-1 transition-all {quiz.quiz_type === 'descriptive' ? 'bg-white dark:bg-dark-surface text-[#4285F4] shadow-sm' : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124]'}"
                                    >
                                        <FileText size={12} />
                                        {$t("editor.descriptive")}
                                    </button>
                                </div>
                            </div>
                            <button onclick={() => removeQuiz(qIndex)} class="p-1.5 text-[#5F6368] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg transition-all">
                                <Trash2 size={16} />
                            </button>
                        </div>
                        <div class="p-6 space-y-6">
                            <div class="space-y-2">
                                <label for="q-{qIndex}" class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.quiz_question")}</label>
                                <input id="q-{qIndex}" type="text" bind:value={quiz.question} placeholder="Enter your question here..." class="w-full text-lg font-bold outline-none bg-transparent border-b-2 border-transparent focus:border-[#4285F4] transition-all" />
                            </div>
                            
                            {#if quiz.quiz_type !== 'descriptive'}
                                <div class="space-y-3">
                                    <div class="flex justify-between items-center">
                                        <label for="opt-{qIndex}" class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.quiz_options")}</label>
                                        <button 
                                            onclick={() => addOption(qIndex)}
                                            class="text-[10px] font-bold text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 px-2 py-1 rounded-lg transition-all flex items-center gap-1"
                                        >
                                            <Plus size={12} />
                                            {$t("editor.add_option")}
                                        </button>
                                    </div>
                                    <div class="grid grid-cols-1 gap-2" id="opt-{qIndex}">
                                        {#each quiz.options as option, oIndex}
                                            <div class="flex items-center gap-3 group/opt">
                                                <button 
                                                    onclick={() => quiz.correct_answer = oIndex}
                                                    aria-label="Mark as correct answer"
                                                    class="w-6 h-6 rounded-full flex items-center justify-center border-2 transition-all {quiz.correct_answer === oIndex ? 'bg-[#34A853] border-[#34A853] text-white' : 'border-[#DADCE0] dark:border-dark-border text-transparent hover:border-[#34A853]'}"
                                                >
                                                    <Check size={14} />
                                                </button>
                                                <input 
                                                    type="text" 
                                                    bind:value={quiz.options[oIndex]} 
                                                    placeholder="Option {oIndex + 1}"
                                                    aria-label="Option {oIndex + 1}"
                                                    class="flex-1 bg-[#F8F9FA] dark:bg-white/5 border border-transparent focus:border-[#DADCE0] dark:focus:border-dark-border rounded-xl px-4 py-2 text-sm transition-all {quiz.correct_answer === oIndex ? 'font-bold text-[#137333] dark:text-green-400' : ''}"
                                                />
                                                {#if quiz.options.length > 2}
                                                    <button 
                                                        onclick={() => removeOption(qIndex, oIndex)}
                                                        class="p-1.5 text-[#5F6368] hover:text-red-500 opacity-0 group-hover/opt:opacity-100 transition-all"
                                                    >
                                                        <X size={14} />
                                                    </button>
                                                {/if}
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {:else}
                                <div class="p-6 border-2 border-dashed border-[#DADCE0] dark:border-dark-border rounded-2xl bg-[#F8F9FA]/50 dark:bg-white/5 flex flex-col items-center justify-center text-center space-y-2">
                                    <FileText size={24} class="text-[#9AA0A6]" />
                                    <p class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted">
                                        {$t("editor.descriptive_hint")}
                                    </p>
                                </div>
                            {/if}
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
        {:else}
            <!-- Results View -->
            <div class="space-y-8" in:fade>
                {#if attendeeStats().length === 0}
                    <div class="flex flex-col items-center justify-center py-20 text-center space-y-4">
                        <div class="w-20 h-20 bg-[#F1F3F4] dark:bg-white/5 rounded-full flex items-center justify-center">
                            <Users2 size={40} class="text-[#BDC1C6]" />
                        </div>
                        <div>
                            <h4 class="text-lg font-bold text-[#202124] dark:text-dark-text">{$t("editor.no_quiz_submissions")}</h4>
                            <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">Wait for participants to complete the quiz.</p>
                        </div>
                    </div>
                {:else}
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div class="bg-[#E8F0FE] dark:bg-[#4285F4]/10 p-6 rounded-2xl border border-[#D2E3FC] dark:border-[#4285F4]/20">
                            <div class="flex items-center gap-3 mb-2">
                                <Users2 size={20} class="text-[#4285F4]" />
                                <span class="text-xs font-bold text-[#4285F4] uppercase tracking-wider">{$t("live.participants")}</span>
                            </div>
                            <div class="text-3xl font-black text-[#1967D2] dark:text-[#4285F4]">{attendeeStats().length}</div>
                        </div>
                        <div class="bg-[#E6F4EA] dark:bg-green-500/10 p-6 rounded-2xl border border-[#CEEAD6] dark:border-green-500/20">
                            <div class="flex items-center gap-3 mb-2">
                                <Trophy size={20} class="text-[#1E8E3E]" />
                                <span class="text-xs font-bold text-[#1E8E3E] uppercase tracking-wider">{$t("editor.score")} (Avg)</span>
                            </div>
                            <div class="text-3xl font-black text-[#137333] dark:text-green-400">
                                {Math.round(attendeeStats().reduce((acc, s) => acc + (s.correct / s.total), 0) / attendeeStats().length * 100)}%
                            </div>
                        </div>
                        <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-2xl border border-[#E8EAED] dark:border-dark-border">
                            <div class="flex items-center gap-3 mb-2">
                                <Clock size={20} class="text-[#5F6368]" />
                                <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">{$t("editor.submitted_at")} (Latest)</span>
                            </div>
                            <div class="text-lg font-bold text-[#202124] dark:text-dark-text truncate">
                                {new Date(attendeeStats()[0].lastSubmitted).toLocaleTimeString()}
                            </div>
                        </div>
                    </div>

                    <div class="overflow-x-auto border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-sm">
                        <table class="w-full text-left border-collapse">
                            <thead>
                                <tr class="bg-[#F8F9FA] dark:bg-white/5 border-b border-[#E8EAED] dark:border-dark-border">
                                    <th class="px-6 py-4 text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase">{$t("editor.attendee_name")}</th>
                                    <th class="px-6 py-4 text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase">{$t("editor.score")}</th>
                                    <th class="px-6 py-4 text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase">Status</th>
                                    <th class="px-6 py-4 text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase">{$t("editor.submitted_at")}</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-[#F1F3F4] dark:divide-dark-border">
                                {#each attendeeStats() as stat}
                                    <tr class="hover:bg-[#F8F9FA] dark:hover:bg-white/5 transition-colors">
                                        <td class="px-6 py-4">
                                            <div class="font-bold text-[#202124] dark:text-dark-text">{stat.name}</div>
                                        </td>
                                        <td class="px-6 py-4">
                                            <div class="flex items-center gap-2">
                                                <div class="w-24 h-2 bg-gray-100 dark:bg-white/10 rounded-full overflow-hidden">
                                                    <div 
                                                        class="h-full bg-[#34A853] rounded-full" 
                                                        style="width: {(stat.correct / stat.total) * 100}%"
                                                    ></div>
                                                </div>
                                                <span class="text-sm font-bold text-[#137333] dark:text-green-400">{stat.correct}/{stat.total}</span>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4">
                                            {#if stat.correct === stat.total}
                                                <span class="bg-[#E6F4EA] text-[#1E8E3E] text-[10px] px-2 py-1 rounded-full font-bold">Passed</span>
                                            {:else}
                                                <span class="bg-gray-100 text-gray-500 text-[10px] px-2 py-1 rounded-full font-bold">Completed</span>
                                            {/if}
                                        </td>
                                        <td class="px-6 py-4 text-sm text-[#5F6368] dark:text-dark-text-muted">
                                            {new Date(stat.lastSubmitted).toLocaleString()}
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>

                    <!-- Detailed Submission View -->
                    <div class="space-y-6">
                        <h4 class="text-lg font-bold text-[#202124] dark:text-dark-text flex items-center gap-2">
                            <ListChecks size={20} />
                            Detailed Responses
                        </h4>
                        <div class="space-y-4">
                            {#each attendeeStats() as stat}
                                <div class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-hidden shadow-sm">
                                    <div class="p-4 bg-[#F8F9FA] dark:bg-white/5 border-b border-[#F1F3F4] dark:border-dark-border font-bold text-sm">
                                        {stat.name}'s Answers
                                    </div>
                                    <div class="p-6 space-y-4">
                                        {#each getAttendeeSubmissions(stat.id) as sub, idx}
                                            <div class="flex gap-4">
                                                <div class="mt-1">
                                                    {#if sub.is_correct}
                                                        <CheckCircle2 size={18} class="text-[#34A853]" />
                                                    {:else}
                                                        <X size={18} class="text-[#EA4335]" />
                                                    {/if}
                                                </div>
                                                <div class="flex-1 space-y-1">
                                                    <div class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase">Q{idx+1}</div>
                                                    <div class="text-sm font-medium text-[#202124] dark:text-dark-text">
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
            </button>
        </div>
    {/if}
</div>
