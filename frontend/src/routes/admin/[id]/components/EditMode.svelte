<script lang="ts">
    import { 
        Heading1, 
        Bold, 
        Italic, 
        List, 
        Code, 
        Image as ImageIcon, 
        Columns2, 
        Sparkles, 
        Loader2, 
        Send 
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import type { Step } from "$lib/api";

    let {
        step = $bindable(),
        isSplitView = $bindable(),
        aiLoading,
        editorEl = $bindable(),
        previewEl = $bindable(),
        fileInput = $bindable(),
        aiInstruction = $bindable(),
        showAiMenu = $bindable(),
        menuPos,
        selectedText,
        renderedContent,
        geminiApiKey,
        insertMarkdown,
        handleFileSelect,
        handleKeydown,
        handlePaste,
        handleMouseUp,
        improveWithAi,
        syncEditorScroll,
        syncPreviewScroll
    } = $props<{
        step: Step;
        isSplitView: boolean;
        aiLoading: boolean;
        editorEl: HTMLTextAreaElement | null;
        previewEl: HTMLDivElement | null;
        fileInput: HTMLInputElement | undefined;
        aiInstruction: string;
        showAiMenu: boolean;
        menuPos: { x: number; y: number };
        selectedText: string;
        renderedContent: string;
        geminiApiKey: string;
        insertMarkdown: (type: string) => void;
        handleFileSelect: (e: Event) => void;
        handleKeydown: (e: KeyboardEvent) => void;
        handlePaste: (e: ClipboardEvent) => void;
        handleMouseUp: (e: MouseEvent) => void;
        improveWithAi: (instruction?: string) => void;
        syncEditorScroll: () => void;
        syncPreviewScroll: () => void;
    }>();
</script>

<div class="flex flex-wrap items-center gap-1 sm:gap-2 mb-4 p-2 bg-[#F8F9FA]/90 dark:bg-white/5 backdrop-blur-sm rounded-xl border border-[#E8EAED] dark:border-dark-border sticky top-[166px] z-20">
    <button
        onclick={() => insertMarkdown("h1")}
        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
        title="Heading"
        ><Heading1 size={20} /></button
    >
    <button
        onclick={() => insertMarkdown("bold")}
        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
        title="Bold"><Bold size={20} /></button
    >
    <button
        onclick={() => insertMarkdown("italic")}
        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
        title="Italic"
        ><Italic size={20} /></button
    >
    <div class="w-px h-6 bg-[#DADCE0] dark:bg-dark-border mx-1"></div>
    <button
        onclick={() => insertMarkdown("list")}
        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
        title="List"><List size={20} /></button
    >
    <button
        onclick={() => insertMarkdown("code")}
        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
        title="Code Block"
        ><Code size={20} /></button
    >
    <button
        onclick={() => insertMarkdown("image")}
        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
        title="Image"
        ><ImageIcon size={20} /></button
    >
    <div class="w-px h-6 bg-[#DADCE0] dark:bg-dark-border mx-1"></div>
    <button
        onclick={() => (isSplitView = !isSplitView)}
        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors {isSplitView ? 'text-[#4285F4] bg-white dark:bg-white/10 shadow-sm' : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]'}"
        title={$t("editor.split_view")}
        ><Columns2 size={20} /></button
    >
</div>

<input
    type="file"
    accept="image/*"
    class="hidden"
    bind:this={fileInput}
    onchange={handleFileSelect}
/>

<div class="flex-1 flex flex-col min-h-[60vh] relative">
    <div class="flex-1 grid {isSplitView ? 'grid-cols-1 lg:grid-cols-2 lg:h-[75vh]' : 'grid-cols-1'} gap-8 relative">
        <textarea
            bind:this={editorEl}
            onscroll={syncEditorScroll}
            bind:value={step.content_markdown}
            onkeydown={handleKeydown}
            onpaste={handlePaste}
            readonly={aiLoading}
            class="w-full h-full outline-none text-[#3C4043] dark:text-dark-text font-mono text-base leading-relaxed resize-none bg-transparent {isSplitView ? 'overflow-y-auto pr-2' : ''}"
            style={aiLoading ? "cursor: wait;" : ""}
            placeholder={$t("editor.start_writing")}
            onmouseup={handleMouseUp}
        ></textarea>

        {#if isSplitView}
            <div 
                bind:this={previewEl}
                onscroll={syncPreviewScroll}
                class="hidden lg:block border-l border-[#F1F3F4] dark:border-dark-border pl-8 overflow-y-auto"
            >
                <div class="prose dark:prose-invert prose-blue max-w-none markdown-body">
                    {@html renderedContent}
                </div>
            </div>
        {/if}
    </div>
</div>

{#if showAiMenu}
    <div
        class="fixed z-50 animate-in fade-in zoom-in-95 duration-200 ai-menu-container"
        style="top: {menuPos.y}px; left: {menuPos.x}px;"
    >
        <div class="bg-white dark:bg-dark-surface rounded-2xl shadow-2xl border border-[#D2E3FC] dark:border-[#4285F4]/30 p-4 w-72 flex flex-col gap-3">
             <div class="flex items-center gap-2 text-[#4285F4] mb-1">
                 <Sparkles size={18} />
                 <span class="font-bold text-sm">{$t("gemini.improve_with_gemini")}</span>
             </div>
             
             <div class="space-y-2">
                 <label for="ai-instruction" class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">
                     {$t("gemini.improvement_instruction")}
                 </label>
                 <textarea
                     id="ai-instruction"
                     bind:value={aiInstruction}
                     placeholder={$t("gemini.improvement_placeholder")}
                     class="w-full h-20 p-2 text-xs bg-[#F8F9FA] dark:bg-white/5 border border-[#DADCE0] dark:border-dark-border rounded-lg outline-none focus:border-[#4285F4] dark:focus:border-[#4285F4] resize-none"
                     onkeydown={(e) => {
                         if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                             e.preventDefault();
                             improveWithAi(aiInstruction);
                         }
                     }}
                 ></textarea>
             </div>

             <div class="flex flex-col gap-1.5">
                 <button
                     onclick={() => improveWithAi(aiInstruction)}
                     disabled={aiLoading || !geminiApiKey}
                     class="w-full py-2 bg-[#4285F4] hover:bg-[#1A73E8] disabled:opacity-50 text-white rounded-xl text-xs font-bold transition-all flex items-center justify-center gap-2"
                 >
                     {#if aiLoading}
                         <Loader2 size={14} class="animate-spin" />
                         <span>{$t("gemini.thinking")}</span>
                     {:else}
                         <Send size={14} />
                         <span>{$t("gemini.ai_improve_submit")}</span>
                     {/if}
                 </button>
                 
                 <div class="grid grid-cols-2 gap-1.5 mt-1">
                     <button
                         onclick={() => improveWithAi("Explain this simply")}
                         disabled={aiLoading || !geminiApiKey}
                         class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                     >
                         {$t("editor.ai_menu.explain")}
                     </button>
                     <button
                         onclick={() => improveWithAi("Summarize this")}
                         disabled={aiLoading || !geminiApiKey}
                         class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                     >
                         {$t("editor.ai_menu.summarize")}
                     </button>
                     <button
                         onclick={() => improveWithAi("Fix grammar and spelling")}
                         disabled={aiLoading || !geminiApiKey}
                         class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                     >
                         {$t("editor.ai_menu.fix_grammar")}
                     </button>
                     <button
                         onclick={() => improveWithAi("Improve writing style")}
                         disabled={aiLoading || !geminiApiKey}
                         class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                     >
                         {$t("editor.ai_menu.improve_writing")}
                     </button>
                 </div>
             </div>

             {#if !geminiApiKey}
                 <p class="text-[9px] text-[#EA4335] font-bold mt-1">
                     * {$t("ai_generator.api_key_required")}
                 </p>
             {/if}
        </div>
    </div>
{/if}
