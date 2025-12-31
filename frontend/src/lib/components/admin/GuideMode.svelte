<script lang="ts">
    import { fade, slide } from "svelte/transition";
    import { Sparkles, Loader2, Wand2, Info, FileDown, FileText, FileCode, FileType, ChevronDown, Printer } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { adminMarked as marked } from "$lib/markdown";
    import DOMPurify from "dompurify";
    import { browser } from "$app/environment";
    import html2pdf from "html2pdf.js";
    // @ts-ignore
    import { asBlob } from "html-docx-js-typescript";

    let { 
        guide_markdown = $bindable(), 
        codelab_title = "Codelab Guide",
        isSaving, 
        handleSave,
        generateGuideWithAi,
        isGenerating
    } = $props<{
        guide_markdown: string;
        codelab_title?: string;
        isSaving: boolean;
        handleSave: () => void;
        generateGuideWithAi: () => void;
        isGenerating: boolean;
    }>();

    let showExportMenu = $state(false);

    let renderedContent = $derived.by(() => {
        if (!guide_markdown) return "";
        try {
            const html = marked.parse(guide_markdown) as string;
            if (browser) {
                return DOMPurify.sanitize(html);
            }
            return html;
        } catch (e) {
            console.error("Markdown parse error", e);
            return "Error parsing markdown";
        }
    });

    function downloadFile(content: Blob | string, fileName: string, contentType: string) {
        const a = document.createElement("a");
        const file = content instanceof Blob ? content : new Blob([content], { type: contentType });
        a.href = URL.createObjectURL(file);
        a.download = fileName;
        a.click();
        URL.revokeObjectURL(a.href);
    }

    function exportToMd() {
        if (!guide_markdown) return;
        downloadFile(guide_markdown, `${codelab_title}_Preparation_Guide.md`, "text/markdown");
        showExportMenu = false;
    }

    async function exportToPdf() {
        if (!guide_markdown) return;
        const element = document.getElementById("guide-preview-content");
        if (!element) return;

        const opt = {
            margin: 1,
            filename: `${codelab_title}_Preparation_Guide.pdf`,
            image: { type: 'jpeg', quality: 0.98 },
            html2canvas: { scale: 2 },
            jsPDF: { unit: 'in', format: 'letter', orientation: 'portrait' }
        };

        try {
            await html2pdf().set(opt).from(element).save();
        } catch (e) {
            console.error("PDF Export failed", e);
            alert("PDF Export failed");
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
                        h1 { color: #202124; }
                        h2 { color: #3C4043; border-bottom: 1px solid #eee; padding-bottom: 5px; }
                        code { background-color: #f1f3f4; padding: 2px 4px; border-radius: 4px; font-family: monospace; }
                        pre { background-color: #f8f9fa; padding: 15px; border-radius: 8px; border: 1px solid #e8eaed; }
                    </style>
                </head>
                <body>
                    <h1>${codelab_title} - Preparation Guide</h1>
                    ${renderedContent}
                </body>
                </html>
            `;

            const docxBlob = await asBlob(htmlString);

            downloadFile(docxBlob, `${codelab_title}_Preparation_Guide.docx`, "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        } catch (e) {
            console.error("DOCX Export failed", e);
            alert("DOCX Export failed. Try copying the content directly to MS Word.");
        }
        showExportMenu = false;
    }
</script>

<div class="space-y-6" in:fade>
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
            <h2 class="text-2xl font-bold text-[#202124] dark:text-dark-text">{$t("editor.guide_tab")}</h2>
            <p class="text-[#5F6368] dark:text-dark-text-muted text-sm">{$t("editor.guide_description")}</p>
        </div>
        
        <div class="flex items-center gap-2">
            <div class="relative">
                <button
                    onclick={() => (showExportMenu = !showExportMenu)}
                    class="bg-white dark:bg-dark-surface hover:bg-[#F8F9FA] dark:hover:bg-white/5 text-[#5F6368] dark:text-dark-text-muted px-4 py-2.5 rounded-full flex items-center gap-2 transition-all border border-[#DADCE0] dark:border-dark-border font-bold text-sm shadow-sm"
                >
                    <FileDown size={18} />
                    {$t("common.export")}
                    <ChevronDown size={14} class="transition-transform {showExportMenu ? 'rotate-180' : ''}" />
                </button>

                {#if showExportMenu}
                    <div 
                        class="absolute right-0 mt-2 w-56 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-xl z-50 overflow-hidden"
                        transition:slide={{ duration: 200 }}
                    >
                        <div class="p-2 space-y-1">
                            <button
                                onclick={exportToMd}
                                class="w-full text-left px-4 py-3 hover:bg-[#F8F9FA] dark:hover:bg-white/5 rounded-xl flex items-center gap-3 transition-colors text-sm font-medium text-[#3C4043] dark:text-dark-text"
                            >
                                <FileCode size={18} class="text-[#5F6368]" />
                                <span>Markdown (.md)</span>
                            </button>
                            <button
                                onclick={exportToPdf}
                                class="w-full text-left px-4 py-3 hover:bg-[#F8F9FA] dark:hover:bg-white/5 rounded-xl flex items-center gap-3 transition-colors text-sm font-medium text-[#3C4043] dark:text-dark-text"
                            >
                                <FileText size={18} class="text-[#EA4335]" />
                                <span>PDF Document (.pdf)</span>
                            </button>
                            <button
                                onclick={exportToDocx}
                                class="w-full text-left px-4 py-3 hover:bg-[#F8F9FA] dark:hover:bg-white/5 rounded-xl flex items-center gap-3 transition-colors text-sm font-medium text-[#3C4043] dark:text-dark-text"
                            >
                                <FileType size={18} class="text-[#4285F4]" />
                                <span>MS Word / Google Docs (.docx)</span>
                            </button>
                        </div>
                    </div>
                {/if}
            </div>

            <button
                onclick={generateGuideWithAi}
                disabled={isGenerating}
                class="bg-white dark:bg-dark-surface hover:bg-[#F8F9FA] dark:hover:bg-white/5 text-[#8E24AA] px-5 py-2.5 rounded-full flex items-center gap-2 transition-all border border-[#DADCE0] dark:border-dark-border font-bold text-sm shadow-sm disabled:opacity-50"
            >
                {#if isGenerating}
                    <Loader2 size={18} class="animate-spin" />
                    {$t("editor.generating_guide")}
                {:else}
                    <Sparkles size={18} />
                    {$t("editor.generate_guide")}
                {/if}
            </button>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 h-[calc(100vh-350px)]">
        <!-- Editor Side -->
        <div class="flex flex-col gap-3">
            <div class="flex items-center justify-between px-2">
                <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">Markdown Editor</span>
            </div>
            <textarea
                bind:value={guide_markdown}
                placeholder={$t("editor.guide_placeholder")}
                class="flex-1 w-full p-6 bg-[#F8F9FA] dark:bg-dark-bg border border-[#E8EAED] dark:border-dark-border rounded-2xl font-mono text-sm focus:ring-4 focus:ring-[#4285F4]/10 outline-none resize-none transition-all dark:text-dark-text"
            ></textarea>
        </div>

        <!-- Preview Side -->
        <div class="flex flex-col gap-3">
            <div class="flex items-center justify-between px-2">
                <span class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">Live Preview</span>
            </div>
            <div class="flex-1 w-full p-8 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-y-auto prose dark:prose-invert max-w-none shadow-inner">
                {#if guide_markdown}
                    <div class="markdown-body" id="guide-preview-content">
                        {@html renderedContent}
                    </div>
                {:else}
                    <div class="h-full flex flex-col items-center justify-center text-center p-8 opacity-40">
                        <Info size={48} class="mb-4" />
                        <p>{$t("editor.guide_empty_preview")}</p>
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

