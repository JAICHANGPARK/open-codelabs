<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";
    import { streamGeminiResponseRobust } from "$lib/gemini";
    import { X, Send, Key, Sparkles, Loader2 } from "lucide-svelte";
    import { marked } from "marked";
    import DOMPurify from "dompurify";
    import { fly, fade } from "svelte/transition";
    import Prism from "prismjs";

    let { context = "", onClose } = $props<{
        context: string;
        onClose: () => void;
    }>();

    let apiKey = $state("");
    let prompt = $state("");
    let response = $state("");
    let loading = $state(false);
    let hasKey = $state(false);
    let inputRef = $state<HTMLTextAreaElement | null>(null);

    onMount(() => {
        const storedKey = localStorage.getItem("gemini_api_key");
        if (storedKey) {
            apiKey = storedKey;
            hasKey = true;
        }
        // Auto-focus input
        setTimeout(() => inputRef?.focus(), 100);
    });

    // Highlight code whenever response updates
    $effect(() => {
        if (response) {
            Prism.highlightAll();
        }
    });

    function saveKey() {
        if (apiKey.trim()) {
            localStorage.setItem("gemini_api_key", apiKey.trim());
            hasKey = true;
        }
    }

    function clearKey() {
        localStorage.removeItem("gemini_api_key");
        apiKey = "";
        hasKey = false;
        response = "";
    }

    async function handleSubmit() {
        if (!prompt.trim() || loading) return;

        loading = true;
        response = ""; // Clear previous
        const currentPrompt = prompt;
        prompt = ""; // Clear input immediately

        try {
            const stream = streamGeminiResponseRobust(currentPrompt, context, {
                apiKey,
            });

            for await (const chunk of stream) {
                response += chunk;
            }
        } catch (e: any) {
            console.error(e);
            response = `**Error:** ${e.message}. Please check your API key.`;
        } finally {
            loading = false;
        }
    }

    let renderedResponse = $derived.by(() => {
        const html = marked.parse(response) as string;
        if (browser) {
            return DOMPurify.sanitize(html);
        }
        return html;
    });
</script>

<div
    class="fixed inset-0 bg-black/5 z-50 flex justify-end"
    transition:fade={{ duration: 200 }}
>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="absolute inset-0"
        onclick={onClose}
    ></div>

    <div
        class="bg-white shadow-2xl w-full max-w-lg h-full flex flex-col overflow-hidden border-l border-[#E8EAED] relative"
        transition:fly={{ x: 500, duration: 300 }}
    >
        <!-- Header -->
        <div
            class="flex items-center justify-between p-4 border-b border-[#E8EAED] bg-[#F8F9FA]"
        >
            <div class="flex items-center gap-2 text-[#4285F4]">
                <Sparkles size={20} />
                <h2 class="font-bold text-[#202124]">Ask Gemini</h2>
            </div>
            <div class="flex items-center gap-2">
                {#if hasKey}
                    <button
                        onclick={clearKey}
                        class="text-xs text-[#5F6368] hover:text-[#EA4335] font-medium underline px-2"
                    >
                        Change Key
                    </button>
                {/if}
                <button
                    onclick={onClose}
                    class="p-1 hover:bg-[#E8EAED] rounded-full text-[#5F6368]"
                >
                    <X size={20} />
                </button>
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto p-6 bg-white markdown-body">
            {#if !hasKey}
                <div
                    class="flex flex-col items-center justify-center h-full text-center space-y-6"
                >
                    <div
                        class="w-16 h-16 bg-[#E8F0FE] rounded-full flex items-center justify-center text-[#4285F4]"
                    >
                        <Key size={32} />
                    </div>
                    <div>
                        <h3 class="font-bold text-lg text-[#202124]">
                            Enter your Gemini API Key
                        </h3>
                        <p class="text-[#5F6368] text-sm mt-1">
                            To use AI features, you need a valid Google Gemini
                            API key.
                        </p>
                        <a
                            href="https://aistudio.google.com/app/apikey"
                            target="_blank"
                            class="text-[#4285F4] text-sm hover:underline mt-1 block"
                            >Get an API key here</a
                        >
                    </div>
                    <form
                        onsubmit={(e) => {
                            e.preventDefault();
                            saveKey();
                        }}
                        class="w-full max-w-sm flex gap-2"
                    >
                        <input
                            type="password"
                            bind:value={apiKey}
                            placeholder="Paste API Key here..."
                            class="flex-1 border border-[#DADCE0] rounded-lg px-4 py-2 text-sm outline-none focus:border-[#4285F4]"
                        />
                        <button
                            type="submit"
                            disabled={!apiKey}
                            class="bg-[#4285F4] text-white font-bold py-2 px-6 rounded-lg hover:bg-[#1A73E8] disabled:opacity-50 transition-all"
                        >
                            Save
                        </button>
                    </form>
                </div>
            {:else}
                <div class="space-y-6">
                    {#if context}
                        <div
                            class="bg-[#F8F9FA] border border-[#E8EAED] rounded-lg p-3"
                        >
                            <span
                                class="text-xs font-bold text-[#5F6368] uppercase mb-1 block"
                                >Context Selected</span
                            >
                            <div
                                class="text-xs text-[#3C4043] font-mono whitespace-pre-wrap max-h-32 overflow-y-auto"
                            >
                                {context}
                            </div>
                        </div>
                    {/if}

                    {#if response}
                        <div
                            class="prose max-w-none text-[#3C4043] text-sm leading-relaxed gemini-response"
                        >
                            {@html renderedResponse}
                        </div>
                    {/if}

                    {#if loading}
                        <div
                            class="flex items-center gap-2 text-[#5F6368] text-sm animate-pulse"
                        >
                            <Loader2 size={16} class="animate-spin" />
                            Thinking...
                        </div>
                    {/if}
                </div>
            {/if}
        </div>

        <!-- Footer Input -->
        {#if hasKey}
            <div class="p-4 border-t border-[#E8EAED] bg-white">
                <div class="relative flex items-end gap-2">
                    <textarea
                        bind:this={inputRef}
                        bind:value={prompt}
                        onkeydown={(e) => {
                            if (e.key === "Enter" && !e.shiftKey) {
                                e.preventDefault();
                                handleSubmit();
                            }
                        }}
                        placeholder="Ask a question about the code..."
                        rows="3"
                        class="w-full pl-4 pr-12 py-3 bg-[#F8F9FA] border border-[#DADCE0] rounded-xl outline-none focus:border-[#4285F4] transition-all text-sm resize-none"
                    ></textarea>
                    <button
                        onclick={handleSubmit}
                        disabled={!prompt.trim() || loading}
                        class="absolute right-2 bottom-2 p-2 text-[#4285F4] hover:bg-[#E8F0FE] rounded-lg transition-all disabled:opacity-50 disabled:hover:bg-transparent"
                    >
                        <Send size={18} />
                    </button>
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    /* Add GitHub-like markdown styles for the response */
    :global(.gemini-response pre) {
        background-color: #f6f8fa;
        border-radius: 6px;
        padding: 16px;
        overflow: auto;
        font-size: 85%;
        line-height: 1.45;
    }
    :global(.gemini-response code) {
        background-color: rgba(175, 184, 193, 0.2);
        padding: 0.2em 0.4em;
        border-radius: 6px;
        font-family:
            ui-monospace,
            SFMono-Regular,
            SF Mono,
            Menlo,
            Consolas,
            Liberation Mono,
            monospace;
        font-size: 85%;
    }
    :global(.gemini-response pre code) {
        background-color: transparent;
        padding: 0;
    }
    :global(.gemini-response p) {
        margin-bottom: 1rem;
    }
    :global(.gemini-response ul, .gemini-response ol) {
        margin-bottom: 1rem;
        padding-left: 1.5rem;
    }
    :global(.gemini-response li) {
        margin-bottom: 0.5rem;
    }
</style>
