<script lang="ts">
    import { MessageSquare } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { t } from "svelte-i18n";
    import type { Feedback } from "$lib/api";

    let { feedbacks } = $props<{ feedbacks: Feedback[] }>();

    let avgSatisfaction = $derived(
        feedbacks.length > 0
            ? (
                  feedbacks.reduce(
                      (acc: number, f: Feedback) => acc + parseInt(f.satisfaction),
                      0,
                  ) / feedbacks.length
              ).toFixed(1)
            : "N/A",
    );

    let avgDifficulty = $derived(
        feedbacks.length > 0
            ? (
                  feedbacks.reduce(
                      (acc: number, f: Feedback) => acc + parseInt(f.difficulty),
                      0,
                  ) / feedbacks.length
              ).toFixed(1)
            : "N/A",
    );
</script>

<div
    class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
    in:fade
>
    <div
        class="p-6 sm:p-8 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/30 dark:bg-white/5 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 sm:gap-8"
    >
        <div
            class="bg-white dark:bg-dark-surface p-4 rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm"
        >
            <p class="text-xs text-[#5F6368] dark:text-dark-text-muted font-bold uppercase tracking-wider mb-2">
                {$t("editor.avg_satisfaction")}
            </p>
            <div
                class="text-3xl font-bold text-[#1E8E3E]"
            >
                {avgSatisfaction}<span
                    class="text-base text-[#5F6368] dark:text-dark-text-muted font-normal"
                    >/5</span
                >
            </div>
        </div>
        <div
            class="bg-white dark:bg-dark-surface p-4 rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm"
        >
            <p class="text-xs text-[#5F6368] dark:text-dark-text-muted font-bold uppercase tracking-wider mb-2">
                {$t("editor.avg_difficulty")}
            </p>
            <div
                class="text-3xl font-bold text-[#F9AB00]"
            >
                {avgDifficulty}<span
                    class="text-base text-[#5F6368] dark:text-dark-text-muted font-normal"
                    >/5</span
                >
            </div>
        </div>
        <div
            class="bg-white dark:bg-dark-surface p-4 rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm sm:col-span-2 md:col-span-1"
        >
            <p class="text-xs text-[#5F6368] dark:text-dark-text-muted font-bold uppercase tracking-wider mb-2">
                {$t("editor.total_responses")}
            </p>
            <div
                class="text-3xl font-bold text-[#4285F4]"
            >
                {feedbacks.length}
            </div>
        </div>
    </div>

    <div
        class="flex-1 p-4 sm:p-8 overflow-y-auto space-y-4"
    >
        {#each feedbacks as f}
            <div
                class="p-6 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl shadow-sm hover:shadow-md transition-shadow"
            >
                <div
                    class="flex flex-col sm:flex-row justify-between items-start gap-4 mb-4"
                >
                    <div class="flex flex-wrap gap-3">
                        <div
                            class="bg-[#E6F4EA] dark:bg-green-500/10 text-[#137333] dark:text-green-400 px-3 py-1 rounded-full text-xs font-bold"
                        >
                            {$t("feedback.satisfaction")}: {f.satisfaction}/5
                        </div>
                        <div
                            class="bg-[#FEF7E0] dark:bg-yellow-500/10 text-[#B06000] dark:text-yellow-400 px-3 py-1 rounded-full text-xs font-bold"
                        >
                            {$t("feedback.difficulty")}: {f.difficulty}/5
                        </div>
                    </div>
                    <span
                        class="text-xs text-[#5F6368] dark:text-dark-text-muted"
                    >
                        {f.created_at
                            ? new Date(
                                  f.created_at,
                              ).toLocaleString()
                            : ""}
                    </span>
                </div>
                {#if f.comment}
                    <p
                        class="text-[#3C4043] dark:text-dark-text text-sm leading-relaxed bg-[#F8F9FA] dark:bg-white/5 p-4 rounded-lg border border-transparent dark:border-dark-border"
                    >
                        "{f.comment}"
                    </p>
                {:else}
                    <p class="text-[#9AA0A6] dark:text-dark-text-muted text-sm italic">
                        {$t("editor.no_comments")}
                    </p>
                {/if}
            </div>
        {:else}
            <div
                class="text-center py-20 text-[#5F6368] dark:text-dark-text-muted"
            >
                <MessageSquare
                    size={48}
                    class="mx-auto mb-4 opacity-20"
                />
                <p class="text-lg font-medium dark:text-dark-text">
                    {$t("editor.no_feedback_yet")}
                </p>
                <p class="text-sm opacity-70">
                    {$t("editor.wait_for_feedback")}
                </p>
            </div>
        {/each}
    </div>
</div>
