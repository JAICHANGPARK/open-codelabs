<script lang="ts">
    import { FileUp, FolderGit2, Download, AlertCircle, CheckCircle2, Loader2, Sparkles, Copy, GitCompare } from 'lucide-svelte';
    import {
        getCodeServerInfo,
        createCodeServer,
        createCodeServerBranch,
        createCodeServerFolder,
        downloadCodeServerWorkspace,
        getWorkspaceFiles,
        getWorkspaceFileContent,
        getWorkspaceFolderFiles,
        getWorkspaceFolderFileContent,
        updateWorkspaceBranchFiles,
        updateWorkspaceFolderFiles,
        type WorkspaceFile
    } from '$lib/api-backend';
    import WorkspaceBrowser from '$lib/components/admin/WorkspaceBrowser.svelte';
    import { streamGeminiStructuredOutput, type GeminiStructuredConfig } from '$lib/gemini';
    import type { Step } from '$lib/api';
    import { fade } from 'svelte/transition';
    import { t } from 'svelte-i18n';

    let { codelabId, steps, geminiApiKey = "" }: { codelabId: string; steps: Step[]; geminiApiKey?: string } = $props();

    let workspaceExists = $state(false);
    let workspaceInfo = $state<{ path: string; structure_type: string } | null>(null);
    let loading = $state(true);
    let creating = $state(false);
    let error = $state('');
    let success = $state('');
    let aiStatus = $state('');
    let aiError = $state('');
    let aiGenerating = $state(false);
    let aiCurrentStep = $state<number | null>(null);
    let aiCurrentPhase = $state<'start' | 'end' | null>(null);
    let aiStreamingText = $state('');
    let aiStepLogs = $state<{ step: number; phase: 'start' | 'end'; updated: number; deleted: number }[]>([]);

    // Workspace creation options
    let structureType = $state<'branch' | 'folder'>('branch');
    let uploadedFiles = $state<{ name: string; content: string }[]>([]);
    let fileInput = $state<HTMLInputElement | null>(null);
    let initialized = $state(false);

    type WorkspaceUpdate = {
        files: WorkspaceFile[];
        deleted_files: string[];
    };

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

    function parseStructuredJson<T>(raw: string): T | null {
        const trimmed = raw.trim();
        if (!trimmed) return null;
        const firstBrace = trimmed.indexOf("{");
        const lastBrace = trimmed.lastIndexOf("}");
        if (firstBrace === -1 || lastBrace === -1) return null;
        const jsonText = trimmed.substring(firstBrace, lastBrace + 1);
        try {
            return JSON.parse(jsonText) as T;
        } catch {
            return null;
        }
    }

    function mapToWorkspaceFiles(files: Map<string, string>): WorkspaceFile[] {
        return Array.from(files.entries()).map(([path, content]) => ({ path, content }));
    }

    async function loadBranchFiles(branch: string): Promise<Map<string, string>> {
        const fileList = await getWorkspaceFiles(codelabId, branch);
        const fileMap = new Map<string, string>();
        for (const file of fileList) {
            const content = await getWorkspaceFileContent(codelabId, branch, file);
            fileMap.set(file, content);
        }
        return fileMap;
    }

    async function loadFolderFiles(folder: string): Promise<Map<string, string>> {
        const fileList = await getWorkspaceFolderFiles(codelabId, folder);
        const fileMap = new Map<string, string>();
        for (const file of fileList) {
            const content = await getWorkspaceFolderFileContent(codelabId, folder, file);
            fileMap.set(file, content);
        }
        return fileMap;
    }

    async function applyBranchSnapshot(branch: string, files: Map<string, string>, commitMessage: string) {
        const existing = await getWorkspaceFiles(codelabId, branch);
        const deleteFiles = existing.filter((path) => !files.has(path));
        await updateWorkspaceBranchFiles(
            codelabId,
            branch,
            mapToWorkspaceFiles(files),
            deleteFiles,
            commitMessage
        );
    }

    async function applyFolderSnapshot(folder: string, files: Map<string, string>) {
        const existing = await getWorkspaceFolderFiles(codelabId, folder);
        const deleteFiles = existing.filter((path) => !files.has(path));
        await updateWorkspaceFolderFiles(codelabId, folder, mapToWorkspaceFiles(files), deleteFiles);
    }

    async function generateStepUpdate(
        step: Step,
        currentFiles: Map<string, string>,
        previousStepsSummary: string,
        stepIndex: number,
        totalSteps: number
    ): Promise<WorkspaceUpdate> {
        const schema = {
            type: "object",
            properties: {
                files: {
                    type: "array",
                    items: {
                        type: "object",
                        properties: {
                            path: { type: "string" },
                            content: { type: "string" }
                        },
                        required: ["path", "content"]
                    }
                },
                deleted_files: {
                    type: "array",
                    items: { type: "string" }
                }
            },
            required: ["files", "deleted_files"]
        };

        const systemPrompt =
            "You are a senior software engineer creating a step-by-step coding tutorial. " +
            "Update the codebase to match the END state of this tutorial step. " +
            "The START state is already prepared - you only need to apply the changes described in the step instructions. " +
            "Return JSON only and follow the schema strictly. Include full file contents for changed/new files only. " +
            "If a file should be deleted, list its path in deleted_files. Do not include explanations.";

        const filesPayload = JSON.stringify(
            Array.from(currentFiles.entries()).map(([path, content]) => ({ path, content })),
            null,
            2
        );

        const prompt =
            `=== Tutorial Context ===\n` +
            `This is Step ${stepIndex + 1} of ${totalSteps}.\n` +
            `${previousStepsSummary}\n\n` +
            `=== Current Step ===\n` +
            `Step Title: ${step.title}\n\n` +
            `Step Instructions (Markdown):\n${step.content_markdown}\n\n` +
            `=== Current Files (START state) ===\n${filesPayload}\n\n` +
            `=== Task ===\n` +
            `Apply the step instructions to transform the START state into the END state. ` +
            `Return only the files that need to be modified, added, or deleted. ` +
            `The next step will use this END state as its starting point.`;

        const config: GeminiStructuredConfig = {
            apiKey: geminiApiKey,
            model: "gemini-3-flash-preview"
        };

        let responseText = "";
        const stream = streamGeminiStructuredOutput(prompt, systemPrompt, schema, config);
        for await (const chunk of stream) {
            if (chunk.content) {
                responseText += chunk.content;
                aiStreamingText = responseText;
            }
        }

        const parsed = parseStructuredJson<WorkspaceUpdate>(responseText);
        if (!parsed) {
            throw new Error($t("ai_generator.error_parse"));
        }
        return {
            files: parsed.files ?? [],
            deleted_files: parsed.deleted_files ?? []
        };
    }

    // Build context summary from previous steps for better AI understanding
    function buildStepsContext(processedSteps: { step: Step; files: Map<string, string> }[]): string {
        if (processedSteps.length === 0) {
            return "This is the first step of the tutorial.";
        }

        const summaries = processedSteps.map((ps, idx) => {
            const fileList = Array.from(ps.files.keys()).slice(0, 5).join(", ");
            const moreFiles = ps.files.size > 5 ? ` and ${ps.files.size - 5} more files` : "";
            return `Step ${idx + 1}: "${ps.step.title}" - Modified files: ${fileList}${moreFiles}`;
        });

        return "Previous steps completed:\n" + summaries.join("\n");
    }

    async function handleGenerateWorkspaceWithAi() {
        if (!geminiApiKey) {
            aiError = $t("ai_generator.api_key_required");
            return;
        }
        if (!workspaceExists || !workspaceInfo) {
            aiError = $t("workspace.errors.not_found");
            return;
        }
        if (!steps.length) {
            aiError = $t("workspace.errors.steps_required");
            return;
        }

        aiGenerating = true;
        aiError = "";
        aiStatus = $t("workspace.ai_running");
        aiCurrentStep = null;
        aiCurrentPhase = null;
        aiStreamingText = "";
        aiStepLogs = [];

        try {
            const structure = workspaceInfo.structure_type;
            const initialKey = `step-1-start`;
            let currentFiles = structure === "branch"
                ? await loadBranchFiles(initialKey)
                : await loadFolderFiles(initialKey);

            // Track processed steps for context building
            const processedSteps: { step: Step; files: Map<string, string> }[] = [];

            for (let i = 0; i < steps.length; i++) {
                const stepNumber = i + 1;
                const startKey = `step-${stepNumber}-start`;
                const endKey = `step-${stepNumber}-end`;
                aiCurrentStep = stepNumber;
                aiStreamingText = "";

                // Step 1 Start: Keep the uploaded base files as-is (no AI needed)
                // Step N Start (N>1): Copy from Step N-1 End (no AI needed)
                if (stepNumber > 1) {
                    // Copy previous end state to current start state
                    aiCurrentPhase = "start";
                    aiStatus = $t("workspace.ai_progress_copy", {
                        values: { current: stepNumber, total: steps.length }
                    });

                    if (structure === "branch") {
                        await applyBranchSnapshot(startKey, currentFiles, `Step ${stepNumber} start (copied from step ${stepNumber - 1} end)`);
                    } else {
                        await applyFolderSnapshot(startKey, currentFiles);
                    }

                    aiStepLogs = [
                        ...aiStepLogs,
                        {
                            step: stepNumber,
                            phase: "start",
                            updated: 0,
                            deleted: 0
                        }
                    ];
                } else {
                    // Step 1: Use uploaded base files directly
                    aiCurrentPhase = "start";
                    aiStatus = $t("workspace.ai_progress_base", {
                        values: { current: stepNumber, total: steps.length }
                    });

                    if (structure === "branch") {
                        await applyBranchSnapshot(startKey, currentFiles, `Step ${stepNumber} start (base files)`);
                    } else {
                        await applyFolderSnapshot(startKey, currentFiles);
                    }

                    aiStepLogs = [
                        ...aiStepLogs,
                        {
                            step: stepNumber,
                            phase: "start",
                            updated: 0,
                            deleted: 0
                        }
                    ];
                }

                // Generate END state with AI (only AI call per step)
                aiCurrentPhase = "end";
                aiStatus = $t("workspace.ai_progress_end", {
                    values: { current: stepNumber, total: steps.length }
                });

                const contextSummary = buildStepsContext(processedSteps);
                const update = await generateStepUpdate(steps[i], currentFiles, contextSummary, i, steps.length);

                for (const file of update.files) {
                    currentFiles.set(file.path, file.content);
                }
                for (const file of update.deleted_files) {
                    currentFiles.delete(file);
                }

                if (structure === "branch") {
                    await applyBranchSnapshot(endKey, currentFiles, `Step ${stepNumber} end (AI generated)`);
                } else {
                    await applyFolderSnapshot(endKey, currentFiles);
                }

                // Store for context in next steps
                processedSteps.push({
                    step: steps[i],
                    files: new Map(currentFiles)
                });

                aiStepLogs = [
                    ...aiStepLogs,
                    {
                        step: stepNumber,
                        phase: "end",
                        updated: update.files.length,
                        deleted: update.deleted_files.length
                    }
                ];
            }

            aiStatus = $t("workspace.ai_done");
        } catch (e) {
            aiStatus = '';
            aiError = (e as Error).message;
        } finally {
            aiGenerating = false;
        }
    }

    // Copy step end to next step start manually
    async function copyStepEndToNextStart(sourceStep: number) {
        if (!workspaceExists || !workspaceInfo) {
            error = $t("workspace.errors.not_found");
            return;
        }

        const targetStep = sourceStep + 1;
        if (targetStep > steps.length) {
            error = $t("workspace.errors.no_next_step");
            return;
        }

        try {
            success = '';
            error = '';
            const structure = workspaceInfo.structure_type;
            const sourceKey = `step-${sourceStep}-end`;
            const targetKey = `step-${targetStep}-start`;

            // Load files from source
            const files = structure === "branch"
                ? await loadBranchFiles(sourceKey)
                : await loadFolderFiles(sourceKey);

            // Save to target
            if (structure === "branch") {
                await applyBranchSnapshot(targetKey, files, `Step ${targetStep} start (copied from step ${sourceStep} end)`);
            } else {
                await applyFolderSnapshot(targetKey, files);
            }

            success = $t("workspace.success.copied", {
                values: { source: sourceStep, target: targetStep }
            });
        } catch (e) {
            error = $t("workspace.errors.copy_failed", { error: (e as Error).message });
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
        <div class="max-w-6xl space-y-6">
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

            {#if aiError}
                <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex items-start gap-3">
                    <AlertCircle class="text-red-600 dark:text-red-400 shrink-0 mt-0.5" size={20} />
                    <p class="text-red-800 dark:text-red-200 text-sm">{aiError}</p>
                </div>
            {/if}

            {#if aiStatus}
                <div class="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg flex items-start gap-3">
                    <Sparkles class="text-blue-600 dark:text-blue-400 shrink-0 mt-0.5" size={20} />
                    <p class="text-blue-800 dark:text-blue-200 text-sm">{aiStatus}</p>
                </div>
            {/if}

            {#if aiGenerating || aiStepLogs.length > 0}
                <div class="bg-[#F8F9FA] dark:bg-white/5 border border-[#E8EAED] dark:border-dark-border rounded-xl p-6 space-y-3">
                    <div class="flex items-center justify-between">
                        <h4 class="text-sm font-bold text-[#202124] dark:text-dark-text">
                            {$t('workspace.ai_response_title')}
                        </h4>
                        {#if aiCurrentStep !== null}
                            <span class="text-xs text-[#5F6368] dark:text-dark-text-muted">
                                {$t('workspace.ai_response_step', {
                                    values: {
                                        step: aiCurrentStep,
                                        phase: aiCurrentPhase
                                            ? $t(`workspace.ai_phase_${aiCurrentPhase}`)
                                            : ''
                                    }
                                })}
                            </span>
                        {/if}
                    </div>
                    <pre class="bg-[#0D1117] text-[#E8EAED] rounded-lg p-4 text-xs overflow-auto max-h-56">
{aiStreamingText || $t('workspace.ai_response_waiting')}
                    </pre>
                    {#if aiStepLogs.length > 0}
                        <div class="text-xs text-[#5F6368] dark:text-dark-text-muted space-y-1">
                            {#each aiStepLogs as log}
                                <div>
                                    {$t('workspace.ai_step_summary', {
                                        values: {
                                            step: log.step,
                                            phase: $t(`workspace.ai_phase_${log.phase}`),
                                            updated: log.updated,
                                            deleted: log.deleted
                                        }
                                    })}
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/if}

            {#if loading}
                <div class="flex items-center justify-center py-12">
                    <Loader2 class="animate-spin text-[#4285F4]" size={32} />
                </div>
            {:else}
                {#if workspaceExists && workspaceInfo}
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
                                            values: {
                                                count: steps.length,
                                                unit: workspaceInfo.structure_type === 'branch' ? $t('workspace.labels.branches') : $t('workspace.labels.folders'),
                                                total: steps.length * 2
                                            }
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

                    <div class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-6 flex flex-col gap-3">
                        <div class="flex items-center gap-3">
                            <div class="p-2 bg-[#34A853]/10 rounded-lg text-[#34A853]">
                                <Sparkles size={20} />
                            </div>
                            <div>
                                <h3 class="text-lg font-bold text-[#202124] dark:text-dark-text">
                                    {$t('workspace.ai_title')}
                                </h3>
                                <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                                    {$t('workspace.ai_desc')}
                                </p>
                            </div>
                        </div>
                        <button
                            onclick={handleGenerateWorkspaceWithAi}
                            disabled={aiGenerating}
                            class="flex items-center gap-2 px-4 py-2 bg-[#34A853] text-white rounded-lg hover:bg-[#1E8E3E] transition-colors text-sm font-medium shadow-sm disabled:opacity-50 disabled:cursor-not-allowed w-fit"
                        >
                            {#if aiGenerating}
                                <Loader2 class="animate-spin" size={16} />
                                {$t('workspace.ai_running')}
                            {:else}
                                <Sparkles size={16} />
                                {$t('workspace.ai_button')}
                            {/if}
                        </button>
                    </div>

                    <!-- Step Copy Management -->
                    <div class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-6 flex flex-col gap-3">
                        <div class="flex items-center gap-3">
                            <div class="p-2 bg-[#4285F4]/10 rounded-lg text-[#4285F4]">
                                <GitCompare size={20} />
                            </div>
                            <div>
                                <h3 class="text-lg font-bold text-[#202124] dark:text-dark-text">
                                    {$t('workspace.copy_title')}
                                </h3>
                                <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                                    {$t('workspace.copy_desc')}
                                </p>
                            </div>
                        </div>

                        <div class="flex flex-wrap gap-2 mt-2">
                            {#each steps.slice(0, -1) as step, i}
                                <button
                                    onclick={() => copyStepEndToNextStart(i + 1)}
                                    disabled={aiGenerating}
                                    class="flex items-center gap-2 px-3 py-1.5 bg-[#F8F9FA] dark:bg-white/5 border border-[#DADCE0] dark:border-dark-border rounded-lg hover:bg-[#E8EAED] dark:hover:bg-white/10 transition-colors text-xs font-medium text-[#5F6368] dark:text-dark-text disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    <Copy size={12} />
                                    {$t('workspace.copy_button', { values: { source: i + 1, target: i + 2 } })}
                                </button>
                            {/each}
                        </div>
                    </div>
                {:else}
                    <!-- Create New Workspace -->
                    <div class="space-y-6">
                        <!-- Structure Type Selection -->
                        <div class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-xl border border-[#E8EAED] dark:border-dark-border">
                            <div class="block text-sm font-bold text-[#202124] dark:text-dark-text mb-3">
                                {$t('workspace.structure_title')}
                            </div>
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
                            <div class="block text-sm font-bold text-[#202124] dark:text-dark-text mb-3">
                                {$t('workspace.base_files')}
                            </div>
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

                {#if workspaceExists && workspaceInfo}
                    <WorkspaceBrowser codelabId={codelabId} embedded />
                {:else}
                    <div class="bg-[#F8F9FA] dark:bg-white/5 border border-[#E8EAED] dark:border-dark-border rounded-xl p-6">
                        <h3 class="text-lg font-bold text-[#202124] dark:text-dark-text mb-2">
                            {$t('workspace.browser_title')}
                        </h3>
                        <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                            {$t('workspace.browser_empty')}
                        </p>
                    </div>
                {/if}
            {/if}
        </div>
    </div>
</div>
