<script lang="ts">
    import {
        Heading1,
        Heading2,
        Heading3,
        Bold,
        Italic,
        List,
        ListOrdered,
        CheckSquare,
        Code,
        Code2,
        Link,
        Quote,
        Table2,
        Image as ImageIcon,
        Columns2,
        BookOpen,
        Terminal,
        Sparkles,
        Loader2,
        Send
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import type { Step } from "$lib/api";
    import hljs from "highlight.js";

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
        insertMarkdown: (
            type: string,
            options?: { language?: string; snippet?: string; url?: string }
        ) => void;
        handleFileSelect: (e: Event) => void;
        handleKeydown: (e: KeyboardEvent) => void;
        handlePaste: (e: ClipboardEvent) => void;
        handleMouseUp: (e: MouseEvent) => void;
        improveWithAi: (instruction?: string) => void;
        syncEditorScroll: () => void;
        syncPreviewScroll: () => void;
    }>();

    const languageOptions = hljs
        .listLanguages()
        .map((lang) => ({
            value: lang,
            label: hljs.getLanguage(lang)?.name || lang,
        }))
        .sort((a, b) => a.label.localeCompare(b.label));

    let codeLanguage = $state("");

    let wordCount = $derived.by(() => {
        const content = step?.content_markdown || "";
        const trimmed = content.trim();
        return trimmed ? trimmed.split(/\s+/).length : 0;
    });

    let charCount = $derived.by(() => {
        const content = step?.content_markdown || "";
        return content.length;
    });

    let lineCount = $derived.by(() => {
        const content = step?.content_markdown || "";
        return content ? content.split("\n").length : 0;
    });

    let selectionCount = $derived.by(() => {
        return selectedText ? selectedText.length : 0;
    });

    const toolbarButtonClass =
        "p-2 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-white dark:hover:bg-white/10";
    const toolbarDividerClass =
        "w-px h-6 bg-[#DADCE0] dark:bg-dark-border mx-1";
    const quickBlockClass =
        "flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[11px] font-bold bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:border-[#4285F4]/40 transition-colors";
</script>

<div class="sticky top-[166px] z-20 mb-4">
    <div class="flex flex-col gap-2 p-2 bg-[#F8F9FA]/90 dark:bg-white/5 backdrop-blur-sm rounded-2xl border border-[#E8EAED] dark:border-dark-border">
        <div class="flex flex-wrap items-center gap-2">
            <div class="flex items-center gap-1">
                <button
                    onclick={() => insertMarkdown("h1")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.heading1")}
                    aria-label={$t("editor.toolbar.heading1")}
                >
                    <Heading1 size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("h2")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.heading2")}
                    aria-label={$t("editor.toolbar.heading2")}
                >
                    <Heading2 size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("h3")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.heading3")}
                    aria-label={$t("editor.toolbar.heading3")}
                >
                    <Heading3 size={18} />
                </button>
            </div>
            <div class={toolbarDividerClass}></div>
            <div class="flex items-center gap-1">
                <button
                    onclick={() => insertMarkdown("bold")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.bold")}
                    aria-label={$t("editor.toolbar.bold")}
                >
                    <Bold size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("italic")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.italic")}
                    aria-label={$t("editor.toolbar.italic")}
                >
                    <Italic size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("inline_code")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.inline_code")}
                    aria-label={$t("editor.toolbar.inline_code")}
                >
                    <Code2 size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("link")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.link")}
                    aria-label={$t("editor.toolbar.link")}
                >
                    <Link size={18} />
                </button>
            </div>
            <div class={toolbarDividerClass}></div>
            <div class="flex items-center gap-1">
                <button
                    onclick={() => insertMarkdown("list")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.bullet_list")}
                    aria-label={$t("editor.toolbar.bullet_list")}
                >
                    <List size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("ordered_list")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.numbered_list")}
                    aria-label={$t("editor.toolbar.numbered_list")}
                >
                    <ListOrdered size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("task_list")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.task_list")}
                    aria-label={$t("editor.toolbar.task_list")}
                >
                    <CheckSquare size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("quote")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.quote")}
                    aria-label={$t("editor.toolbar.quote")}
                >
                    <Quote size={18} />
                </button>
            </div>
            <div class={toolbarDividerClass}></div>
            <div class="flex flex-wrap items-center gap-2">
                <div class="flex items-center gap-1 rounded-lg border border-[#E8EAED] dark:border-dark-border bg-white/80 dark:bg-dark-surface/60 px-2 py-1">
                    <span class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">
                        {$t("editor.toolbar.code_language")}
                    </span>
                    <select
                        bind:value={codeLanguage}
                        class="bg-transparent text-xs font-bold text-[#3C4043] dark:text-dark-text outline-none"
                        aria-label={$t("editor.toolbar.code_language")}
                    >
                        <option value="">({$t("editor.toolbar.auto")})</option>
                        {#each languageOptions as language}
                            <option value={language.value}>{language.label}</option>
                        {/each}
                    </select>
                </div>
                <button
                    onclick={() => insertMarkdown("code_block", { language: codeLanguage })}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.code_block")}
                    aria-label={$t("editor.toolbar.code_block")}
                >
                    <Code size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("table")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.table")}
                    aria-label={$t("editor.toolbar.table")}
                >
                    <Table2 size={18} />
                </button>
                <button
                    onclick={() => insertMarkdown("image")}
                    class={toolbarButtonClass}
                    title={$t("editor.toolbar.image")}
                    aria-label={$t("editor.toolbar.image")}
                >
                    <ImageIcon size={18} />
                </button>
            </div>
            <div class="ml-auto flex items-center gap-2">
                <button
                    onclick={() => (isSplitView = !isSplitView)}
                    class="flex items-center gap-2 px-3 py-2 rounded-full transition-colors border border-[#E8EAED] dark:border-dark-border {isSplitView ? 'text-[#4285F4] bg-white dark:bg-white/10 shadow-sm' : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-white dark:hover:bg-white/10'}"
                    title={$t("editor.split_view")}
                    aria-label={$t("editor.split_view")}
                >
                    <Columns2 size={18} />
                    <span class="hidden sm:inline text-xs font-bold">
                        {$t("editor.split_view")}
                    </span>
                </button>
                <details class="relative">
                    <summary
                        class="list-none cursor-pointer flex items-center gap-2 px-3 py-2 rounded-full border border-[#E8EAED] dark:border-dark-border text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-white dark:hover:bg-white/10 transition-colors"
                        title={$t("editor.markdown_cheatsheet")}
                        aria-label={$t("editor.markdown_cheatsheet")}
                    >
                        <BookOpen size={18} />
                        <span class="hidden sm:inline text-xs font-bold">
                            {$t("editor.markdown_cheatsheet")}
                        </span>
                    </summary>
                    <div class="absolute right-0 mt-2 w-80 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-xl p-4 text-xs text-[#3C4043] dark:text-dark-text z-30">
                        <div class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider mb-2">
                            {$t("editor.markdown_cheatsheet")}
                        </div>
                        <pre class="font-mono text-[11px] leading-relaxed bg-[#F8F9FA] dark:bg-dark-bg rounded-lg p-3 border border-[#E8EAED] dark:border-dark-border whitespace-pre-wrap"># {$t("editor.toolbar.heading1")}
**{$t("editor.toolbar.bold")}** *{$t("editor.toolbar.italic")}* `{$t("editor.toolbar.inline_code")}`
- {$t("editor.toolbar.bullet_list")}
1. {$t("editor.toolbar.numbered_list")}
> {$t("editor.toolbar.quote")}
&#96;&#96;&#96;ts
{$t("editor.toolbar.code_block")}
&#96;&#96;&#96;</pre>
                    </div>
                </details>
            </div>
        </div>

        <div class="flex flex-wrap items-center gap-2 pt-1">
            <span class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">
                {$t("editor.snippets.label")}
            </span>
            <button
                onclick={() => insertMarkdown("snippet", { snippet: $t("editor.snippets.step_outline") })}
                class={quickBlockClass}
            >
                <ListOrdered size={12} />
                <span>{$t("editor.snippets.step_outline_label")}</span>
            </button>
            <button
                onclick={() => insertMarkdown("snippet", { snippet: $t("editor.snippets.checklist") })}
                class={quickBlockClass}
            >
                <CheckSquare size={12} />
                <span>{$t("editor.snippets.checklist_label")}</span>
            </button>
            <button
                onclick={() => insertMarkdown("snippet", { snippet: $t("editor.snippets.callout") })}
                class={quickBlockClass}
            >
                <Quote size={12} />
                <span>{$t("editor.snippets.callout_label")}</span>
            </button>
            <button
                onclick={() => insertMarkdown("snippet", { snippet: $t("editor.snippets.command_block") })}
                class={quickBlockClass}
            >
                <Terminal size={12} />
                <span>{$t("editor.snippets.command_block_label")}</span>
            </button>
        </div>
    </div>
</div>

<input
    type="file"
    accept="image/*"
    class="hidden"
    bind:this={fileInput}
    onchange={handleFileSelect}
/>

<div class="flex-1 flex flex-col min-h-[60vh] relative">
    <div class="flex-1 grid {isSplitView ? 'grid-cols-1 lg:grid-cols-2' : 'grid-cols-1'} gap-6 relative">
        <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between px-1">
                <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">
                    {$t("editor.markdown_editor")}
                </span>
            </div>
            <textarea
                bind:this={editorEl}
                onscroll={syncEditorScroll}
                bind:value={step.content_markdown}
                onkeydown={handleKeydown}
                onpaste={handlePaste}
                readonly={aiLoading}
                class="flex-1 w-full p-6 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl font-mono text-sm sm:text-base leading-relaxed resize-none outline-none focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 transition-all text-[#3C4043] dark:text-dark-text"
                style={aiLoading ? "cursor: wait;" : ""}
                placeholder={$t("editor.start_writing")}
                onmouseup={handleMouseUp}
            ></textarea>
        </div>

        {#if isSplitView}
            <div class="flex flex-col gap-2">
                <div class="flex items-center justify-between px-1">
                    <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">
                        {$t("editor.live_preview")}
                    </span>
                </div>
                <div
                    bind:this={previewEl}
                    onscroll={syncPreviewScroll}
                    class="flex-1 w-full p-6 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-y-auto shadow-inner"
                >
                    <div class="prose dark:prose-invert prose-blue max-w-none markdown-body">
                        {@html renderedContent}
                    </div>
                </div>
            </div>
        {/if}
    </div>

    <div class="mt-4 flex flex-wrap items-center justify-between gap-2 text-[11px] text-[#5F6368] dark:text-dark-text-muted">
        <div class="flex items-center gap-3">
            <span>{$t("editor.stats.words", { values: { count: wordCount } })}</span>
            <span>{$t("editor.stats.chars", { values: { count: charCount } })}</span>
            <span>{$t("editor.stats.lines", { values: { count: lineCount } })}</span>
            {#if selectionCount > 0}
                <span>{$t("editor.stats.selection", { values: { count: selectionCount } })}</span>
            {/if}
        </div>
        <div class="hidden xl:flex items-center gap-2 text-[#9AA0A6] dark:text-dark-text-muted">
            <span>{$t("editor.shortcut_hint")}</span>
        </div>
    </div>
</div>

{#if showAiMenu}
    <div
        class="fixed z-50 animate-in fade-in zoom-in-95 duration-200 ai-menu-container"
        style="top: {menuPos.y}px; left: {menuPos.x}px;"
    >
        <div class="bg-white dark:bg-dark-surface rounded-2xl shadow-2xl border border-[#D2E3FC] dark:border-[#4285F4]/30 p-4 w-80 flex flex-col gap-3">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2 text-[#4285F4]">
                    <Sparkles size={18} />
                    <span class="font-bold text-sm">{$t("gemini.improve_with_gemini")}</span>
                </div>
                <span class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted">
                    {$t("editor.ai_menu.selection", { values: { count: selectionCount } })}
                </span>
            </div>

            <div class="space-y-2">
                <label
                    for="ai-instruction"
                    class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                >
                    {$t("gemini.improvement_instruction")}
                </label>
                <textarea
                    id="ai-instruction"
                    bind:value={aiInstruction}
                    placeholder={$t("gemini.improvement_placeholder")}
                    class="w-full h-20 p-2 text-xs bg-[#F8F9FA] dark:bg-white/5 border border-[#DADCE0] dark:border-dark-border rounded-lg outline-none focus:border-[#4285F4] dark:focus:border-[#4285F4] resize-none"
                    onkeydown={(e) => {
                        if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
                            e.preventDefault();
                            improveWithAi(aiInstruction);
                        }
                    }}
                ></textarea>
                <p class="text-[10px] text-[#9AA0A6] dark:text-dark-text-muted">
                    {$t("editor.ai_menu.submit_hint")}
                </p>
            </div>

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

            <div class="grid grid-cols-2 gap-1.5">
                <button
                    onclick={() => improveWithAi("Explain this for a developer new to the topic.")}
                    disabled={aiLoading || !geminiApiKey}
                    class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                >
                    {$t("editor.ai_menu.explain")}
                </button>
                <button
                    onclick={() => improveWithAi("Summarize this in 3-5 concise bullets.")}
                    disabled={aiLoading || !geminiApiKey}
                    class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                >
                    {$t("editor.ai_menu.summarize")}
                </button>
                <button
                    onclick={() => improveWithAi("Fix grammar, spelling, and punctuation.")}
                    disabled={aiLoading || !geminiApiKey}
                    class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                >
                    {$t("editor.ai_menu.fix_grammar")}
                </button>
                <button
                    onclick={() => improveWithAi("Improve clarity and structure for a technical audience.")}
                    disabled={aiLoading || !geminiApiKey}
                    class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                >
                    {$t("editor.ai_menu.improve_writing")}
                </button>
                <button
                    onclick={() => improveWithAi("Convert this into step-by-step instructions in Markdown.")}
                    disabled={aiLoading || !geminiApiKey}
                    class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                >
                    {$t("editor.ai_menu.steps")}
                </button>
                <button
                    onclick={() => improveWithAi("Add concise code comments where helpful without changing meaning.")}
                    disabled={aiLoading || !geminiApiKey}
                    class="py-1.5 px-2 bg-[#F1F3F4] dark:bg-white/5 hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] rounded-lg text-[10px] font-bold transition-all text-left"
                >
                    {$t("editor.ai_menu.code_comments")}
                </button>
            </div>

            {#if !geminiApiKey}
                <p class="text-[9px] text-[#EA4335] font-bold mt-1">
                    * {$t("ai_generator.api_key_required")}
                </p>
            {/if}
        </div>
    </div>
{/if}
