<script lang="ts">
    import { fade, slide } from "svelte/transition";
    import {
        Sparkles,
        Loader2,
        Info,
        FileDown,
        FileText,
        FileCode,
        FileType,
        ChevronDown,
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
        X,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { adminMarked as marked } from "$lib/markdown";
    import DOMPurify from "dompurify";
    import { browser } from "$app/environment";
    import html2pdf from "html2pdf.js";
    import hljs from "highlight.js";
    // @ts-ignore
    import { asBlob } from "html-docx-js-typescript";

    let {
        guide_markdown = $bindable(),
        codelab_title = $t("editor.guide_tab"),
        isSaving,
        handleSave,
        generateGuideWithAi,
        generateGuideWithAiPro,
        isGenerating,
        isGuideProGenerating,
        guideProStage,
        guideProPlanOutput,
        guideProDraftOutput,
        guideProReviewOutput,
        guideProRevisedOutput,
    } = $props<{
        guide_markdown: string;
        codelab_title?: string;
        isSaving: boolean;
        handleSave: () => void;
        generateGuideWithAi: () => void;
        generateGuideWithAiPro: () => void;
        isGenerating: boolean;
        isGuideProGenerating: boolean;
        guideProStage: "plan" | "draft" | "review" | "revise" | null;
        guideProPlanOutput: string;
        guideProDraftOutput: string;
        guideProReviewOutput: string;
        guideProRevisedOutput: string;
    }>();

    let showExportMenu = $state(false);
    let showGuideProModal = $state(false);
    let isSplitView = $state(true);
    let editorEl = $state<HTMLTextAreaElement | null>(null);
    let previewEl = $state<HTMLDivElement | null>(null);
    let isScrollingEditor = false;
    let isScrollingPreview = false;
    let guideProDraftView = $state<"markdown" | "raw">("markdown");
    let guideProRevisedView = $state<"markdown" | "raw">("markdown");
    let guideProDiffView = $state<"unified" | "split">("unified");

    const languageOptions = hljs
        .listLanguages()
        .map((lang) => ({
            value: lang,
            label: hljs.getLanguage(lang)?.name || lang,
        }))
        .sort((a, b) => a.label.localeCompare(b.label));

    let codeLanguage = $state("");

    let wordCount = $derived.by(() => {
        const content = guide_markdown || "";
        const trimmed = content.trim();
        return trimmed ? trimmed.split(/\s+/).length : 0;
    });

    let charCount = $derived.by(() => {
        const content = guide_markdown || "";
        return content.length;
    });

    let lineCount = $derived.by(() => {
        const content = guide_markdown || "";
        return content ? content.split("\n").length : 0;
    });

    const toolbarButtonClass =
        "p-2 rounded-lg transition-colors text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:bg-white dark:hover:bg-white/10";
    const toolbarDividerClass = "w-px h-6 bg-border dark:bg-dark-border mx-1";
    const quickBlockClass =
        "flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[11px] font-bold bg-white dark:bg-dark-surface border border-border dark:border-dark-border text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:border-primary/40 transition-colors";

    const guideProStages = ["plan", "draft", "review", "revise"] as const;
    type GuideProStage = (typeof guideProStages)[number];
    const guideProStageLabels: Record<GuideProStage, string> = {
        plan: "editor.guide_pro_stage.plan",
        draft: "editor.guide_pro_stage.draft",
        review: "editor.guide_pro_stage.review",
        revise: "editor.guide_pro_stage.revise",
    };

    let guideProLastOutputStage = $derived.by(() => {
        if (guideProRevisedOutput) return "revise";
        if (guideProReviewOutput) return "review";
        if (guideProDraftOutput) return "draft";
        if (guideProPlanOutput) return "plan";
        return null;
    });

    let guideProStageIndex = $derived.by(() => {
        const stage = guideProStage || guideProLastOutputStage;
        if (!stage) return -1;
        return guideProStages.indexOf(stage);
    });

    let guideProDisplayStage = $derived.by(() => {
        return guideProStage || guideProLastOutputStage;
    });

    let guideProHasOutput = $derived.by(() => {
        return !!(
            guideProPlanOutput ||
            guideProDraftOutput ||
            guideProReviewOutput ||
            guideProRevisedOutput
        );
    });

    function extractYoutubeId(rawUrl: string): string | null {
        try {
            const url = new URL(rawUrl);
            const host = url.hostname.replace(/^www\./, "");
            if (host === "youtu.be") {
                return url.pathname.replace("/", "").split("/")[0] || null;
            }
            if (host === "youtube.com") {
                if (url.pathname === "/watch") {
                    return url.searchParams.get("v");
                }
                if (url.pathname.startsWith("/embed/")) {
                    return url.pathname.split("/")[2] || null;
                }
                if (url.pathname.startsWith("/shorts/")) {
                    return url.pathname.split("/")[2] || null;
                }
            }
        } catch (e) {
            // ignore invalid urls
        }
        return null;
    }

    function injectYoutubeEmbeds(html: string): string {
        const anchorRegex = /<a[^>]*href="([^"]+)"[^>]*>.*?<\/a>/gi;
        return html.replace(anchorRegex, (full, href) => {
            const id = extractYoutubeId(href);
            if (!id) return full;
            const embedUrl = `https://www.youtube-nocookie.com/embed/${id}`;
            return `
<div class="video-embed" style="width:100%;max-width:100%;display:block;margin:1.25rem 0;">
  <iframe
    src="${embedUrl}"
    title="YouTube video"
    loading="lazy"
    style="width:100%;height:auto;aspect-ratio:16/9;display:block;border:0;border-radius:16px;background:#000;"
    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
    referrerpolicy="strict-origin-when-cross-origin"
    allowfullscreen
  ></iframe>
</div>`;
        });
    }

    let renderedContent = $derived.by(() => {
        if (!guide_markdown) return "";
        try {
            const html = marked.parse(guide_markdown) as string;
            if (browser) {
                const sanitized = DOMPurify.sanitize(html);
                return injectYoutubeEmbeds(sanitized);
            }
            return html;
        } catch (e) {
            console.error("Markdown parse error", e);
            return $t("editor.parse_error");
        }
    });

    const renderMarkdown = (markdown: string) => {
        if (!markdown) return "";
        try {
            const html = marked.parse(markdown) as string;
            if (browser) {
                const sanitized = DOMPurify.sanitize(html);
                return injectYoutubeEmbeds(sanitized);
            }
            return html;
        } catch (e) {
            console.error("Markdown parse error", e);
            return $t("editor.parse_error");
        }
    };

    let guideProDraftHtml = $derived.by(() =>
        renderMarkdown(guideProDraftOutput),
    );
    let guideProRevisedHtml = $derived.by(() =>
        renderMarkdown(guideProRevisedOutput),
    );

    type DiffLine = {
        type: "equal" | "add" | "remove";
        text: string;
    };

    type DiffRow = {
        leftText: string;
        rightText: string;
        leftType: "equal" | "remove" | "empty";
        rightType: "equal" | "add" | "empty";
    };

    const MAX_DIFF_CELLS = 2_000_000;

    function buildLineDiff(
        sourceText: string,
        targetText: string,
    ): { lines: DiffLine[]; truncated: boolean } {
        const source = sourceText ? sourceText.split("\n") : [];
        const target = targetText ? targetText.split("\n") : [];
        const rows = source.length;
        const cols = target.length;

        if (!rows && !cols) {
            return { lines: [], truncated: false };
        }

        if (rows * cols > MAX_DIFF_CELLS) {
            return { lines: [], truncated: true };
        }

        const width = cols + 1;
        const dp = new Uint32Array((rows + 1) * width);

        for (let i = 1; i <= rows; i += 1) {
            for (let j = 1; j <= cols; j += 1) {
                const idx = i * width + j;
                if (source[i - 1] === target[j - 1]) {
                    dp[idx] = dp[(i - 1) * width + (j - 1)] + 1;
                } else {
                    const top = dp[(i - 1) * width + j];
                    const left = dp[i * width + (j - 1)];
                    dp[idx] = top > left ? top : left;
                }
            }
        }

        const lines: DiffLine[] = [];
        let i = rows;
        let j = cols;
        while (i > 0 && j > 0) {
            if (source[i - 1] === target[j - 1]) {
                lines.push({ type: "equal", text: source[i - 1] });
                i -= 1;
                j -= 1;
            } else {
                const top = dp[(i - 1) * width + j];
                const left = dp[i * width + (j - 1)];
                if (top >= left) {
                    lines.push({ type: "remove", text: source[i - 1] });
                    i -= 1;
                } else {
                    lines.push({ type: "add", text: target[j - 1] });
                    j -= 1;
                }
            }
        }
        while (i > 0) {
            lines.push({ type: "remove", text: source[i - 1] });
            i -= 1;
        }
        while (j > 0) {
            lines.push({ type: "add", text: target[j - 1] });
            j -= 1;
        }

        lines.reverse();
        return { lines, truncated: false };
    }

    let guideProDiff = $derived.by(() =>
        buildLineDiff(guideProDraftOutput, guideProRevisedOutput),
    );

    let guideProDiffRows = $derived.by(() => {
        if (guideProDiff.truncated) return [];
        return guideProDiff.lines.map<DiffRow>((line) => {
            if (line.type === "equal") {
                return {
                    leftText: line.text,
                    rightText: line.text,
                    leftType: "equal",
                    rightType: "equal",
                };
            }
            if (line.type === "remove") {
                return {
                    leftText: line.text,
                    rightText: "",
                    leftType: "remove",
                    rightType: "empty",
                };
            }
            return {
                leftText: "",
                rightText: line.text,
                leftType: "empty",
                rightType: "add",
            };
        });
    });

    type InsertOptions = {
        language?: string;
        snippet?: string;
        url?: string;
    };

    function syncEditorScroll() {
        if (isScrollingPreview || !editorEl || !previewEl) return;
        isScrollingEditor = true;
        const maxEditor = editorEl.scrollHeight - editorEl.clientHeight;
        const maxPreview = previewEl.scrollHeight - previewEl.clientHeight;

        if (maxEditor > 0 && maxPreview > 0) {
            const percentage = editorEl.scrollTop / maxEditor;
            previewEl.scrollTop = percentage * maxPreview;
        }

        setTimeout(() => (isScrollingEditor = false), 50);
    }

    function syncPreviewScroll() {
        if (isScrollingEditor || !editorEl || !previewEl) return;
        isScrollingPreview = true;
        const maxEditor = editorEl.scrollHeight - editorEl.clientHeight;
        const maxPreview = previewEl.scrollHeight - previewEl.clientHeight;

        if (maxEditor > 0 && maxPreview > 0) {
            const percentage = previewEl.scrollTop / maxPreview;
            editorEl.scrollTop = percentage * maxEditor;
        }

        setTimeout(() => (isScrollingPreview = false), 50);
    }

    function handleProGenerate() {
        if (isGenerating || isGuideProGenerating) return;
        showGuideProModal = true;
        guideProDraftView = "markdown";
        guideProRevisedView = "markdown";
        guideProDiffView = "unified";
        generateGuideWithAiPro();
    }

    function openGuideProModal() {
        showGuideProModal = true;
        guideProDiffView = "unified";
    }

    function insertMarkdown(type: string, options: InsertOptions = {}) {
        const textarea = editorEl;
        if (!textarea) return;

        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value || "";
        const selected = text.substring(start, end);
        const language = options.language ?? "";

        let replacement = "";
        let selectionStart = start;
        let selectionEnd = start;

        const setCursorToEnd = () => {
            selectionStart = start + replacement.length;
            selectionEnd = selectionStart;
        };

        const setSelection = (offset: number, length: number) => {
            selectionStart = start + offset;
            selectionEnd = selectionStart + length;
        };

        switch (type) {
            case "bold":
                if (selected) {
                    replacement = `**${selected}**`;
                    setCursorToEnd();
                } else {
                    const placeholder = "bold text";
                    replacement = `**${placeholder}**`;
                    setSelection(2, placeholder.length);
                }
                break;
            case "italic":
                if (selected) {
                    replacement = `*${selected}*`;
                    setCursorToEnd();
                } else {
                    const placeholder = "italic text";
                    replacement = `*${placeholder}*`;
                    setSelection(1, placeholder.length);
                }
                break;
            case "inline_code":
                if (selected) {
                    replacement = `\`${selected}\``;
                    setCursorToEnd();
                } else {
                    const placeholder = "code";
                    replacement = `\`${placeholder}\``;
                    setSelection(1, placeholder.length);
                }
                break;
            case "code":
            case "code_block": {
                const placeholder = selected || "// code here";
                const prefix = language ? `\n\`\`\`${language}\n` : "\n```\n";
                replacement = `${prefix}${placeholder}\n\`\`\`\n`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(prefix.length, placeholder.length);
                }
                break;
            }
            case "h1":
            case "h2":
            case "h3": {
                const level =
                    type === "h1" ? "#" : type === "h2" ? "##" : "###";
                const placeholder = "Heading";
                const content = selected || placeholder;
                replacement = `${level} ${content}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(level.length + 1, placeholder.length);
                }
                break;
            }
            case "list": {
                const placeholder = "list item";
                const lines = selected ? selected.split("\n") : [""];
                const listText = lines
                    .map((line) => `- ${line || placeholder}`)
                    .join("\n");
                replacement = selected ? listText : `\n${listText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "ordered_list": {
                const placeholder = "list item";
                const lines = selected ? selected.split("\n") : [""];
                const listText = lines
                    .map(
                        (line, index) => `${index + 1}. ${line || placeholder}`,
                    )
                    .join("\n");
                replacement = selected ? listText : `\n${listText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "task_list": {
                const placeholder = "task";
                const lines = selected ? selected.split("\n") : [""];
                const listText = lines
                    .map((line) => `- [ ] ${line || placeholder}`)
                    .join("\n");
                replacement = selected ? listText : `\n${listText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "quote": {
                const placeholder = "Quote";
                const lines = selected ? selected.split("\n") : [""];
                const quoteText = lines
                    .map((line) => `> ${line || placeholder}`)
                    .join("\n");
                replacement = selected ? quoteText : `\n${quoteText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "link": {
                const linkText = selected || "link text";
                const url = options.url || "https://";
                const prefix = `[${linkText}](`;
                replacement = `${prefix}${url})`;
                setSelection(prefix.length, url.length);
                break;
            }
            case "image": {
                const altText = selected || "image";
                const url = options.url || "https://";
                const prefix = `![${altText}](`;
                replacement = `${prefix}${url})`;
                setSelection(prefix.length, url.length);
                break;
            }
            case "table": {
                const header = "| Column | Column | Column |";
                const divider = "| --- | --- | --- |";
                const row = "| Cell | Cell | Cell |";
                const prefix = "\n| ";
                replacement = `\n${header}\n${divider}\n${row}\n`;
                setSelection(prefix.length, "Column".length);
                break;
            }
            case "snippet": {
                const snippet = options.snippet?.trimEnd();
                if (!snippet) return;
                const needsPrefix = start > 0 && text[start - 1] !== "\n";
                const needsSuffix = end < text.length && text[end] !== "\n";
                replacement = `${needsPrefix ? "\n" : ""}${snippet}${needsSuffix ? "\n" : ""}`;
                setCursorToEnd();
                break;
            }
            default:
                return;
        }

        // Use execCommand to preserve undo history
        textarea.focus();
        textarea.setSelectionRange(start, end);
        try {
            document.execCommand("insertText", false, replacement);
        } catch (e) {
            console.error(
                "execCommand failed, falling back to setRangeText",
                e,
            );
            textarea.setRangeText(replacement, start, end, "preserve");
        }

        textarea.dispatchEvent(new Event("input", { bubbles: true }));

        setTimeout(() => {
            textarea.focus();
            textarea.setSelectionRange(selectionStart, selectionEnd);
        }, 0);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.metaKey || e.ctrlKey) {
            if (e.shiftKey) {
                switch (e.code) {
                    case "Digit7":
                        e.preventDefault();
                        insertMarkdown("ordered_list");
                        return;
                    case "Digit8":
                        e.preventDefault();
                        insertMarkdown("list");
                        return;
                    case "Digit9":
                        e.preventDefault();
                        insertMarkdown("quote");
                        return;
                }
            }

            switch (e.key.toLowerCase()) {
                case "b":
                    e.preventDefault();
                    insertMarkdown("bold");
                    break;
                case "i":
                    e.preventDefault();
                    insertMarkdown("italic");
                    break;
                case "k":
                    e.preventDefault();
                    insertMarkdown("link");
                    break;
            }
        }
    }

    function downloadFile(
        content: Blob | string,
        fileName: string,
        contentType: string,
    ) {
        const a = document.createElement("a");
        const file =
            content instanceof Blob
                ? content
                : new Blob([content], { type: contentType });
        a.href = URL.createObjectURL(file);
        a.download = fileName;
        a.click();
        URL.revokeObjectURL(a.href);
    }

    function exportToMd() {
        if (!guide_markdown) return;
        downloadFile(
            guide_markdown,
            `${codelab_title}_Preparation_Guide.md`,
            "text/markdown",
        );
        showExportMenu = false;
    }

    async function exportToPdf() {
        if (!guide_markdown) return;
        const element = document.getElementById("guide-preview-content");
        if (!element) return;

        const opt = {
            margin: 1,
            filename: `${codelab_title}_Preparation_Guide.pdf`,
            image: { type: "jpeg", quality: 0.98 },
            html2canvas: { scale: 2 },
            jsPDF: { unit: "in", format: "letter", orientation: "portrait" },
        };

        try {
            await html2pdf().set(opt).from(element).save();
        } catch (e) {
            console.error("PDF Export failed", e);
            alert($t("editor.pdf_export_failed"));
        }
        showExportMenu = false;
    }

    async function exportToDocx() {
        if (!guide_markdown) return;

        try {
            // Basic HTML structure for DOCX
            const htmlString = `
                <!DOCTYPE html>
                <html>
                <head>
                    <meta charset="UTF-8">
                    <title>${codelab_title} Preparation Guide</title>
                    <style>
                        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; line-height: 1.6; }
                        h1 { color: rgb(32 33 36); }
                        h2 { color: rgb(60 64 67); border-bottom: 1px solid rgb(238 238 238); padding-bottom: 5px; }
                        code { background-color: rgb(241 243 244); padding: 2px 4px; border-radius: 4px; font-family: monospace; }
                        pre { background-color: rgb(248 249 250); padding: 15px; border-radius: 8px; border: 1px solid rgb(232 234 237); }
                    </style>
                </head>
                <body>
                    <h1>${codelab_title} - Preparation Guide</h1>
                    ${renderedContent}
                </body>
                </html>
            `;

            const docxBlob = await asBlob(htmlString);

            downloadFile(
                docxBlob,
                `${codelab_title}_Preparation_Guide.docx`,
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            );
        } catch (e) {
            console.error("DOCX Export failed", e);
            alert($t("editor.docx_export_failed"));
        }
        showExportMenu = false;
    }
</script>

<div class="space-y-6" in:fade>
    <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4"
    >
        <div>
            <h2 class="text-2xl font-bold text-foreground dark:text-dark-text">
                {$t("editor.guide_tab")}
            </h2>
            <p class="text-muted-foreground dark:text-dark-text-muted text-sm">
                {$t("editor.guide_description")}
            </p>
        </div>

        <div class="flex flex-col items-start sm:items-end gap-2">
            <div class="flex flex-wrap items-center gap-2">
                <div class="relative">
                    <button
                        onclick={() => (showExportMenu = !showExportMenu)}
                        class="bg-white dark:bg-dark-surface hover:bg-muted dark:hover:bg-white/5 text-muted-foreground dark:text-dark-text-muted px-4 py-2.5 rounded-full flex items-center gap-2 transition-all border border-border dark:border-dark-border font-bold text-sm shadow-sm"
                    >
                        <FileDown size={18} />
                        {$t("common.export")}
                        <ChevronDown
                            size={14}
                            class="transition-transform {showExportMenu
                                ? 'rotate-180'
                                : ''}"
                        />
                    </button>

                    {#if showExportMenu}
                        <div
                            class="absolute right-0 mt-2 w-56 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-xl z-50 overflow-hidden"
                            transition:slide={{ duration: 200 }}
                        >
                            <div class="p-2 space-y-1">
                                <button
                                    onclick={exportToMd}
                                    class="w-full text-left px-4 py-3 hover:bg-muted dark:hover:bg-white/5 rounded-xl flex items-center gap-3 transition-colors text-sm font-medium text-foreground dark:text-dark-text"
                                >
                                    <FileCode
                                        size={18}
                                        class="text-muted-foreground"
                                    />
                                    <span>Markdown (.md)</span>
                                </button>
                                <button
                                    onclick={exportToPdf}
                                    class="w-full text-left px-4 py-3 hover:bg-muted dark:hover:bg-white/5 rounded-xl flex items-center gap-3 transition-colors text-sm font-medium text-foreground dark:text-dark-text"
                                >
                                    <FileText size={18} class="text-red-500" />
                                    <span>PDF Document (.pdf)</span>
                                </button>
                                <button
                                    onclick={exportToDocx}
                                    class="w-full text-left px-4 py-3 hover:bg-muted dark:hover:bg-white/5 rounded-xl flex items-center gap-3 transition-colors text-sm font-medium text-foreground dark:text-dark-text"
                                >
                                    <FileType size={18} class="text-primary" />
                                    <span>MS Word / Google Docs (.docx)</span>
                                </button>
                            </div>
                        </div>
                    {/if}
                </div>

                <button
                    onclick={generateGuideWithAi}
                    disabled={isGenerating || isGuideProGenerating}
                    class="bg-primary hover:bg-primary/90 text-white px-5 py-2.5 rounded-full flex items-center gap-2 transition-all font-bold text-sm shadow-sm disabled:opacity-50"
                >
                    {#if isGenerating}
                        <Loader2 size={18} class="animate-spin" />
                        {$t("editor.generating_guide")}
                    {:else}
                        <Sparkles size={18} />
                        {$t("editor.generate_guide")}
                    {/if}
                </button>

                <button
                    onclick={handleProGenerate}
                    disabled={isGenerating || isGuideProGenerating}
                    class="bg-foreground hover:bg-foreground/90 text-background px-5 py-2.5 rounded-full flex items-center gap-2 transition-all font-bold text-sm shadow-sm disabled:opacity-50"
                >
                    {#if isGuideProGenerating}
                        <Loader2 size={18} class="animate-spin" />
                        {$t("editor.generating_guide_pro")}
                    {:else}
                        <Terminal size={18} />
                        {$t("editor.generate_guide_pro")}
                    {/if}
                </button>
                {#if guideProHasOutput || isGuideProGenerating}
                    <button
                        onclick={openGuideProModal}
                        class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border text-muted-foreground dark:text-dark-text-muted px-4 py-2.5 rounded-full flex items-center gap-2 transition-all font-bold text-xs hover:border-primary hover:text-primary"
                    >
                        {$t("editor.guide_pro_view_results")}
                    </button>
                {/if}
            </div>

            {#if isGuideProGenerating}
                <div
                    class="flex flex-wrap items-center gap-2 text-[10px] text-muted-foreground dark:text-dark-text-muted"
                >
                    {#each guideProStages as stage, index}
                        <span
                            class="px-2 py-1 rounded-full border text-[10px] font-bold uppercase tracking-wider {index <
                            guideProStageIndex
                                ? 'bg-accent/70 text-primary border-primary/30'
                                : index === guideProStageIndex
                                  ? 'bg-primary text-white border-primary'
                                  : 'bg-white dark:bg-dark-surface border-border dark:border-dark-border'}"
                        >
                            {$t(guideProStageLabels[stage])}
                        </span>
                        {#if index < guideProStages.length - 1}
                            <span
                                class="text-muted-foreground/40 dark:text-dark-border"
                                >&gt;</span
                            >
                        {/if}
                    {/each}
                </div>
            {/if}
        </div>
    </div>

    {#if showGuideProModal}
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
            role="dialog"
            tabindex="-1"
            aria-modal="true"
            aria-labelledby="guide-pro-modal-title"
            onclick={() => (showGuideProModal = false)}
            onkeydown={(e) => e.key === "Escape" && (showGuideProModal = false)}
            in:fade
        >
            <div
                class="bg-white dark:bg-dark-surface w-full max-w-5xl h-[85vh] rounded-3xl shadow-2xl border border-border dark:border-dark-border flex flex-col overflow-hidden"
                role="none"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
            >
                <div
                    class="flex items-center justify-between px-6 py-4 bg-foreground text-background"
                >
                    <div>
                        <h3
                            id="guide-pro-modal-title"
                            class="text-lg font-bold"
                        >
                            {$t("editor.guide_pro_modal_title")}
                        </h3>
                        <p class="text-xs opacity-80">
                            {$t("editor.guide_pro_modal_desc")}
                        </p>
                    </div>
                    <button
                        onclick={() => (showGuideProModal = false)}
                        class="p-2 hover:bg-white/10 rounded-full transition-colors"
                        aria-label={$t("common.close") || "Close"}
                    >
                        <X size={18} />
                    </button>
                </div>
                <div
                    class="flex-1 overflow-y-auto p-6 bg-muted dark:bg-dark-bg"
                >
                    <div
                        class="flex flex-wrap items-center gap-2 mb-4 text-[10px] text-muted-foreground dark:text-dark-text-muted"
                    >
                        {#each guideProStages as stage, index}
                            <span
                                class="px-2 py-1 rounded-full border text-[10px] font-bold uppercase tracking-wider {index <
                                guideProStageIndex
                                    ? 'bg-accent/70 text-primary border-primary/30'
                                    : index === guideProStageIndex
                                      ? 'bg-primary text-white border-primary'
                                      : 'bg-white dark:bg-dark-surface border-border dark:border-dark-border'}"
                            >
                                {$t(guideProStageLabels[stage])}
                            </span>
                            {#if index < guideProStages.length - 1}
                                <span
                                    class="text-muted-foreground/40 dark:text-dark-border"
                                    >&gt;</span
                                >
                            {/if}
                        {/each}
                    </div>

                    <div
                        class="rounded-2xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface/60 p-4 shadow-sm"
                    >
                        <div
                            class="flex flex-wrap items-center justify-between gap-3 mb-3"
                        >
                            <div>
                                <div
                                    class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("editor.guide_pro_results")}
                                </div>
                                <p
                                    class="text-xs text-muted-foreground dark:text-dark-text-muted"
                                >
                                    {$t("editor.guide_pro_results_desc")}
                                </p>
                            </div>
                            {#if isGuideProGenerating}
                                <span
                                    class="text-[10px] font-bold text-primary uppercase tracking-wider"
                                >
                                    {$t("editor.generating_guide_pro")}
                                </span>
                            {/if}
                        </div>
                        <div class="grid gap-3">
                            <details open={guideProDisplayStage === "plan"}>
                                <summary
                                    class="cursor-pointer list-none flex items-center justify-between rounded-xl border border-border dark:border-dark-border px-3 py-2 text-xs font-bold text-foreground dark:text-dark-text bg-muted dark:bg-dark-bg"
                                >
                                    <span
                                        >{$t(
                                            "editor.guide_pro_result_plan",
                                        )}</span
                                    >
                                </summary>
                                <div
                                    class="mt-2 rounded-xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface p-3 max-h-64 overflow-y-auto"
                                >
                                    {#if guideProPlanOutput}
                                        <pre
                                            class="text-[11px] leading-relaxed font-mono whitespace-pre-wrap text-foreground dark:text-dark-text">{guideProPlanOutput}</pre>
                                    {:else}
                                        <p
                                            class="text-[11px] text-muted-foreground/80 dark:text-dark-text-muted"
                                        >
                                            {$t(
                                                "editor.guide_pro_result_empty",
                                            )}
                                        </p>
                                    {/if}
                                </div>
                            </details>

                            <details open={guideProDisplayStage === "draft"}>
                                <summary
                                    class="cursor-pointer list-none flex items-center justify-between rounded-xl border border-border dark:border-dark-border px-3 py-2 text-xs font-bold text-foreground dark:text-dark-text bg-muted dark:bg-dark-bg"
                                >
                                    <span
                                        >{$t(
                                            "editor.guide_pro_result_draft",
                                        )}</span
                                    >
                                </summary>
                                <div
                                    class="mt-2 rounded-xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface p-3 max-h-64 overflow-y-auto"
                                >
                                    <div
                                        class="flex items-center justify-end gap-1 mb-3"
                                    >
                                        <button
                                            onclick={() =>
                                                (guideProDraftView =
                                                    "markdown")}
                                            class="px-2 py-1 rounded-full text-[10px] font-bold border {guideProDraftView ===
                                            'markdown'
                                                ? 'bg-primary text-white border-primary'
                                                : 'bg-white dark:bg-dark-surface text-muted-foreground dark:text-dark-text-muted border-border dark:border-dark-border'}"
                                        >
                                            {$t(
                                                "editor.guide_pro_view_markdown",
                                            )}
                                        </button>
                                        <button
                                            onclick={() =>
                                                (guideProDraftView = "raw")}
                                            class="px-2 py-1 rounded-full text-[10px] font-bold border {guideProDraftView ===
                                            'raw'
                                                ? 'bg-primary text-white border-primary'
                                                : 'bg-white dark:bg-dark-surface text-muted-foreground dark:text-dark-text-muted border-border dark:border-dark-border'}"
                                        >
                                            {$t("editor.guide_pro_view_raw")}
                                        </button>
                                    </div>
                                    {#if guideProDraftOutput}
                                        {#if guideProDraftView === "markdown"}
                                            <div class="markdown-body text-sm">
                                                {@html guideProDraftHtml}
                                            </div>
                                        {:else}
                                            <pre
                                                class="text-[11px] leading-relaxed font-mono whitespace-pre-wrap text-foreground dark:text-dark-text">{guideProDraftOutput}</pre>
                                        {/if}
                                    {:else}
                                        <p
                                            class="text-[11px] text-muted-foreground/80 dark:text-dark-text-muted"
                                        >
                                            {$t(
                                                "editor.guide_pro_result_empty",
                                            )}
                                        </p>
                                    {/if}
                                </div>
                            </details>

                            <details open={guideProDisplayStage === "review"}>
                                <summary
                                    class="cursor-pointer list-none flex items-center justify-between rounded-xl border border-border dark:border-dark-border px-3 py-2 text-xs font-bold text-foreground dark:text-dark-text bg-muted dark:bg-dark-bg"
                                >
                                    <span
                                        >{$t(
                                            "editor.guide_pro_result_review",
                                        )}</span
                                    >
                                </summary>
                                <div
                                    class="mt-2 rounded-xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface p-3 max-h-64 overflow-y-auto"
                                >
                                    {#if guideProReviewOutput}
                                        <pre
                                            class="text-[11px] leading-relaxed font-mono whitespace-pre-wrap text-foreground dark:text-dark-text">{guideProReviewOutput}</pre>
                                    {:else}
                                        <p
                                            class="text-[11px] text-muted-foreground/80 dark:text-dark-text-muted"
                                        >
                                            {$t(
                                                "editor.guide_pro_result_empty",
                                            )}
                                        </p>
                                    {/if}
                                </div>
                            </details>

                            <details open={guideProDisplayStage === "revise"}>
                                <summary
                                    class="cursor-pointer list-none flex items-center justify-between rounded-xl border border-border dark:border-dark-border px-3 py-2 text-xs font-bold text-foreground dark:text-dark-text bg-muted dark:bg-dark-bg"
                                >
                                    <span
                                        >{$t(
                                            "editor.guide_pro_result_revise",
                                        )}</span
                                    >
                                </summary>
                                <div
                                    class="mt-2 rounded-xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface p-3 max-h-64 overflow-y-auto"
                                >
                                    <div
                                        class="flex items-center justify-end gap-1 mb-3"
                                    >
                                        <button
                                            onclick={() =>
                                                (guideProRevisedView =
                                                    "markdown")}
                                            class="px-2 py-1 rounded-full text-[10px] font-bold border {guideProRevisedView ===
                                            'markdown'
                                                ? 'bg-primary text-white border-primary'
                                                : 'bg-white dark:bg-dark-surface text-muted-foreground dark:text-dark-text-muted border-border dark:border-dark-border'}"
                                        >
                                            {$t(
                                                "editor.guide_pro_view_markdown",
                                            )}
                                        </button>
                                        <button
                                            onclick={() =>
                                                (guideProRevisedView = "raw")}
                                            class="px-2 py-1 rounded-full text-[10px] font-bold border {guideProRevisedView ===
                                            'raw'
                                                ? 'bg-primary text-white border-primary'
                                                : 'bg-white dark:bg-dark-surface text-muted-foreground dark:text-dark-text-muted border-border dark:border-dark-border'}"
                                        >
                                            {$t("editor.guide_pro_view_raw")}
                                        </button>
                                    </div>
                                    {#if guideProRevisedOutput}
                                        {#if guideProRevisedView === "markdown"}
                                            <div class="markdown-body text-sm">
                                                {@html guideProRevisedHtml}
                                            </div>
                                        {:else}
                                            <pre
                                                class="text-[11px] leading-relaxed font-mono whitespace-pre-wrap text-foreground dark:text-dark-text">{guideProRevisedOutput}</pre>
                                        {/if}
                                    {:else}
                                        <p
                                            class="text-[11px] text-muted-foreground/80 dark:text-dark-text-muted"
                                        >
                                            {$t(
                                                "editor.guide_pro_result_empty",
                                            )}
                                        </p>
                                    {/if}
                                </div>
                            </details>
                        </div>
                    </div>

                    <div
                        class="mt-6 rounded-2xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface/60 p-4 shadow-sm"
                    >
                        <div
                            class="flex flex-wrap items-center justify-between gap-3 mb-3"
                        >
                            <div>
                                <div
                                    class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
                                >
                                    {$t("editor.guide_pro_diff_title")}
                                </div>
                                <p
                                    class="text-xs text-muted-foreground dark:text-dark-text-muted"
                                >
                                    {$t("editor.guide_pro_diff_desc")}
                                </p>
                            </div>
                            <div class="flex items-center gap-1">
                                <button
                                    onclick={() =>
                                        (guideProDiffView = "unified")}
                                    class="px-2 py-1 rounded-full text-[10px] font-bold border {guideProDiffView ===
                                    'unified'
                                        ? 'bg-primary text-white border-primary'
                                        : 'bg-white dark:bg-dark-surface text-muted-foreground dark:text-dark-text-muted border-border dark:border-dark-border'}"
                                >
                                    {$t("editor.guide_pro_diff_view_unified")}
                                </button>
                                <button
                                    onclick={() => (guideProDiffView = "split")}
                                    class="px-2 py-1 rounded-full text-[10px] font-bold border {guideProDiffView ===
                                    'split'
                                        ? 'bg-primary text-white border-primary'
                                        : 'bg-white dark:bg-dark-surface text-muted-foreground dark:text-dark-text-muted border-border dark:border-dark-border'}"
                                >
                                    {$t("editor.guide_pro_diff_view_split")}
                                </button>
                            </div>
                        </div>
                        {#if !guideProDraftOutput || !guideProRevisedOutput}
                            <p
                                class="text-xs text-muted-foreground/80 dark:text-dark-text-muted"
                            >
                                {$t("editor.guide_pro_diff_empty")}
                            </p>
                        {:else if guideProDiff.truncated}
                            <p
                                class="text-xs text-muted-foreground/80 dark:text-dark-text-muted"
                            >
                                {$t("editor.guide_pro_diff_too_large")}
                            </p>
                        {:else if guideProDiffView === "unified"}
                            <div
                                class="rounded-xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface max-h-72 overflow-y-auto"
                            >
                                <div
                                    class="font-mono text-[11px] leading-relaxed"
                                >
                                    {#each guideProDiff.lines as line}
                                        <div
                                            class="flex items-start gap-2 px-3 py-1 {line.type ===
                                            'add'
                                                ? 'bg-emerald-50 text-emerald-700'
                                                : line.type === 'remove'
                                                  ? 'bg-red-50 text-red-600'
                                                  : 'text-foreground dark:text-dark-text'}"
                                        >
                                            <span
                                                class="w-4 text-[10px] font-bold"
                                            >
                                                {line.type === "add"
                                                    ? "+"
                                                    : line.type === "remove"
                                                      ? "-"
                                                      : " "}
                                            </span>
                                            <span
                                                class="whitespace-pre-wrap break-words flex-1"
                                            >
                                                {line.text || " "}
                                            </span>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {:else}
                            <div
                                class="rounded-xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface max-h-72 overflow-y-auto"
                            >
                                <div
                                    class="grid grid-cols-2 font-mono text-[11px] leading-relaxed"
                                >
                                    <div
                                        class="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground dark:text-dark-text-muted bg-muted dark:bg-dark-bg border-b border-border dark:border-dark-border"
                                    >
                                        {$t("editor.guide_pro_diff_left")}
                                    </div>
                                    <div
                                        class="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground dark:text-dark-text-muted bg-muted dark:bg-dark-bg border-b border-l border-border dark:border-dark-border"
                                    >
                                        {$t("editor.guide_pro_diff_right")}
                                    </div>
                                    {#each guideProDiffRows as row}
                                        <div
                                            class="flex items-start gap-2 px-3 py-1 border-r border-border dark:border-dark-border {row.leftType ===
                                            'remove'
                                                ? 'bg-red-50 text-red-600'
                                                : 'text-foreground dark:text-dark-text'}"
                                        >
                                            <span
                                                class="w-4 text-[10px] font-bold"
                                            >
                                                {row.leftType === "remove"
                                                    ? "-"
                                                    : " "}
                                            </span>
                                            <span
                                                class="whitespace-pre-wrap break-words flex-1"
                                            >
                                                {row.leftText || " "}
                                            </span>
                                        </div>
                                        <div
                                            class="flex items-start gap-2 px-3 py-1 {row.rightType ===
                                            'add'
                                                ? 'bg-emerald-50 text-emerald-700'
                                                : 'text-foreground dark:text-dark-text'}"
                                        >
                                            <span
                                                class="w-4 text-[10px] font-bold"
                                            >
                                                {row.rightType === "add"
                                                    ? "+"
                                                    : " "}
                                            </span>
                                            <span
                                                class="whitespace-pre-wrap break-words flex-1"
                                            >
                                                {row.rightText || " "}
                                            </span>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {/if}
                    </div>
                </div>
            </div>
        </div>
    {/if}

    <div
        class="flex flex-col gap-2 p-2 bg-muted/90 dark:bg-white/5 backdrop-blur-sm rounded-2xl border border-border dark:border-dark-border"
    >
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
                <div
                    class="flex items-center gap-1 rounded-lg border border-border dark:border-dark-border bg-white/80 dark:bg-dark-surface/60 px-2 py-1"
                >
                    <span
                        class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
                    >
                        {$t("editor.toolbar.code_language")}
                    </span>
                    <select
                        bind:value={codeLanguage}
                        class="bg-transparent text-xs font-bold text-foreground dark:text-dark-text outline-none"
                        aria-label={$t("editor.toolbar.code_language")}
                    >
                        <option value="">({$t("editor.toolbar.auto")})</option>
                        {#each languageOptions as language}
                            <option value={language.value}
                                >{language.label}</option
                            >
                        {/each}
                    </select>
                </div>
                <button
                    onclick={() =>
                        insertMarkdown("code_block", {
                            language: codeLanguage,
                        })}
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
                    class="flex items-center gap-2 px-3 py-2 rounded-full transition-colors border border-border dark:border-dark-border {isSplitView
                        ? 'text-primary bg-white dark:bg-white/10 shadow-sm'
                        : 'text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:bg-white dark:hover:bg-white/10'}"
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
                        class="list-none cursor-pointer flex items-center gap-2 px-3 py-2 rounded-full border border-border dark:border-dark-border text-muted-foreground dark:text-dark-text-muted hover:text-primary hover:bg-white dark:hover:bg-white/10 transition-colors"
                        title={$t("editor.markdown_cheatsheet")}
                        aria-label={$t("editor.markdown_cheatsheet")}
                    >
                        <BookOpen size={18} />
                        <span class="hidden sm:inline text-xs font-bold">
                            {$t("editor.markdown_cheatsheet")}
                        </span>
                    </summary>
                    <div
                        class="absolute right-0 mt-2 w-80 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-xl p-4 text-xs text-foreground dark:text-dark-text z-30"
                    >
                        <div
                            class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider mb-2"
                        >
                            {$t("editor.markdown_cheatsheet")}
                        </div>
                        <pre
                            class="font-mono text-[11px] leading-relaxed bg-muted dark:bg-dark-bg rounded-lg p-3 border border-border dark:border-dark-border whitespace-pre-wrap"># {$t(
                                "editor.toolbar.heading1",
                            )}
**{$t("editor.toolbar.bold")}** *{$t("editor.toolbar.italic")}* `{$t(
                                "editor.toolbar.inline_code",
                            )}`
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
            <span
                class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
            >
                {$t("editor.snippets.label")}
            </span>
            <button
                onclick={() =>
                    insertMarkdown("snippet", {
                        snippet: $t("editor.snippets.step_outline"),
                    })}
                class={quickBlockClass}
            >
                <ListOrdered size={12} />
                <span>{$t("editor.snippets.step_outline_label")}</span>
            </button>
            <button
                onclick={() =>
                    insertMarkdown("snippet", {
                        snippet: $t("editor.snippets.checklist"),
                    })}
                class={quickBlockClass}
            >
                <CheckSquare size={12} />
                <span>{$t("editor.snippets.checklist_label")}</span>
            </button>
            <button
                onclick={() =>
                    insertMarkdown("snippet", {
                        snippet: $t("editor.snippets.callout"),
                    })}
                class={quickBlockClass}
            >
                <Quote size={12} />
                <span>{$t("editor.snippets.callout_label")}</span>
            </button>
            <button
                onclick={() =>
                    insertMarkdown("snippet", {
                        snippet: $t("editor.snippets.command_block"),
                    })}
                class={quickBlockClass}
            >
                <Terminal size={12} />
                <span>{$t("editor.snippets.command_block_label")}</span>
            </button>
        </div>
    </div>

    <div
        class="grid grid-cols-1 {isSplitView
            ? 'lg:grid-cols-2'
            : ''} gap-8 min-h-[60vh]"
    >
        <!-- Editor Side -->
        <div class="flex flex-col gap-3">
            <div class="flex items-center justify-between px-2">
                <span
                    class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
                >
                    {$t("editor.markdown_editor")}
                </span>
            </div>
            <textarea
                bind:this={editorEl}
                bind:value={guide_markdown}
                onscroll={syncEditorScroll}
                onkeydown={handleKeydown}
                placeholder={$t("editor.guide_placeholder")}
                class="flex-1 w-full p-6 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl font-mono text-sm sm:text-base leading-relaxed focus:ring-4 focus:ring-primary/10 outline-none resize-none transition-all text-foreground dark:text-dark-text"
            ></textarea>
        </div>

        <!-- Preview Side -->
        {#if isSplitView}
            <div class="flex flex-col gap-3">
                <div class="flex items-center justify-between px-2">
                    <span
                        class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
                    >
                        {$t("editor.live_preview")}
                    </span>
                </div>
                <div
                    bind:this={previewEl}
                    onscroll={syncPreviewScroll}
                    class="flex-1 w-full p-6 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl overflow-y-auto prose dark:prose-invert max-w-none shadow-inner"
                >
                    {#if guide_markdown}
                        <div class="markdown-body" id="guide-preview-content">
                            {@html renderedContent}
                        </div>
                    {:else}
                        <div
                            class="h-full flex flex-col items-center justify-center text-center p-8 opacity-40"
                        >
                            <Info size={48} class="mb-4" />
                            <p>{$t("editor.guide_empty_preview")}</p>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}
    </div>

    <div
        class="mt-4 flex flex-wrap items-center justify-between gap-2 text-[11px] text-muted-foreground dark:text-dark-text-muted"
    >
        <div class="flex items-center gap-3">
            <span
                >{$t("editor.stats.words", {
                    values: { count: wordCount },
                })}</span
            >
            <span
                >{$t("editor.stats.chars", {
                    values: { count: charCount },
                })}</span
            >
            <span
                >{$t("editor.stats.lines", {
                    values: { count: lineCount },
                })}</span
            >
        </div>
        <div
            class="hidden xl:flex items-center gap-2 text-muted-foreground/80 dark:text-dark-text-muted"
        >
            <span>{$t("editor.shortcut_hint")}</span>
        </div>
    </div>
</div>
