<script lang="ts">
    import { FileUp, FolderGit2, Download, AlertCircle, CheckCircle2, Loader2 } from 'lucide-svelte';
    import { getCodeServerInfo, createCodeServer, createCodeServerBranch, createCodeServerFolder, downloadCodeServerWorkspace, type WorkspaceFile } from '$lib/api-backend';
    import type { Step } from '$lib/api';
    import { fade } from 'svelte/transition';

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

    $effect(() => {
        loadWorkspaceInfo();
    });

    async function loadWorkspaceInfo() {
        try {
            loading = true;
            error = '';
            const info = await getCodeServerInfo(codelabId);
            workspaceInfo = info;
            workspaceExists = true;
        } catch (e) {
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
            error = 'Please upload at least one file for the workspace';
            return;
        }

        if (steps.length === 0) {
            error = 'Please add steps to the codelab before creating a workspace';
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

            success = 'Workspace created successfully!';
            await loadWorkspaceInfo();
        } catch (e) {
            error = 'Failed to create workspace: ' + (e as Error).message;
        } finally {
            creating = false;
        }
    }

    async function handleDownload() {
        try {
            error = '';
            await downloadCodeServerWorkspace(codelabId);
        } catch (e) {
            error = 'Download failed: ' + (e as Error).message;
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
                <h3 class="text-xl font-bold text-[#202124] dark:text-dark-text">Workspace Setup</h3>
                <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">Create and manage workspace files for hands-on coding exercises</p>
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
                                Workspace Active
                            </h3>
                            <div class="space-y-2 text-sm">
                                <p class="text-[#5F6368] dark:text-dark-text-muted">
                                    <span class="font-medium text-[#202124] dark:text-dark-text">Structure:</span>
                                    <span class="capitalize ml-2">{workspaceInfo.structure_type}</span>
                                </p>
                                <p class="text-[#5F6368] dark:text-dark-text-muted">
                                    <span class="font-medium text-[#202124] dark:text-dark-text">Path:</span>
                                    <code class="ml-2 bg-white dark:bg-dark-bg px-2 py-0.5 rounded text-xs font-mono">{workspaceInfo.path}</code>
                                </p>
                                <p class="text-[#5F6368] dark:text-dark-text-muted">
                                    <span class="font-medium text-[#202124] dark:text-dark-text">Steps:</span>
                                    <span class="ml-2">{steps.length} steps × 2 ({workspaceInfo.structure_type === 'branch' ? 'branches' : 'folders'}) = {steps.length * 2} total</span>
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
                            Download Workspace
                        </button>
                    </div>
                </div>
            {:else}
                <!-- Create New Workspace -->
                <div class="space-y-6">
                    <!-- Structure Type Selection -->
                    <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-xl border border-[#E8EAED] dark:border-dark-border">
                        <label class="block text-sm font-bold text-[#202124] dark:text-dark-text mb-3">
                            Workspace Structure
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
                                    Branch-based (Git branches)
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
                                    Folder-based (Directories)
                                </span>
                            </label>
                        </div>
                    </div>

                    <!-- File Upload -->
                    <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-xl border border-[#E8EAED] dark:border-dark-border">
                        <label class="block text-sm font-bold text-[#202124] dark:text-dark-text mb-3">
                            Base Files
                        </label>
                        <p class="text-xs text-[#5F6368] dark:text-dark-text-muted mb-4">
                            Upload the initial files that will be copied to all step branches/folders
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
                            Upload Files
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
                                            Remove
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
                                <p class="font-medium mb-2">What happens when you create a workspace?</p>
                                <ul class="list-disc list-inside space-y-1 text-xs">
                                    <li>For each step, two {structureType === 'branch' ? 'branches' : 'folders'} are created: <code class="bg-white/50 px-1 rounded">step-N-start</code> and <code class="bg-white/50 px-1 rounded">step-N-end</code></li>
                                    <li>All uploaded base files are copied to each {structureType === 'branch' ? 'branch' : 'folder'}</li>
                                    <li>You can later download, modify, and re-upload specific step files</li>
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
                            Creating Workspace...
                        {:else}
                            <FolderGit2 size={16} />
                            Create Workspace
                        {/if}
                    </button>
                </div>
            {/if}
        </div>
    </div>
</div>
