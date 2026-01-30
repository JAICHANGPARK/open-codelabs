<script lang="ts">
    import { onMount } from "svelte";
    import { getAiConversations, type AiConversation } from "$lib/api";
    import { MessageSquare, User, Clock, Hash } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let { codelabId } = $props<{ codelabId: string }>();

    let conversations = $state<AiConversation[]>([]);
    let loading = $state(true);
    let error = $state("");
    let selectedConversation = $state<AiConversation | null>(null);

    onMount(async () => {
        await loadConversations();
    });

    async function loadConversations() {
        try {
            loading = true;
            conversations = await getAiConversations(codelabId);
        } catch (e) {
            error = e instanceof Error ? e.message : String(e);
            console.error("Failed to load AI conversations:", e);
        } finally {
            loading = false;
        }
    }

    function formatDate(dateStr?: string) {
        if (!dateStr) return "";
        return new Date(dateStr).toLocaleString();
    }
</script>

<div class="h-full flex flex-col bg-white dark:bg-gray-900">
    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <div class="p-2 bg-purple-100 dark:bg-purple-900/30 rounded-lg">
                    <MessageSquare size={24} class="text-purple-600 dark:text-purple-400" />
                </div>
                <div>
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white">
                        AI {$t("common.conversations")}
                    </h2>
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                        {conversations.length} {$t("common.total")}
                    </p>
                </div>
            </div>
            <button
                onclick={loadConversations}
                class="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors"
            >
                {$t("common.refresh")}
            </button>
        </div>
    </div>

    <div class="flex-1 overflow-hidden flex">
        <!-- List -->
        <div class="w-1/3 border-r border-gray-200 dark:border-gray-700 overflow-y-auto">
            {#if loading}
                <div class="p-8 text-center text-gray-500 dark:text-gray-400">
                    {$t("common.loading")}...
                </div>
            {:else if error}
                <div class="p-8 text-center text-red-600 dark:text-red-400">
                    {error}
                </div>
            {:else if conversations.length === 0}
                <div class="p-8 text-center text-gray-500 dark:text-gray-400">
                    {$t("common.no_data")}
                </div>
            {:else}
                {#each conversations as conv}
                    <button
                        onclick={() => selectedConversation = conv}
                        class="w-full p-4 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-left {selectedConversation?.id === conv.id ? 'bg-purple-50 dark:bg-purple-900/20' : ''}"
                    >
                        <div class="flex items-start gap-3">
                            <div class="flex-shrink-0 p-2 bg-gray-100 dark:bg-gray-800 rounded-full">
                                <User size={16} class="text-gray-600 dark:text-gray-400" />
                            </div>
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-2 mb-1">
                                    <span class="font-semibold text-gray-900 dark:text-white">
                                        {conv.user_name}
                                    </span>
                                    <span class="px-2 py-0.5 text-xs rounded-full {conv.user_type === 'admin' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400' : 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'}">
                                        {conv.user_type}
                                    </span>
                                </div>
                                <p class="text-sm text-gray-600 dark:text-gray-400 truncate">
                                    {conv.question.substring(0, 80)}...
                                </p>
                                <div class="flex items-center gap-3 mt-2 text-xs text-gray-500 dark:text-gray-500">
                                    {#if conv.step_number}
                                        <div class="flex items-center gap-1">
                                            <Hash size={12} />
                                            Step {conv.step_number}
                                        </div>
                                    {/if}
                                    <div class="flex items-center gap-1">
                                        <Clock size={12} />
                                        {formatDate(conv.created_at)}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </button>
                {/each}
            {/if}
        </div>

        <!-- Detail -->
        <div class="flex-1 overflow-y-auto p-6">
            {#if selectedConversation}
                <div class="max-w-4xl mx-auto space-y-6">
                    <!-- Header -->
                    <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
                        <div class="flex items-center justify-between mb-2">
                            <div class="flex items-center gap-3">
                                <div class="p-2 bg-white dark:bg-gray-700 rounded-full">
                                    <User size={20} class="text-gray-600 dark:text-gray-400" />
                                </div>
                                <div>
                                    <div class="flex items-center gap-2">
                                        <span class="font-bold text-gray-900 dark:text-white">
                                            {selectedConversation.user_name}
                                        </span>
                                        <span class="px-2 py-0.5 text-xs rounded-full {selectedConversation.user_type === 'admin' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400' : 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'}">
                                            {selectedConversation.user_type}
                                        </span>
                                    </div>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">
                                        {formatDate(selectedConversation.created_at)}
                                    </p>
                                </div>
                            </div>
                            {#if selectedConversation.step_number}
                                <div class="flex items-center gap-2 px-3 py-1 bg-white dark:bg-gray-700 rounded-full">
                                    <Hash size={16} class="text-gray-600 dark:text-gray-400" />
                                    <span class="text-sm font-medium text-gray-900 dark:text-white">
                                        Step {selectedConversation.step_number}
                                    </span>
                                </div>
                            {/if}
                        </div>
                        {#if selectedConversation.model}
                            <div class="text-xs text-gray-500 dark:text-gray-500">
                                Model: {selectedConversation.model}
                            </div>
                        {/if}
                    </div>

                    <!-- Question -->
                    <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-6">
                        <div class="text-sm font-semibold text-blue-900 dark:text-blue-300 mb-2">
                            {$t("common.question")}
                        </div>
                        <div class="text-gray-900 dark:text-white whitespace-pre-wrap">
                            {selectedConversation.question}
                        </div>
                    </div>

                    <!-- Answer -->
                    <div class="bg-green-50 dark:bg-green-900/20 rounded-lg p-6">
                        <div class="text-sm font-semibold text-green-900 dark:text-green-300 mb-2">
                            {$t("common.answer")}
                        </div>
                        <div class="text-gray-900 dark:text-white whitespace-pre-wrap prose prose-sm dark:prose-invert max-w-none">
                            {selectedConversation.answer}
                        </div>
                    </div>
                </div>
            {:else}
                <div class="h-full flex items-center justify-center text-gray-500 dark:text-gray-400">
                    {$t("common.select_item")}
                </div>
            {/if}
        </div>
    </div>
</div>
