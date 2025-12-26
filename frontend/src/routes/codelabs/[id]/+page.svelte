<script lang="ts">
    import { onMount } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import { getCodelab, type Codelab, type Step } from "$lib/api";
    import { loadProgress, saveProgress } from "$lib/Progress";
    import { marked } from "marked";
    import DOMPurify from "dompurify";
    import {
        ChevronLeft,
        ChevronRight,
        Menu,
        X,
        Clock,
        User,
        CheckCircle2,
        Home,
    } from "lucide-svelte";

    let id = page.params.id as string;
    let codelab: Codelab | null = null;
    let steps: Step[] = [];
    let loading = true;
    let currentStepIndex = 0;
    let showSidebar = true;
    let isFinished = false;

    onMount(async () => {
        try {
            const data = await getCodelab(id);
            codelab = data[0];
            steps = data[1];
            currentStepIndex = loadProgress(id);
            if (currentStepIndex >= steps.length) currentStepIndex = 0;
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    function nextStep() {
        if (currentStepIndex < steps.length - 1) {
            currentStepIndex++;
            saveProgress(id, currentStepIndex);
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    }

    function prevStep() {
        if (currentStepIndex > 0) {
            currentStepIndex--;
            saveProgress(id, currentStepIndex);
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    }

    function jumpToStep(index: number) {
        currentStepIndex = index;
        saveProgress(id, currentStepIndex);
        if (window.innerWidth < 1024) showSidebar = false;
        window.scrollTo({ top: 0, behavior: "smooth" });
    }

    function finishCodelab() {
        isFinished = true;
        window.scrollTo({ top: 0, behavior: "smooth" });
    }

    $: currentStep = steps[currentStepIndex];
    // @ts-ignore
    $: renderedContent = currentStep
        ? (DOMPurify.sanitize(
              marked.parse(currentStep.content_markdown) as string,
          ) as string)
        : "";
    $: progressPercent =
        steps.length > 0 ? ((currentStepIndex + 1) / steps.length) * 100 : 0;
</script>

<div
    class="min-h-screen bg-white flex flex-col font-sans text-[#3C4043] selection:bg-[#4285F4]/20 selection:text-[#4285F4]"
>
    <!-- Header -->
    <header
        class="h-16 border-b border-[#E8EAED] flex items-center justify-between px-4 lg:px-8 sticky top-0 bg-white z-30"
    >
        <div class="flex items-center gap-4">
            <button
                on:click={() => (showSidebar = !showSidebar)}
                class="p-2 hover:bg-[#F1F3F4] rounded-full lg:hidden transition-colors"
                aria-label="Toggle sidebar"
            >
                {#if showSidebar}<X size={20} />{:else}<Menu size={20} />{/if}
            </button>
            <div class="flex items-center gap-3">
                <div
                    class="w-8 h-8 bg-[#4285F4] rounded flex items-center justify-center text-white font-bold"
                >
                    A
                </div>
                <h1 class="font-bold text-lg hidden sm:block text-[#5F6368]">
                    AntiGravity
                </h1>
            </div>
        </div>

        <div class="flex-1 max-w-2xl px-8 text-center hidden md:block">
            <h2 class="font-medium text-[#3C4043] truncate text-base">
                {codelab?.title || "Loading..."}
            </h2>
        </div>

        <div class="flex items-center gap-6">
            <div
                class="hidden sm:flex items-center gap-2 text-[#5F6368] text-[11px] font-bold uppercase tracking-wider"
            >
                <Clock size={14} />
                <span>{steps.length * 5} mins remaining</span>
            </div>
            <div
                class="w-8 h-8 rounded-full bg-[#E8EAED] flex items-center justify-center text-[#5F6368]"
            >
                <User size={18} />
            </div>
        </div>
    </header>

    <!-- Progress Bar -->
    <div class="h-1 bg-[#F1F3F4] transition-all sticky top-16 z-30">
        <div
            class="h-full bg-[#4285F4] transition-all duration-700 ease-out"
            style="width: {isFinished ? 100 : progressPercent}%"
        ></div>
    </div>

    <div class="flex flex-1 relative overflow-hidden">
        <!-- Sidebar -->
        <aside
            class="fixed inset-y-0 left-0 transform {showSidebar
                ? 'translate-x-0'
                : '-translate-x-full'} lg:relative lg:translate-x-0 transition-transform duration-300 ease-in-out z-20 w-72 bg-[#F8F9FA] border-r border-[#E8EAED] overflow-y-auto pt-16 lg:pt-0"
        >
            <nav class="p-4 space-y-1">
                {#each steps as step, i}
                    <button
                        on:click={() => {
                            isFinished = false;
                            jumpToStep(i);
                        }}
                        class="w-full text-left p-3 rounded-lg flex items-start gap-4 transition-all duration-200 {currentStepIndex ===
                            i && !isFinished
                            ? 'bg-[#E8F0FE] text-[#1967D2]'
                            : 'hover:bg-[#F1F3F4] text-[#5F6368]'}"
                    >
                        <span
                            class="text-xs font-bold mt-1 w-5 h-5 rounded-full flex items-center justify-center shrink-0 {currentStepIndex ===
                                i && !isFinished
                                ? 'bg-[#4285F4] text-white'
                                : 'bg-[#E8EAED] text-[#5F6368]'}">{i + 1}</span
                        >
                        <span class="text-sm font-medium leading-tight pt-1"
                            >{step.title}</span
                        >
                    </button>
                {/each}
            </nav>
        </aside>

        {#if showSidebar}
            <button
                on:click={() => (showSidebar = false)}
                aria-label="Close sidebar"
                class="fixed inset-0 bg-[#3C4043]/40 backdrop-blur-[2px] z-10 lg:hidden transition-opacity"
                transition:fade={{ duration: 200 }}
            ></button>
        {/if}

        <!-- Content Area -->
        <main class="flex-1 overflow-y-auto p-6 lg:p-12 bg-white relative">
            <div class="max-w-3xl mx-auto min-h-full">
                {#if loading}
                    <div class="space-y-6" in:fade>
                        <div
                            class="h-12 bg-[#F1F3F4] rounded-md w-3/4 animate-pulse"
                        ></div>
                        <div class="space-y-3">
                            <div
                                class="h-4 bg-[#F1F3F4] rounded w-full animate-pulse"
                            ></div>
                            <div
                                class="h-4 bg-[#F1F3F4] rounded w-5/6 animate-pulse"
                            ></div>
                            <div
                                class="h-4 bg-[#F1F3F4] rounded w-4/5 animate-pulse"
                            ></div>
                        </div>
                        <div
                            class="h-80 bg-[#F8F9FA] rounded-xl w-full mt-10 animate-pulse"
                        ></div>
                    </div>
                {:else if isFinished}
                    <div
                        class="flex flex-col items-center justify-center py-20 text-center"
                        in:fly={{ y: 20, duration: 500 }}
                    >
                        <div
                            class="w-24 h-24 bg-[#E6F4EA] text-[#1E8E3E] rounded-full flex items-center justify-center mb-8"
                        >
                            <CheckCircle2 size={48} />
                        </div>
                        <h1 class="text-4xl font-extrabold text-[#202124] mb-4">
                            You're all done!
                        </h1>
                        <p
                            class="text-[#5F6368] text-xl max-w-lg mb-12 leading-relaxed"
                        >
                            Congratulations on completing <strong
                                >{codelab?.title}</strong
                            >. You've successfully finished all the steps in
                            this workshop.
                        </p>
                        <div class="flex flex-wrap justify-center gap-4">
                            <a
                                href="/"
                                class="bg-[#4285F4] hover:bg-[#1A73E8] text-white px-8 py-3 rounded-full font-bold shadow-md hover:shadow-lg transition-all flex items-center gap-2"
                            >
                                <Home size={20} />
                                Back to Home
                            </a>
                        </div>
                    </div>
                {:else if currentStep}
                    <div
                        class="prose max-w-none text-[#3C4043]"
                        in:fade={{ duration: 300 }}
                    >
                        <h1
                            class="text-[32px] leading-tight font-bold text-[#202124] border-b border-[#F1F3F4] pb-6 mb-10"
                        >
                            {currentStepIndex + 1}. {currentStep.title}
                        </h1>
                        <div class="markdown-body">
                            {@html renderedContent}
                        </div>
                    </div>
                {/if}
            </div>
        </main>
    </div>

    <!-- Footer Navigation -->
    <footer
        class="h-20 border-t border-[#E8EAED] bg-white flex items-center justify-center px-8 sticky bottom-0 z-30"
    >
        <div class="max-w-3xl w-full flex justify-between items-center">
            <button
                on:click={prevStep}
                disabled={currentStepIndex === 0 || isFinished}
                class="flex items-center gap-2 px-6 py-2 rounded-full font-bold transition-all {currentStepIndex ===
                    0 || isFinished
                    ? 'text-[#DADCE0] cursor-not-allowed'
                    : 'text-[#5F6368] hover:bg-[#F1F3F4]'}"
            >
                <ChevronLeft size={20} />
                Back
            </button>
            <div
                class="flex items-center text-sm font-bold text-[#5F6368] bg-[#F1F3F4] px-4 py-1.5 rounded-full"
            >
                {isFinished ? steps.length : currentStepIndex + 1} / {steps.length}
            </div>
            {#if currentStepIndex === steps.length - 1 && !isFinished}
                <button
                    on:click={finishCodelab}
                    class="bg-[#1E8E3E] hover:bg-[#178037] text-white px-10 py-2.5 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                >
                    Finish
                </button>
            {:else if isFinished}
                <div class="w-[100px]"></div>
            {:else}
                <button
                    on:click={nextStep}
                    class="bg-[#4285F4] hover:bg-[#1A73E8] text-white px-10 py-2.5 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                >
                    Next
                    <ChevronRight size={20} />
                </button>
            {/if}
        </div>
    </footer>
</div>

<style>
    :global(.markdown-body) {
        font-size: 1.125rem;
        line-height: 1.75;
    }
    :global(.markdown-body pre) {
        background-color: #f8f9fa;
        border: 1px solid #e8eaed;
        border-radius: 8px;
        padding: 24px;
        margin: 24px 0;
        overflow-x: auto;
    }
    :global(.markdown-body code) {
        font-family: "Google Sans Mono", "JetBrains Mono", monospace;
        font-size: 0.9em;
        color: #c5221f;
        background-color: #fce8e6;
        padding: 2px 6px;
        border-radius: 4px;
    }
    :global(.markdown-body h2) {
        font-size: 1.5rem;
        font-weight: 700;
        color: #202124;
        margin-top: 3rem;
        margin-bottom: 1.5rem;
        border-bottom: 1px solid #f1f3f4;
        padding-bottom: 0.5rem;
    }
    :global(.markdown-body p) {
        margin-bottom: 1.5rem;
    }
    :global(.markdown-body ul, .markdown-body ol) {
        margin-bottom: 1.5rem;
        padding-left: 1.5rem;
    }
    :global(.markdown-body li) {
        margin-bottom: 0.5rem;
    }
</style>
