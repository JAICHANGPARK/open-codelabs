<script lang="ts">
    import { onMount } from 'svelte';
    import { getWorkspaceBranches, getWorkspaceFiles, getWorkspaceFileContent, getCodeServerInfo } from '$lib/api-backend';
    import { t } from 'svelte-i18n';

    export let codelabId: string;
    export let onClose: () => void;

    let structureType: 'branch' | 'folder' = 'branch';
    let items: string[] = []; // branches or folders
    let selectedItem = '';
    let files: string[] = [];
    let selectedFile = '';
    let fileContent = '';
    let loading = false;
    let error = '';

    onMount(async () => {
        await loadWorkspaceInfo();
    });

    async function loadWorkspaceInfo() {
        try {
            loading = true;
            error = '';

            // Get workspace info to determine structure type
            const info = await getCodeServerInfo(codelabId);
            structureType = info.structure_type as 'branch' | 'folder';

            // Load branches or folders
            if (structureType === 'branch') {
                items = await getWorkspaceBranches(codelabId);
            } else {
                const { getWorkspaceFolders } = await import('$lib/api-backend');
                items = await getWorkspaceFolders(codelabId);
            }

            if (items.length > 0) {
                selectedItem = items[0];
                await loadFiles();
            }
        } catch (e) {
            error = $t('workspace.errors.load_workspace_failed', { error: (e as Error).message });
        } finally {
            loading = false;
        }
    }

    async function loadFiles() {
        if (!selectedItem) return;
        try {
            loading = true;
            error = '';
            fileContent = '';
            selectedFile = '';

            if (structureType === 'branch') {
                files = await getWorkspaceFiles(codelabId, selectedItem);
            } else {
                const { getWorkspaceFolderFiles } = await import('$lib/api-backend');
                files = await getWorkspaceFolderFiles(codelabId, selectedItem);
            }
        } catch (e) {
            error = $t('workspace.errors.load_files_failed', { error: (e as Error).message });
        } finally {
            loading = false;
        }
    }

    async function loadFileContent(file: string) {
        try {
            loading = true;
            error = '';
            selectedFile = file;

            if (structureType === 'branch') {
                fileContent = await getWorkspaceFileContent(codelabId, selectedItem, file);
            } else {
                const { getWorkspaceFolderFileContent } = await import('$lib/api-backend');
                fileContent = await getWorkspaceFolderFileContent(codelabId, selectedItem, file);
            }
        } catch (e) {
            error = $t('workspace.errors.load_file_content_failed', { error: (e as Error).message });
        } finally {
            loading = false;
        }
    }

    async function handleItemChange() {
        await loadFiles();
    }
</script>

<div class="workspace-browser">
    <div class="header">
        <h2>{$t('workspace.browser_title')}</h2>
        <button onclick={onClose} class="close-btn">✕</button>
    </div>

    {#if error}
        <div class="error">{error}</div>
    {/if}

    <div class="content">
        <div class="sidebar">
            <div class="branch-selector">
                <label for="item">{structureType === 'branch' ? $t('workspace.labels.branch') : $t('workspace.labels.folder')}:</label>
                <select
                    id="item"
                    bind:value={selectedItem}
                    onchange={handleItemChange}
                    disabled={loading}
                >
                    {#each items as item}
                        <option value={item}>{item}</option>
                    {/each}
                </select>
            </div>

            <div class="file-tree">
                <h3>{$t('workspace.browser_files_title')}</h3>
                {#if loading && files.length === 0}
                    <div class="loading">{$t('workspace.loading')}</div>
                {:else if files.length === 0}
                    <div class="empty">{$t('workspace.no_files')}</div>
                {:else}
                    <ul>
                        {#each files as file}
                            <li
                                class:selected={selectedFile === file}
                                onclick={() => loadFileContent(file)}
                                onkeydown={(e) => e.key === 'Enter' && loadFileContent(file)}
                                role="button"
                                tabindex="0"
                            >
                                {file}
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        </div>

        <div class="file-viewer">
            {#if loading && selectedFile}
                <div class="loading">{$t('workspace.loading_file')}</div>
            {:else if selectedFile}
                <div class="file-header">
                    <h3>{selectedFile}</h3>
                </div>
                <pre class="code">{fileContent}</pre>
            {:else}
                <div class="empty">{$t('workspace.select_file')}</div>
            {/if}
        </div>
    </div>
</div>

<style>
    .workspace-browser {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 90vw;
        max-width: 1200px;
        height: 80vh;
        background: white;
        border-radius: 8px;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
        display: flex;
        flex-direction: column;
        z-index: 1000;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 1.5rem;
        border-bottom: 1px solid #e0e0e0;
    }

    .header h2 {
        margin: 0;
        font-size: 1.25rem;
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        color: #666;
        padding: 0;
        width: 30px;
        height: 30px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .close-btn:hover {
        color: #000;
    }

    .error {
        background: #fee;
        color: #c00;
        padding: 1rem;
        margin: 1rem;
        border-radius: 4px;
    }

    .content {
        display: flex;
        flex: 1;
        overflow: hidden;
    }

    .sidebar {
        width: 300px;
        border-right: 1px solid #e0e0e0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .branch-selector {
        padding: 1rem;
        border-bottom: 1px solid #e0e0e0;
    }

    .branch-selector label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
    }

    .branch-selector select {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ccc;
        border-radius: 4px;
    }

    .file-tree {
        flex: 1;
        overflow-y: auto;
        padding: 1rem;
    }

    .file-tree h3 {
        margin: 0 0 0.5rem 0;
        font-size: 0.9rem;
        color: #666;
        text-transform: uppercase;
    }

    .file-tree ul {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    .file-tree li {
        padding: 0.5rem;
        cursor: pointer;
        border-radius: 4px;
        font-size: 0.9rem;
        font-family: monospace;
    }

    .file-tree li:hover {
        background: #f5f5f5;
    }

    .file-tree li.selected {
        background: #e3f2fd;
        color: #1976d2;
    }

    .file-viewer {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .file-header {
        padding: 1rem 1.5rem;
        border-bottom: 1px solid #e0e0e0;
    }

    .file-header h3 {
        margin: 0;
        font-size: 1rem;
        font-family: monospace;
    }

    .code {
        flex: 1;
        margin: 0;
        padding: 1.5rem;
        overflow: auto;
        font-family: 'Courier New', monospace;
        font-size: 0.9rem;
        line-height: 1.5;
        background: #f8f8f8;
        white-space: pre-wrap;
        word-wrap: break-word;
    }

    .loading,
    .empty {
        padding: 2rem;
        text-align: center;
        color: #666;
    }
</style>
