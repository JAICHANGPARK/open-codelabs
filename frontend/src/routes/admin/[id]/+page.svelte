<script lang="ts">
    import { onMount } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import { getCodelab, type Codelab, type Step } from "$lib/api";
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
    } from "lucide-svelte";

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
            title: "New Step",
            content_markdown: "# New Step Content\n\nStart writing here...",
        };
        steps = [...steps, newStep];
        activeStepIndex = steps.length - 1;
    }

    async function handleSave() {
        isSaving = true;
        // Mocking save for now
        setTimeout(() => {
            isSaving = false;
            saveSuccess = true;
            setTimeout(() => (saveSuccess = false), 3000);
        }, 1000);
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
                                title="View live"
                            >
                                <ExternalLink size={16} />
                            </a>
                        </h1>
                        <p class="text-xs text-[#5F6368] font-medium mt-0.5">
                            ID: {id} &bull; Facilitator Mode
                        </p>
                    {/if}
                </div>
            </div>
            <div class="flex items-center gap-4">
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
                    {saveSuccess ? "Saved" : "Save Content"}
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
                            >Codelab Steps</span
                        >
                        <button
                            on:click={addStep}
                            class="text-[#4285F4] hover:bg-[#E8F0FE] p-1.5 rounded-full transition-colors"
                            title="Add new step"
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
                            Attendee Access
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

                        <div class="flex-1 p-8">
                            {#if mode === "edit"}
                                <textarea
                                    bind:value={
                                        steps[activeStepIndex].content_markdown
                                    }
                                    class="w-full h-full min-h-[50vh] outline-none text-[#3C4043] font-mono text-base leading-relaxed resize-none bg-transparent"
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
                            Empty Codelab
                        </h3>
                        <p
                            class="text-[#5F6368] text-lg mb-10 max-w-sm mx-auto"
                        >
                            Bring your workshop to life by adding your first
                            educational step.
                        </p>
                        <button
                            on:click={addStep}
                            class="bg-[#4285F4] text-white px-10 py-3 rounded-full font-bold flex items-center gap-2 mx-auto shadow-md hover:shadow-lg transition-all active:scale-95"
                        >
                            <Plus size={20} /> Add First Step
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
