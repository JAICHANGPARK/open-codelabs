<script lang="ts">
    import { FileUp, FolderGit2, Download, AlertCircle, CheckCircle2, Loader2 } from 'lucide-svelte';
    import { getCodeServerInfo, createCodeServer, createCodeServerBranch, createCodeServerFolder, downloadCodeServerWorkspace, type WorkspaceFile } from '$lib/api-backend';
    import type { Step } from '$lib/api';
    import { fade } from 'svelte/transition';
    import { t } from 'svelte-i18n';

    let { codelabId, steps }: { codelabId: string; steps: Step[] } = $props();

    let workspaceExists = $state(false);
    let workspaceInfo = $state<{ path: string; structure_type: string } | null>(null);
    let loading = $state(true);
    let creating = $state(false);
    let error = $state('');
    let success = $state('');

    // Workspace creation options
    let structureType = $state<'branch' | 'folder'>('branch');
    let uploadedFiles = $state<{ name: string; content: string }[]>([]);
    let fileInput: HTMLInputElement;
    let initialized = $state(false);

    $effect(() => {
        if (!initialized) {
            initialized = true;
            loadWorkspaceInfo();
        }
    });

    async function loadWorkspaceInfo() {
        try {
            loading = true;
            error = '';
            const info = await getCodeServerInfo(codelabId);
            workspaceInfo = info;
            workspaceExists = true;
        } catch (e) {
            // Workspace doesn't exist yet - this is normal
            workspaceExists = false;
            workspaceInfo = null;
        } finally {
            loading = false;
        }
    }

    async function handleFileUpload(event: Event) {
        const target = event.target as HTMLInputElement;
        if (!target.files) return;

        const newFiles: { name: string; content: string }[] = [];

        for (const file of Array.from(target.files)) {
            const content = await file.text();
            newFiles.push({
                name: file.name,
                content
            });
        }

        uploadedFiles = [...uploadedFiles, ...newFiles];
        target.value = '';
    }

    function removeFile(index: number) {
        uploadedFiles = uploadedFiles.filter((_, i) => i !== index);
    }

    async function createWorkspace() {
        if (uploadedFiles.length === 0) {
            error = $t('workspace.errors.upload_required');
            return;
        }

        if (steps.length === 0) {
            error = $t('workspace.errors.steps_required');
            return;
        }

        try {
            creating = true;
            error = '';
            success = '';

            const workspaceFiles: WorkspaceFile[] = uploadedFiles.map(f => ({
                path: f.name,
                content: f.content
            }));

            // Create workspace
            await createCodeServer(codelabId, workspaceFiles, structureType);

            // Create step branches/folders
            if (structureType === 'branch') {
                for (let i = 0; i < steps.length; i++) {
                    await createCodeServerBranch(codelabId, i + 1, 'start');
                    await createCodeServerBranch(codelabId, i + 1, 'end');
                }
            } else {
                for (let i = 0; i < steps.length; i++) {
                    await createCodeServerFolder(codelabId, i + 1, 'start', workspaceFiles);
                    await createCodeServerFolder(codelabId, i + 1, 'end', workspaceFiles);
                }
            }

            success = $t('workspace.success.created');
            await loadWorkspaceInfo();
        } catch (e) {
            error = $t('workspace.errors.create_failed', { error: (e as Error).message });
        } finally {
            creating = false;
        }
    }

    async function handleDownload() {
        try {
            error = '';
            await downloadCodeServerWorkspace(codelabId);
        } catch (e) {
            error = $t('workspace.errors.download_failed', { error: (e as Error).message });
        }
    }
</script>

<div
    class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
    in:fade
>
    <div class="p-6 sm:p-8 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/30 dark:bg-white/5 flex flex-col sm:flex-row justify-between items-center gap-4">
        <div class="flex items-center gap-3">
            <div class="p-2 bg-[#4285F4]/10 rounded-lg text-[#4285F4]">
                <FolderGit2 size={24} />
            </div>
            <div>
                <h3 class="text-xl font-bold text-[#202124] dark:text-dark-text">{$t('workspace.title')}</h3>
                <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">{$t('workspace.subtitle')}</p>
            </div>
        </div>
    </div>

    <div class="p-6 sm:p-8 flex-1 overflow-y-auto">
        <div class="max-w-3xl space-y-6">
            {#if error}
                <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex items-start gap-3">
                    <AlertCircle class="text-red-600 dark:text-red-400 shrink-0 mt-0.5" size={20} />
                    <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
                </div>
            {/if}

            {#if success}
                <div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg flex items-start gap-3">
                    <CheckCircle2 class="text-green-600 dark:text-green-400 shrink-0 mt-0.5" size={20} />
                    <p class="text-green-800 dark:text-green-200 text-sm">{success}</p>
                </div>
            {/if}

            {#if loading}
                <div class="flex items-center justify-center py-12">
                    <Loader2 class="animate-spin text-[#4285F4]" size={32} />
                </div>
            {:else if workspaceExists && workspaceInfo}
                <!-- Existing Workspace -->
                <div class="bg-[#F8F9FA] dark:bg-white/5 border border-[#E8EAED] dark:border-dark-border rounded-xl p-6">
                    <div class="flex items-start justify-between mb-4">
                        <div>
                            <h3 class="text-lg font-bold text-[#202124] dark:text-dark-text mb-3">
                                {$t('workspace.active_title')}
                            </h3>
                            <div class="space-y-2 text-sm">
                                <p class="text-[#5F6368] dark:text-dark-text-muted">
                                    <span class="font-medium text-[#202124] dark:text-dark-text">{$t('workspace.labels.structure')}:</span>
                                    <span class="capitalize ml-2">{workspaceInfo.structure_type === 'branch' ? $t('workspace.labels.branch') : $t('workspace.labels.folder')}</span>
                                </p>
                                <p class="text-[#5F6368] dark:text-dark-text-muted">
                                    <span class="font-medium text-[#202124] dark:text-dark-text">{$t('workspace.labels.path')}:</span>
                                    <code class="ml-2 bg-white dark:bg-dark-bg px-2 py-0.5 rounded text-xs font-mono">{workspaceInfo.path}</code>
                                </p>
                                <p class="text-[#5F6368] dark:text-dark-text-muted">
                                    <span class="font-medium text-[#202124] dark:text-dark-text">{$t('workspace.labels.steps')}:</span>
                                    <span class="ml-2">{$t('workspace.steps_summary', {
                                        count: steps.length,
                                        unit: workspaceInfo.structure_type === 'branch' ? $t('workspace.labels.branches') : $t('workspace.labels.folders'),
                                        total: steps.length * 2
                                    })}</span>
                                </p>
                            </div>
                        </div>
                        <CheckCircle2 class="text-[#34A853]" size={24} />
                    </div>

                    <div class="pt-4 border-t border-[#E8EAED] dark:border-dark-border">
                        <button
                            onclick={handleDownload}
                            class="flex items-center gap-2 px-4 py-2 bg-[#4285F4] text-white rounded-lg hover:bg-[#1A73E8] transition-colors text-sm font-medium shadow-sm"
                        >
                            <Download size={16} />
                            {$t('workspace.download')}
                        </button>
                    </div>
                </div>
            {:else}
                <!-- Create New Workspace -->
                <div class="space-y-6">
                    <!-- Structure Type Selection -->
                    <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-xl border border-[#E8EAED] dark:border-dark-border">
                        <label class="block text-sm font-bold text-[#202124] dark:text-dark-text mb-3">
                            {$t('workspace.structure_title')}
                        </label>
                        <div class="flex gap-4">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="radio"
                                    bind:group={structureType}
                                    value="branch"
                                    class="w-4 h-4 text-[#4285F4]"
                                />
                                <span class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                                    {$t('workspace.structure_options.branch')}
                                </span>
                            </label>
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="radio"
                                    bind:group={structureType}
                                    value="folder"
                                    class="w-4 h-4 text-[#4285F4]"
                                />
                                <span class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                                    {$t('workspace.structure_options.folder')}
                                </span>
                            </label>
                        </div>
                    </div>

                    <!-- File Upload -->
                    <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-xl border border-[#E8EAED] dark:border-dark-border">
                        <label class="block text-sm font-bold text-[#202124] dark:text-dark-text mb-3">
                            {$t('workspace.base_files')}
                        </label>
                        <p class="text-xs text-[#5F6368] dark:text-dark-text-muted mb-4">
                            {$t('workspace.base_files_desc')}
                        </p>

                        <input
                            bind:this={fileInput}
                            type="file"
                            multiple
                            onchange={handleFileUpload}
                            class="hidden"
                        />

                        <button
                            onclick={() => fileInput?.click()}
                            class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-dark-surface border border-[#DADCE0] dark:border-dark-border rounded-lg hover:bg-[#F8F9FA] dark:hover:bg-dark-hover transition-colors text-sm font-medium text-[#5F6368] dark:text-dark-text"
                        >
                            <FileUp size={16} />
                            {$t('workspace.upload_files')}
                        </button>

                        {#if uploadedFiles.length > 0}
                            <div class="mt-4 space-y-2">
                                {#each uploadedFiles as file, i}
                                    <div class="flex items-center justify-between p-3 bg-white dark:bg-dark-surface rounded-lg border border-[#E8EAED] dark:border-dark-border">
                                        <span class="text-sm font-mono text-[#202124] dark:text-dark-text">
                                            {file.name}
                                        </span>
                                        <button
                                            onclick={() => removeFile(i)}
                                            class="text-[#EA4335] hover:text-[#D93025] text-xs font-medium"
                                        >
                                            {$t('workspace.remove_file')}
                                        </button>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>

                    <!-- Info Box -->
                    <div class="p-4 bg-[#E8F0FE] dark:bg-[#4285F4]/10 border border-[#4285F4]/20 rounded-lg">
                        <div class="flex items-start gap-3">
                            <FolderGit2 class="text-[#4285F4] shrink-0 mt-0.5" size={20} />
                            <div class="text-sm text-[#1967D2] dark:text-[#8AB4F8]">
                                <p class="font-medium mb-2">{$t('workspace.info_title')}</p>
                                <ul class="list-disc list-inside space-y-1 text-xs">
                                    <li>{$t('workspace.info_step_create', { unit: structureType === 'branch' ? $t('workspace.labels.branches') : $t('workspace.labels.folders') })} <code class="bg-white/50 px-1 rounded">step-N-start</code>, <code class="bg-white/50 px-1 rounded">step-N-end</code></li>
                                    <li>{$t('workspace.info_copy_base', { unit: structureType === 'branch' ? $t('workspace.labels.branch') : $t('workspace.labels.folder') })}</li>
                                    <li>{$t('workspace.info_modify_later')}</li>
                                </ul>
                            </div>
                        </div>
                    </div>

                    <!-- Create Button -->
                    <button
                        onclick={createWorkspace}
                        disabled={creating || uploadedFiles.length === 0 || steps.length === 0}
                        class="flex items-center gap-2 px-6 py-3 bg-[#4285F4] text-white rounded-lg hover:bg-[#1A73E8] transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium shadow-sm"
                    >
                        {#if creating}
                            <Loader2 class="animate-spin" size={16} />
                            {$t('workspace.creating_button')}
                        {:else}
                            <FolderGit2 size={16} />
                            {$t('workspace.create_button')}
                        {/if}
                    </button>
                </div>
            {/if}
        </div>
    </div>
</div>
