<script lang="ts">
    import { onMount } from "svelte";
    import { getAuditLogs, type AuditLog } from "$lib/api-backend";
    import { goto } from "$app/navigation";
    import { t } from "svelte-i18n";

    let logs = $state<AuditLog[]>([]);
    let loading = $state(true);
    let error = $state("");
    let limit = $state(100);
    let offset = $state(0);
    let filterAction = $state("");
    let filterCodelabId = $state("");

    onMount(async () => {
        await loadLogs();
    });

    async function loadLogs() {
        try {
            loading = true;
            error = "";
            logs = await getAuditLogs({
                limit,
                offset,
                action: filterAction || undefined,
                codelab_id: filterCodelabId || undefined,
            });
        } catch (e) {
            error = $t("audit.error_load", {
                values: { error: (e as Error).message },
            });
        } finally {
            loading = false;
        }
    }

    function handleFilter() {
        offset = 0;
        loadLogs();
    }

    function handleNextPage() {
        offset += limit;
        loadLogs();
    }

    function handlePrevPage() {
        offset = Math.max(0, offset - limit);
        loadLogs();
    }

    function formatDate(dateStr: string): string {
        const date = new Date(dateStr);
        return date.toLocaleString();
    }

    function formatMetadata(metadata?: string): string {
        if (!metadata) return "-";
        try {
            return JSON.stringify(JSON.parse(metadata), null, 2);
        } catch {
            return metadata;
        }
    }
</script>

<div class="min-h-screen bg-gray-50 dark:bg-dark-bg">
    <div class="max-w-7xl mx-auto px-4 py-8">
        <div class="flex justify-between items-center mb-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-dark-text">
                {$t("audit.title")}
            </h1>
            <button
                onclick={() => goto("/admin")}
                class="px-4 py-2 text-sm bg-white dark:bg-dark-surface border border-gray-300 dark:border-dark-border rounded-lg hover:bg-gray-50 dark:hover:bg-dark-hover transition-colors"
            >
                {$t("audit.back_to_admin")}
            </button>
        </div>

        <!-- Filters -->
        <div class="bg-white dark:bg-dark-surface rounded-lg shadow p-4 mb-6">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                    <label
                        for="action"
                        class="block text-sm font-medium text-gray-700 dark:text-dark-text mb-1"
                    >
                        {$t("audit.filter_action")}
                    </label>
                    <input
                        id="action"
                        type="text"
                        bind:value={filterAction}
                        placeholder={$t("audit.placeholder_action")}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-dark-bg dark:text-dark-text"
                    />
                </div>
                <div>
                    <label
                        for="codelabId"
                        class="block text-sm font-medium text-gray-700 dark:text-dark-text mb-1"
                    >
                        {$t("audit.filter_codelab_id")}
                    </label>
                    <input
                        id="codelabId"
                        type="text"
                        bind:value={filterCodelabId}
                        placeholder={$t("audit.placeholder_codelab")}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-dark-border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-dark-bg dark:text-dark-text"
                    />
                </div>
                <div class="flex items-end">
                    <button
                        onclick={handleFilter}
                        class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                        {$t("audit.apply_filters")}
                    </button>
                </div>
            </div>
        </div>

        {#if error}
            <div
                class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6"
            >
                <p class="text-red-800 dark:text-red-200">{error}</p>
            </div>
        {/if}

        {#if loading}
            <div class="text-center py-12">
                <p class="text-gray-600 dark:text-dark-text-muted">
                    {$t("audit.loading")}
                </p>
            </div>
        {:else if logs.length === 0}
            <div
                class="bg-white dark:bg-dark-surface rounded-lg shadow p-8 text-center"
            >
                <p class="text-gray-600 dark:text-dark-text-muted">
                    {$t("audit.no_logs")}
                </p>
            </div>
        {:else}
            <div
                class="bg-white dark:bg-dark-surface rounded-lg shadow overflow-hidden"
            >
                <div class="overflow-x-auto">
                    <table class="w-full">
                        <thead class="bg-gray-50 dark:bg-dark-hover">
                            <tr>
                                <th
                                    class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("audit.col_time")}
                                </th>
                                <th
                                    class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("audit.col_action")}
                                </th>
                                <th
                                    class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("audit.col_actor")}
                                </th>
                                <th
                                    class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("audit.col_target")}
                                </th>
                                <th
                                    class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("audit.col_ip")}
                                </th>
                                <th
                                    class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("audit.col_details")}
                                </th>
                            </tr>
                        </thead>
                        <tbody
                            class="divide-y divide-gray-200 dark:divide-dark-border"
                        >
                            {#each logs as log (log.id)}
                                <tr
                                    class="hover:bg-gray-50 dark:hover:bg-dark-hover"
                                >
                                    <td
                                        class="px-4 py-3 text-sm text-gray-900 dark:text-dark-text whitespace-nowrap"
                                    >
                                        {formatDate(log.created_at)}
                                    </td>
                                    <td
                                        class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-dark-text"
                                    >
                                        <span
                                            class="px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200 rounded text-xs"
                                        >
                                            {log.action}
                                        </span>
                                    </td>
                                    <td
                                        class="px-4 py-3 text-sm text-gray-600 dark:text-dark-text-muted"
                                    >
                                        <div>{log.actor_type}</div>
                                        {#if log.actor_id}
                                            <div
                                                class="text-xs text-gray-500 dark:text-dark-text-muted"
                                            >
                                                {log.actor_id}
                                            </div>
                                        {/if}
                                    </td>
                                    <td
                                        class="px-4 py-3 text-sm text-gray-600 dark:text-dark-text-muted"
                                    >
                                        {#if log.codelab_id}
                                            <div class="text-xs">
                                                {$t(
                                                    "audit.col_target_codelab",
                                                )}: {log.codelab_id}
                                            </div>
                                        {/if}
                                        {#if log.target_id && log.target_id !== log.codelab_id}
                                            <div class="text-xs">
                                                {$t("audit.col_target_id")}: {log.target_id}
                                            </div>
                                        {/if}
                                        {#if !log.codelab_id && !log.target_id}
                                            <span class="text-gray-400">-</span>
                                        {/if}
                                    </td>
                                    <td
                                        class="px-4 py-3 text-sm text-gray-600 dark:text-dark-text-muted font-mono"
                                    >
                                        {log.ip || "-"}
                                    </td>
                                    <td class="px-4 py-3 text-sm">
                                        {#if log.metadata}
                                            <details class="cursor-pointer">
                                                <summary
                                                    class="text-blue-600 dark:text-blue-400 hover:underline"
                                                    >{$t(
                                                        "audit.view_details",
                                                    )}</summary
                                                >
                                                <pre
                                                    class="mt-2 p-2 bg-gray-100 dark:bg-dark-bg rounded text-xs overflow-x-auto">{formatMetadata(
                                                        log.metadata,
                                                    )}</pre>
                                            </details>
                                        {:else}
                                            <span class="text-gray-400">-</span>
                                        {/if}
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>

                <!-- Pagination -->
                <div
                    class="bg-gray-50 dark:bg-dark-hover px-4 py-3 flex items-center justify-between border-t border-gray-200 dark:border-dark-border"
                >
                    <div class="text-sm text-gray-700 dark:text-dark-text">
                        {$t("audit.pagination_info", {
                            values: {
                                start: offset + 1,
                                end: offset + logs.length,
                                limit,
                            },
                        })}
                    </div>
                    <div class="flex gap-2">
                        <button
                            onclick={handlePrevPage}
                            disabled={offset === 0}
                            class="px-3 py-1 text-sm bg-white dark:bg-dark-surface border border-gray-300 dark:border-dark-border rounded hover:bg-gray-50 dark:hover:bg-dark-bg disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            {$t("audit.prev")}
                        </button>
                        <button
                            onclick={handleNextPage}
                            disabled={logs.length < limit}
                            class="px-3 py-1 text-sm bg-white dark:bg-dark-surface border border-gray-300 dark:border-dark-border rounded hover:bg-gray-50 dark:hover:bg-dark-bg disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            {$t("audit.next")}
                        </button>
                    </div>
                </div>
            </div>
        {/if}
    </div>
</div>
