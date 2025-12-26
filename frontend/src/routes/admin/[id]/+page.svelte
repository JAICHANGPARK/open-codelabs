<script lang="ts">
    import { onMount } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import {
        getCodelab,
        saveSteps,
        exportCodelab,
        type Codelab,
        type Step,
    } from "$lib/api";
    // @ts-ignore
    import QRCode from "svelte-qrcode";
    import { marked } from "marked";
    import DOMPurify from "dompurify";
    import {
        ChevronLeft,
        Save,
        Plus,
        Trash2,
        Eye,
        Edit3,
        ExternalLink,
        CheckCircle2,
        Download,
        Code,
        Image as ImageIcon,
        Bold,
        Italic,
        List,
        Heading1,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let id = page.params.id as string;
    let codelab: Codelab | null = null;
    let steps: Step[] = [];
    let loading = true;
    let activeStepIndex = 0;
    let mode: "edit" | "preview" = "edit";
    let isSaving = false;
    let saveSuccess = false;

    onMount(async () => {
        try {
            const data = await getCodelab(id);
            codelab = data[0];
            steps = data[1];
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    function addStep() {
        const newStep: Step = {
            id: "",
            codelab_id: id,
            step_number: steps.length + 1,
            title: $t("editor.untitled_step"),
            content_markdown: `# ${$t("editor.untitled_step")}\n\nStart writing here...`,
        };
        steps = [...steps, newStep];
        activeStepIndex = steps.length - 1;
    }

    async function handleSave() {
        if (isSaving) return;
        isSaving = true;
        try {
            await saveSteps(
                id,
                steps.map((s) => ({
                    title: s.title,
                    content_markdown: s.content_markdown,
                })),
            );
            saveSuccess = true;
            setTimeout(() => (saveSuccess = false), 3000);
        } catch (e) {
            alert("Save failed: " + e);
        } finally {
            isSaving = false;
        }
    }

    async function handleExport() {
        try {
            await exportCodelab(id);
        } catch (e) {
            alert("Export failed: " + e);
        }
    }

    function insertMarkdown(type: string) {
        if (mode !== "edit" || !steps[activeStepIndex]) return;

        const textarea = document.querySelector("textarea");
        if (!textarea) return;

        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = steps[activeStepIndex].content_markdown;
        const selected = text.substring(start, end);

        let replacement = "";
        let cursorOffset = 0;

        switch (type) {
            case "bold":
                replacement = `**${selected || "bold text"}**`;
                cursorOffset = selected ? 0 : 2;
                break;
            case "italic":
                replacement = `*${selected || "italic text"}*`;
                cursorOffset = selected ? 0 : 1;
                break;
            case "code":
                replacement = `\n\`\`\`javascript\n${selected || "// code here"}\n\`\`\`\n`;
                cursorOffset = selected ? 0 : 15;
                break;
            case "image":
                replacement = `![description](https://via.placeholder.com/600x400)`;
                cursorOffset = 2;
                break;
            case "h1":
                replacement = `# ${selected || "Heading"}`;
                cursorOffset = 0;
                break;
            case "list":
                replacement = `\n- ${selected || "list item"}`;
                cursorOffset = 0;
                break;
        }

        steps[activeStepIndex].content_markdown =
            text.substring(0, start) + replacement + text.substring(end);

        // Refocus and set cursor
        setTimeout(() => {
            textarea.focus();
            const newCursorPos = start + replacement.length - cursorOffset;
            textarea.setSelectionRange(newCursorPos, newCursorPos);
        }, 0);
    }

    function handlePaste(event: ClipboardEvent) {
        const items = event.clipboardData?.items;
        if (!items) return;

        for (const item of items) {
            if (item.type.indexOf("image") !== -1) {
                // For now, we just insert a placeholder or hint that image upload will be here
                // A full implementation would upload to a server and get a URL
                insertMarkdown("image");
                event.preventDefault();
            }
        }
    }

    $: currentStep = steps[activeStepIndex];
    // @ts-ignore
    $: renderedContent = currentStep
        ? (DOMPurify.sanitize(
              marked.parse(currentStep.content_markdown) as string,
          ) as string)
        : "";
</script>

<div class="min-h-screen bg-[#F8F9FA] flex flex-col font-sans text-[#3C4043]">
    <header
        class="bg-white border-b border-[#E8EAED] py-4 px-8 sticky top-0 z-30 shadow-sm"
    >
        <div class="max-w-7xl mx-auto flex justify-between items-center">
            <div class="flex items-center gap-6">
                <a
                    href="/admin"
                    class="text-[#5F6368] hover:text-[#202124] hover:bg-[#F1F3F4] p-2 rounded-full transition-all"
                    aria-label="Back to dashboard"
                >
                    <ChevronLeft size={24} />
                </a>
                <div>
                    {#if loading}
                        <div
                            class="h-6 w-48 bg-[#F1F3F4] animate-pulse rounded"
                        ></div>
                    {:else}
                        <h1
                            class="text-xl font-bold text-[#202124] flex items-center gap-2"
                        >
                            {codelab?.title}
                            <a
                                href="/codelabs/{id}"
                                target="_blank"
                                class="text-[#4285F4] hover:text-[#1A73E8]"
                                title={$t("editor.view_live")}
                            >
                                <ExternalLink size={16} />
                            </a>
                        </h1>
                        <p class="text-xs text-[#5F6368] font-medium mt-0.5">
                            ID: {id} &bull; {$t("common.facilitator")} Mode
                        </p>
                    {/if}
                </div>
            </div>
            <div class="flex items-center gap-4">
                <button
                    on:click={handleExport}
                    class="p-2.5 text-[#5F6368] hover:text-[#4285F4] hover:bg-[#E8F0FE] rounded-full transition-all"
                    title="Export Codelab"
                >
                    <Download size={24} />
                </button>
                <div
                    class="flex bg-[#F1F3F4] p-1 rounded-full border border-[#E8EAED]"
                >
                    <button
                        on:click={() => (mode = "edit")}
                        class="px-5 py-1.5 rounded-full flex items-center gap-2 text-sm font-bold transition-all {mode ===
                        'edit'
                            ? 'bg-white shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] hover:text-[#202124]'}"
                    >
                        <Edit3 size={16} /> Edit
                    </button>
                    <button
                        on:click={() => (mode = "preview")}
                        class="px-5 py-1.5 rounded-full flex items-center gap-2 text-sm font-bold transition-all {mode ===
                        'preview'
                            ? 'bg-white shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] hover:text-[#202124]'}"
                    >
                        <Eye size={16} /> Preview
                    </button>
                </div>
                <button
                    on:click={handleSave}
                    disabled={isSaving}
                    class="bg-[#4285F4] hover:bg-[#1A73E8] text-white px-6 py-2.5 rounded-full flex items-center gap-2 text-sm font-bold transition-all shadow-md active:scale-95 disabled:opacity-50 {saveSuccess
                        ? 'bg-[#1E8E3E]'
                        : ''}"
                >
                    {#if isSaving}
                        <div
                            class="w-4 h-4 border-2 border-white border-t-transparent animate-spin rounded-full"
                        ></div>
                    {:else if saveSuccess}
                        <CheckCircle2 size={18} />
                    {:else}
                        <Save size={18} />
                    {/if}
                    {saveSuccess
                        ? $t("editor.saved")
                        : $t("editor.save_content")}
                </button>
            </div>
        </div>
    </header>

    {#if loading}
        <div class="flex-1 flex justify-center items-center">
            <div
                class="animate-spin rounded-full h-12 w-12 border-4 border-[#E8EAED] border-t-[#4285F4]"
            ></div>
        </div>
    {:else}
        <main
            class="max-w-7xl mx-auto w-full p-8 flex-1 grid grid-cols-12 gap-8 items-start"
        >
            <!-- Sidebar Navigation -->
            <div class="col-span-3 sticky top-28">
                <div
                    class="bg-white rounded-2xl border border-[#E8EAED] overflow-hidden shadow-sm"
                >
                    <div
                        class="p-5 border-b border-[#F1F3F4] bg-[#F8F9FA] flex justify-between items-center"
                    >
                        <span
                            class="text-xs font-bold text-[#5F6368] uppercase tracking-widest"
                            >{$t("editor.step_navigation")}</span
                        >
                        <button
                            on:click={addStep}
                            class="text-[#4285F4] hover:bg-[#E8F0FE] p-1.5 rounded-full transition-colors"
                            title={$t("editor.add_step")}
                        >
                            <Plus size={18} />
                        </button>
                    </div>
                    <div class="max-h-[50vh] overflow-y-auto">
                        {#each steps as step, i}
                            <button
                                on:click={() => (activeStepIndex = i)}
                                class="w-full text-left px-5 py-4 hover:bg-[#F8F9FA] transition-all flex items-start gap-4 border-l-4 {activeStepIndex ===
                                i
                                    ? 'border-[#4285F4] bg-[#E8F0FE]/30'
                                    : 'border-transparent'}"
                            >
                                <span
                                    class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold shrink-0 {activeStepIndex ===
                                    i
                                        ? 'bg-[#4285F4] text-white'
                                        : 'bg-[#F1F3F4] text-[#5F6368]'}"
                                    >{i + 1}</span
                                >
                                <span
                                    class="text-sm font-bold {activeStepIndex ===
                                    i
                                        ? 'text-[#1967D2]'
                                        : 'text-[#5F6368]'} line-clamp-1 pt-0.5"
                                    >{step.title}</span
                                >
                            </button>
                        {/each}
                    </div>

                    <div
                        class="p-8 border-t border-[#F1F3F4] bg-[#F8F9FA]/50 flex flex-col items-center"
                    >
                        <div
                            class="bg-white p-3 rounded-2xl border border-[#E8EAED] shadow-sm mb-4"
                        >
                            <QRCode
                                value="{window.location.origin}/codelabs/{id}"
                                size={140}
                            />
                        </div>
                        <p
                            class="text-[11px] text-[#5F6368] text-center uppercase tracking-widest font-bold"
                        >
                            {$t("editor.attendee_access")}
                        </p>
                    </div>
                </div>
            </div>

            <!-- Content Area -->
            <div class="col-span-9" in:fade>
                {#if steps.length > 0}
                    <div
                        class="bg-white rounded-2xl border border-[#E8EAED] shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
                    >
                        <div
                            class="p-8 border-b border-[#F1F3F4] bg-[#F8F9FA]/30"
                        >
                            <input
                                type="text"
                                bind:value={steps[activeStepIndex].title}
                                class="text-3xl font-bold text-[#202124] w-full bg-transparent outline-none placeholder-[#DADCE0] border-b-2 border-transparent focus:border-[#4285F4] transition-all pb-2"
                                placeholder="Untitled Step"
                            />
                        </div>

                        <div class="flex-1 p-8 flex flex-col">
                            {#if mode === "edit"}
                                <div
                                    class="flex items-center gap-2 mb-4 p-2 bg-[#F8F9FA] rounded-xl border border-[#E8EAED]"
                                >
                                    <button
                                        on:click={() => insertMarkdown("h1")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Heading"
                                        ><Heading1 size={20} /></button
                                    >
                                    <button
                                        on:click={() => insertMarkdown("bold")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Bold"><Bold size={20} /></button
                                    >
                                    <button
                                        on:click={() =>
                                            insertMarkdown("italic")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Italic"
                                        ><Italic size={20} /></button
                                    >
                                    <div
                                        class="w-px h-6 bg-[#DADCE0] mx-1"
                                    ></div>
                                    <button
                                        on:click={() => insertMarkdown("list")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="List"><List size={20} /></button
                                    >
                                    <button
                                        on:click={() => insertMarkdown("code")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Code Block"
                                        ><Code size={20} /></button
                                    >
                                    <button
                                        on:click={() => insertMarkdown("image")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Image"
                                        ><ImageIcon size={20} /></button
                                    >
                                </div>
                                <textarea
                                    bind:value={
                                        steps[activeStepIndex].content_markdown
                                    }
                                    on:paste={handlePaste}
                                    class="w-full flex-1 min-h-[50vh] outline-none text-[#3C4043] font-mono text-base leading-relaxed resize-none bg-transparent"
                                    placeholder="Write your markdown here..."
                                ></textarea>
                            {:else}
                                <div
                                    class="prose max-w-none text-[#3C4043]"
                                    in:fade
                                >
                                    <div class="markdown-body">
                                        {@html renderedContent}
                                    </div>
                                </div>
                            {/if}
                        </div>
                    </div>
                {:else}
                    <div
                        class="bg-white rounded-3xl border-2 border-dashed border-[#DADCE0] p-24 text-center"
                        in:fly={{ y: 20 }}
                    >
                        <div
                            class="w-20 h-20 bg-[#F1F3F4] rounded-full flex items-center justify-center mx-auto mb-8"
                        >
                            <Plus size={40} class="text-[#BDC1C6]" />
                        </div>
                        <h3 class="text-2xl font-bold text-[#202124] mb-3">
                            {$t("editor.empty_codelab")}
                        </h3>
                        <p
                            class="text-[#5F6368] text-lg mb-10 max-w-sm mx-auto"
                        >
                            {$t("editor.empty_desc")}
                        </p>
                        <button
                            on:click={addStep}
                            class="bg-[#4285F4] text-white px-10 py-3 rounded-full font-bold flex items-center gap-2 mx-auto shadow-md hover:shadow-lg transition-all active:scale-95"
                        >
                            <Plus size={20} />
                            {$t("editor.add_first_step")}
                        </button>
                    </div>
                {/if}
            </div>
        </main>
    {/if}
</div>

<style>
    :global(.markdown-body) {
        font-size: 1.1rem;
        line-height: 1.6;
    }
    :global(.markdown-body pre) {
        background-color: #f8f9fa;
        border: 1px solid #e8eaed;
        border-radius: 8px;
        padding: 20px;
        margin: 20px 0;
        overflow-x: auto;
    }
    :global(.markdown-body code) {
        font-family: "Google Sans Mono", "JetBrains Mono", monospace;
        color: #c5221f;
        background-color: #fce8e6;
        padding: 2px 5px;
        border-radius: 4px;
        font-size: 0.9em;
    }
    :global(.markdown-body h2) {
        font-size: 1.4rem;
        font-weight: 700;
        color: #202124;
        margin-top: 2rem;
        border-bottom: 1px solid #f1f3f4;
        padding-bottom: 0.5rem;
    }
</style>
