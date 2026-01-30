<script lang="ts">
    import { ChevronRight, ChevronDown, FileText, Folder, FolderOpen } from 'lucide-svelte';

    type TreeNode = {
        name: string;
        path: string;
        type: 'folder' | 'file';
        children?: TreeNode[];
    };

    let {
        nodes,
        selectedFile,
        expandedFolders,
        onToggleFolder,
        onFileSelect
    } = $props<{
        nodes: TreeNode[];
        selectedFile: string;
        expandedFolders: Set<string>;
        onToggleFolder: (path: string) => void;
        onFileSelect: (path: string) => void;
    }>();

    const isExpanded = (path: string) => expandedFolders.has(path);
</script>

<ul class="tree">
    {#each nodes as node}
        {#if node.type === 'folder'}
            <li class="tree-item">
                <button
                    type="button"
                    class="tree-row folder-row"
                    onclick={() => onToggleFolder(node.path)}
                >
                    <span class="chevron">
                        {#if isExpanded(node.path)}
                            <ChevronDown size={14} />
                        {:else}
                            <ChevronRight size={14} />
                        {/if}
                    </span>
                    <span class="icon">
                        {#if isExpanded(node.path)}
                            <FolderOpen size={14} />
                        {:else}
                            <Folder size={14} />
                        {/if}
                    </span>
                    <span class="label">{node.name}</span>
                </button>

                {#if isExpanded(node.path) && node.children?.length}
                    <div class="tree-children">
                        <WorkspaceFileTree
                            nodes={node.children}
                            {selectedFile}
                            {expandedFolders}
                            {onToggleFolder}
                            {onFileSelect}
                        />
                    </div>
                {/if}
            </li>
        {:else}
            <li class="tree-item">
                <button
                    type="button"
                    class="tree-row file-row"
                    class:selected={selectedFile === node.path}
                    onclick={() => onFileSelect(node.path)}
                >
                    <span class="icon">
                        <FileText size={14} />
                    </span>
                    <span class="label">{node.name}</span>
                </button>
            </li>
        {/if}
    {/each}
</ul>

<style>
    .tree {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    .tree-item + .tree-item {
        margin-top: 0.15rem;
    }

    .tree-row {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        width: 100%;
        text-align: left;
        padding: 0.3rem 0.5rem;
        border-radius: 6px;
        border: none;
        background: transparent;
        cursor: pointer;
        color: #3c4043;
        font-size: 0.88rem;
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    }

    .tree-row:hover {
        background: #f1f3f4;
    }

    .file-row.selected {
        background: #e8f0fe;
        color: #1a73e8;
        font-weight: 600;
    }

    .chevron {
        display: inline-flex;
        width: 16px;
        justify-content: center;
        color: #5f6368;
    }

    .icon {
        display: inline-flex;
        color: #5f6368;
    }

    .tree-children {
        padding-left: 1rem;
        border-left: 1px dashed #e0e0e0;
        margin-left: 0.4rem;
        margin-top: 0.2rem;
    }
</style>
