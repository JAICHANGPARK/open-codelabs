<script lang="ts">
    import { onMount } from 'svelte';
    import { browser } from '$app/environment';
    import DOMPurify from 'dompurify';
    import hljs from 'highlight.js';
    import { getWorkspaceBranches, getWorkspaceFiles, getWorkspaceFileContent, getCodeServerInfo } from '$lib/api-backend';
    import { adminMarked } from '$lib/markdown';
    import WorkspaceFileTree from '$lib/components/admin/WorkspaceFileTree.svelte';
    import { t } from 'svelte-i18n';

    let { codelabId, onClose = null, embedded = false } = $props<{
        codelabId: string;
        onClose?: (() => void) | null;
        embedded?: boolean;
    }>();

    type TreeNode = {
        name: string;
        path: string;
        type: 'folder' | 'file';
        children?: TreeNode[];
    };

    type NotebookCell = {
        kind: 'markdown' | 'code';
        source: string;
    };

    let structureType = $state<'branch' | 'folder'>('branch');
    let items = $state<string[]>([]); // branches or folders
    let selectedItem = $state('');
    let files = $state<string[]>([]);
    let selectedFile = $state('');
    let fileContent = $state('');
    let highlightedContent = $state('');
    let treeNodes = $state<TreeNode[]>([]);
    let expandedFolders = $state(new Set<string>());
    let isNotebook = $state(false);
    let notebookCells = $state<NotebookCell[]>([]);
    let notebookError = $state('');
    let notebookLanguage = $state('');
    let loading = $state(false);
    let error = $state('');

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
            isNotebook = false;
            notebookCells = [];
            notebookError = '';
            notebookLanguage = '';
            highlightedContent = '';
            treeNodes = [];
            expandedFolders = new Set();

            if (structureType === 'branch') {
                files = await getWorkspaceFiles(codelabId, selectedItem);
            } else {
                const { getWorkspaceFolderFiles } = await import('$lib/api-backend');
                files = await getWorkspaceFolderFiles(codelabId, selectedItem);
            }

            treeNodes = buildFileTree(files);
        } catch (e) {
            error = $t('workspace.errors.load_files_failed', { error: (e as Error).message });
        } finally {
            loading = false;
        }
    }

    function toNotebookSource(value: unknown): string {
        if (Array.isArray(value)) return value.join('');
        if (typeof value === 'string') return value;
        return '';
    }

    function parseNotebook(content: string): { cells: NotebookCell[]; language: string } {
        const parsed = JSON.parse(content) as {
            cells?: Array<{ cell_type?: string; source?: unknown }>;
            metadata?: { language_info?: { name?: string } };
        };
        const cells = Array.isArray(parsed?.cells) ? parsed.cells : [];
        const language = typeof parsed?.metadata?.language_info?.name === 'string'
            ? parsed.metadata.language_info.name
            : '';

        const extracted = cells.flatMap((cell) => {
            const cellType = cell?.cell_type;
            if (cellType !== 'markdown' && cellType !== 'code') return [];
            const source = toNotebookSource(cell?.source);
            return [{ kind: cellType === 'markdown' ? 'markdown' : 'code', source }];
        });

        return { cells: extracted, language };
    }

    function renderMarkdown(source: string): string {
        if (!source) return '';
        try {
            const html = adminMarked.parse(source) as string;
            return browser ? DOMPurify.sanitize(html) : html;
        } catch {
            return source;
        }
    }

    function languageFromFilename(filename: string): string {
        const name = filename.toLowerCase();
        if (name.endsWith('.ts') || name.endsWith('.tsx')) return 'typescript';
        if (name.endsWith('.js') || name.endsWith('.jsx')) return 'javascript';
        if (name.endsWith('.json')) return 'json';
        if (name.endsWith('.yml') || name.endsWith('.yaml')) return 'yaml';
        if (name.endsWith('.toml')) return 'toml';
        if (name.endsWith('.md') || name.endsWith('.markdown')) return 'markdown';
        if (name.endsWith('.py')) return 'python';
        if (name.endsWith('.go')) return 'go';
        if (name.endsWith('.java')) return 'java';
        if (name.endsWith('.kt') || name.endsWith('.kts')) return 'kotlin';
        if (name.endsWith('.rs')) return 'rust';
        if (name.endsWith('.dart')) return 'dart';
        if (name.endsWith('.html') || name.endsWith('.xml')) return 'xml';
        if (name.endsWith('.css') || name.endsWith('.scss')) return 'css';
        if (name.endsWith('.sh') || name.endsWith('.bash')) return 'bash';
        if (name.endsWith('.sql')) return 'sql';
        if (name.endsWith('.c') || name.endsWith('.h')) return 'c';
        if (name.endsWith('.cpp') || name.endsWith('.cc') || name.endsWith('.hpp')) return 'cpp';
        return '';
    }

    function escapeHtml(value: string): string {
        return value
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    function renderCodeBlock(source: string, languageHint: string): string {
        if (!source) return '';
        try {
            const normalized = languageHint.trim().toLowerCase();
            if (normalized) {
                // Try to highlight with specified language
                const result = hljs.highlight(source, { language: normalized, ignoreIllegals: true });
                return result.value;
            }
            // Auto-detect language if no hint provided
            return hljs.highlightAuto(source).value;
        } catch {
            // Fallback to auto-highlight or plain text
            try {
                return hljs.highlightAuto(source).value;
            } catch {
                return escapeHtml(source);
            }
        }
    }

    function buildFileTree(paths: string[]): TreeNode[] {
        const root: TreeNode[] = [];

        for (const filePath of paths) {
            const parts = filePath.split('/').filter(Boolean);
            let current = root;
            let currentPath = '';

            parts.forEach((part, index) => {
                currentPath = currentPath ? `${currentPath}/${part}` : part;
                const isFile = index === parts.length - 1;
                let node = current.find((entry) => entry.name === part);

                if (!node) {
                    node = {
                        name: part,
                        path: currentPath,
                        type: isFile ? 'file' : 'folder',
                        children: isFile ? undefined : []
                    };
                    current.push(node);
                }

                if (!isFile) {
                    if (!node.children) node.children = [];
                    current = node.children;
                }
            });
        }

        const sortNodes = (nodes: TreeNode[]) => {
            nodes.sort((a, b) => {
                if (a.type !== b.type) return a.type === 'folder' ? -1 : 1;
                return a.name.localeCompare(b.name);
            });
            for (const node of nodes) {
                if (node.children) sortNodes(node.children);
            }
        };

        sortNodes(root);
        return root;
    }

    function toggleFolder(path: string) {
        const next = new Set(expandedFolders);
        if (next.has(path)) {
            next.delete(path);
        } else {
            next.add(path);
        }
        expandedFolders = next;
    }

    async function loadFileContent(file: string) {
        try {
            loading = true;
            error = '';
            selectedFile = file;
            isNotebook = false;
            notebookCells = [];
            notebookError = '';
            notebookLanguage = '';
            highlightedContent = '';

            if (structureType === 'branch') {
                fileContent = await getWorkspaceFileContent(codelabId, selectedItem, file);
            } else {
                const { getWorkspaceFolderFileContent } = await import('$lib/api-backend');
                fileContent = await getWorkspaceFolderFileContent(codelabId, selectedItem, file);
            }

            if (file.toLowerCase().endsWith('.ipynb')) {
                isNotebook = true;
                try {
                    const parsed = parseNotebook(fileContent);
                    notebookCells = parsed.cells;
                    notebookLanguage = parsed.language;
                } catch {
                    notebookError = $t('workspace.notebook.invalid');
                }
            } else {
                highlightedContent = renderCodeBlock(fileContent, languageFromFilename(file));
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

<div class="workspace-browser" class:embedded={embedded}>
    <div class="header">
        <h2>{$t('workspace.browser_title')}</h2>
        {#if !embedded && onClose}
            <button onclick={onClose} class="close-btn">✕</button>
        {/if}
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
                    <WorkspaceFileTree
                        nodes={treeNodes}
                        selectedFile={selectedFile}
                        expandedFolders={expandedFolders}
                        onToggleFolder={toggleFolder}
                        onFileSelect={loadFileContent}
                    />
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
                {#if isNotebook}
                    {#if notebookError}
                        <div class="error">{notebookError}</div>
                    {:else if notebookCells.length === 0}
                        <div class="empty">{$t('workspace.notebook.empty')}</div>
                    {:else}
                        <div class="notebook">
                            {#each notebookCells as cell}
                                <div class="notebook-cell">
                                    <div class="cell-label">
                                        {cell.kind === 'markdown'
                                            ? $t('workspace.notebook.markdown_label')
                                            : $t('workspace.notebook.code_label')}
                                    </div>
                                    {#if cell.kind === 'markdown'}
                                        <div class="markdown-body">
                                            {@html renderMarkdown(cell.source)}
                                        </div>
                                    {:else}
                                        <pre class="code notebook-code">
                                            <code class="hljs">{@html renderCodeBlock(cell.source, notebookLanguage)}</code>
                                        </pre>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {/if}
                {:else}
                    <pre class="code">
                        <code class="hljs">{@html highlightedContent}</code>
                    </pre>
                {/if}
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

    .workspace-browser.embedded {
        position: relative;
        top: auto;
        left: auto;
        transform: none;
        width: 100%;
        max-width: none;
        height: 65vh;
        z-index: 1;
        box-shadow: none;
        border: 1px solid #e0e0e0;
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
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
        font-size: 0.9rem;
        line-height: 1.5;
        background: #0d1117;
        white-space: pre-wrap;
        word-wrap: break-word;
    }

    .code code {
        display: block;
        white-space: pre;
    }

    .notebook {
        flex: 1;
        padding: 1.5rem;
        overflow: auto;
        background: #fafafa;
    }

    .notebook-cell {
        border: 1px solid #e0e0e0;
        background: white;
        border-radius: 6px;
        margin-bottom: 1rem;
        overflow: hidden;
    }

    .cell-label {
        padding: 0.4rem 0.75rem;
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        color: #5f6368;
        background: #f1f3f4;
        border-bottom: 1px solid #e0e0e0;
    }

    .notebook-code {
        background: #0d1117;
    }

    .notebook .markdown-body {
        padding: 0.75rem 1rem;
    }

    .loading,
    .empty {
        padding: 2rem;
        text-align: center;
        color: #666;
    }

    :global(html.dark) .workspace-browser {
        background: var(--color-dark-surface);
        color: var(--color-dark-text);
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.45);
    }

    :global(html.dark) .workspace-browser.embedded {
        border-color: var(--color-dark-border);
    }

    :global(html.dark) .header {
        border-bottom-color: var(--color-dark-border);
    }

    :global(html.dark) .header h2 {
        color: var(--color-dark-text);
    }

    :global(html.dark) .close-btn {
        color: var(--color-dark-text-muted);
    }

    :global(html.dark) .close-btn:hover {
        color: var(--color-dark-text);
    }

    :global(html.dark) .error {
        background: rgba(120, 30, 30, 0.35);
        color: #fca5a5;
    }

    :global(html.dark) .sidebar {
        border-right-color: var(--color-dark-border);
    }

    :global(html.dark) .branch-selector {
        border-bottom-color: var(--color-dark-border);
    }

    :global(html.dark) .branch-selector label {
        color: var(--color-dark-text);
    }

    :global(html.dark) .branch-selector select {
        background: var(--color-dark-bg);
        color: var(--color-dark-text);
        border-color: var(--color-dark-border);
    }

    :global(html.dark) .file-tree h3 {
        color: var(--color-dark-text-muted);
    }

    :global(html.dark) .file-header {
        border-bottom-color: var(--color-dark-border);
    }

    :global(html.dark) .file-header h3 {
        color: var(--color-dark-text);
    }

    :global(html.dark) .notebook {
        background: var(--color-dark-bg);
    }

    :global(html.dark) .notebook-cell {
        background: var(--color-dark-surface);
        border-color: var(--color-dark-border);
    }

    :global(html.dark) .cell-label {
        color: var(--color-dark-text-muted);
        background: rgba(255, 255, 255, 0.05);
        border-bottom-color: var(--color-dark-border);
    }

    :global(html.dark) .notebook-code {
        background: #0d1117;
    }

    :global(html.dark) .loading,
    :global(html.dark) .empty {
        color: var(--color-dark-text-muted);
    }
</style>
