<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";
    import { streamGeminiResponseRobust } from "$lib/gemini";
    import { saveAiConversation } from "$lib/api";
    import { X, Send, Key, Sparkles, Loader2 } from "lucide-svelte";
    import { attendeeMarked as marked } from "$lib/markdown";
    import DOMPurify from "dompurify";
    import { fly, fade } from "svelte/transition";
    import { t } from "svelte-i18n";
    import FocusTrap from "$lib/components/FocusTrap.svelte";

    let {
        context = "",
        codelabId = "",
        stepNumber,
        onClose,
    } = $props<{
        context: string;
        codelabId?: string;
        stepNumber?: number;
        onClose: () => void;
    }>();

    let apiKey = $state("");
    let prompt = $state("");
    let response = $state("");
    let loading = $state(false);
    let hasKey = $state(false);
    let inputRef = $state<HTMLTextAreaElement | null>(null);
    let dialogRef: HTMLDivElement | null = null;

    onMount(() => {
        const storedKey = localStorage.getItem("gemini_api_key");
        if (storedKey) {
            apiKey = storedKey;
            hasKey = true;
        }
        // Auto-focus input
        setTimeout(() => inputRef?.focus(), 100);
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

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            event.preventDefault();
            onClose();
        }
    }

    async function handleSubmit() {
        if (!prompt.trim() || loading) return;

        loading = true;
        response = ""; // Clear previous
        const currentPrompt = prompt;
        const fullQuestion = context
            ? `${context}\n\n${currentPrompt}`
            : currentPrompt;
        prompt = ""; // Clear input immediately

        try {
            const stream = streamGeminiResponseRobust(currentPrompt, context, {
                apiKey,
            });

            for await (const chunk of stream) {
                if (chunk.text) {
                    response += chunk.text;
                }
            }

            // Save conversation to database if codelabId is provided
            if (codelabId && response) {
                try {
                    await saveAiConversation({
                        codelab_id: codelabId,
                        step_number: stepNumber,
                        question: fullQuestion,
                        answer: response,
                        model: "gemini-1.5-flash", // Default model
                    });
                } catch (saveError) {
                    console.error("Failed to save AI conversation:", saveError);
                    // Don't show error to user, just log it
                }
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
    onkeydown={handleKeydown}
    role="presentation"
>
    <button
        class="absolute inset-0"
        onclick={onClose}
        aria-label={$t("common.close")}
    ></button>

    <FocusTrap>
        <div
            bind:this={dialogRef}
            class="bg-white dark:bg-dark-surface shadow-2xl w-full max-w-lg h-full flex flex-col overflow-hidden border-l border-border dark:border-dark-border relative"
            transition:fly={{ x: 500, duration: 300 }}
            role="dialog"
            aria-modal="true"
            aria-labelledby="gemini-title"
        >
            <!-- Header -->
            <div
                class="flex items-center justify-between p-4 border-b border-border dark:border-dark-border bg-accent/60 dark:bg-dark-surface"
            >
                <div class="flex items-center gap-2 text-primary">
                    <span class="sr-only">{$t("gemini.ask_gemini")}</span>
                    <Sparkles size={20} aria-hidden="true" />
                    <h2
                        id="gemini-title"
                        class="font-bold text-foreground dark:text-dark-text"
                    >
                        {$t("gemini.ask_gemini")}
                    </h2>
                </div>
                <div class="flex items-center gap-2">
                    {#if hasKey}
                        <button
                            onclick={clearKey}
                            class="text-xs text-muted-foreground dark:text-dark-text-muted hover:text-red-500 font-medium underline px-2"
                        >
                            {$t("gemini.change_key")}
                        </button>
                    {/if}
                    <button
                        onclick={onClose}
                        class="p-1 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full text-muted-foreground dark:text-dark-text-muted"
                        aria-label={$t("common.close")}
                    >
                        <X size={20} />
                    </button>
                </div>
            </div>

            <!-- Content -->
            <div
                class="flex-1 overflow-y-auto p-6 bg-white dark:bg-dark-bg markdown-body"
                aria-live="polite"
            >
                {#if !hasKey}
                    <div
                        class="flex flex-col items-center justify-center h-full text-center space-y-6"
                    >
                        <div
                            class="w-16 h-16 bg-accent/70 dark:bg-primary/10 rounded-full flex items-center justify-center text-primary"
                            aria-hidden="true"
                        >
                            <Key size={32} />
                        </div>
                        <div>
                            <h3
                                class="font-bold text-lg text-foreground dark:text-dark-text"
                            >
                                {$t("gemini.enter_api_key")}
                            </h3>
                            <p
                                class="text-muted-foreground dark:text-dark-text-muted text-sm mt-1"
                            >
                                {$t("gemini.api_key_required_desc")}
                            </p>
                            <a
                                href="https://aistudio.google.com/app/apikey"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-primary text-sm hover:underline mt-1 block"
                                >{$t("gemini.get_api_key")}</a
                            >
                        </div>
                        <form
                            onsubmit={(e) => {
                                e.preventDefault();
                                saveKey();
                            }}
                            class="w-full max-sm flex flex-col sm:flex-row gap-2"
                        >
                            <input
                                type="password"
                                bind:value={apiKey}
                                placeholder={$t("gemini.paste_api_key")}
                                aria-label={$t(
                                    "dashboard.settings.gemini_api_key",
                                )}
                                class="flex-1 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-lg px-4 py-2 text-sm outline-none focus:border-primary dark:text-dark-text"
                            />
                            <button
                                type="submit"
                                disabled={!apiKey}
                                class="bg-primary text-primary-foreground font-bold py-2 px-6 rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-all"
                            >
                                {$t("common.save")}
                            </button>
                        </form>
                    </div>
                {:else}
                    <div class="space-y-6">
                        {#if context}
                            <div
                                class="bg-accent/60 dark:bg-dark-surface border border-border dark:border-dark-border rounded-lg p-3"
                            >
                                <span
                                    class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase mb-1 block"
                                    >{$t("gemini.context_selected")}</span
                                >
                                <div
                                    class="text-xs text-foreground dark:text-dark-text font-mono whitespace-pre-wrap max-h-32 overflow-y-auto"
                                >
                                    {context}
                                </div>
                            </div>
                        {/if}

                        {#if response}
                            <div
                                class="prose dark:prose-invert max-w-none text-foreground dark:text-dark-text text-sm leading-relaxed gemini-response"
                            >
                                {@html renderedResponse}
                            </div>
                        {/if}

                        {#if loading}
                            <div
                                class="flex items-center gap-2 text-muted-foreground dark:text-dark-text-muted text-sm animate-pulse"
                            >
                                <Loader2
                                    size={16}
                                    class="animate-spin"
                                    aria-hidden="true"
                                />
                                {$t("gemini.thinking")}
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>

            <!-- Footer Input -->
            {#if hasKey}
                <div
                    class="p-4 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface"
                >
                    <div class="relative flex items-end gap-2">
                        <textarea
                            bind:this={inputRef}
                            bind:value={prompt}
                            onkeydown={(e) => {
                                if (
                                    e.key === "Enter" &&
                                    !e.shiftKey &&
                                    !e.isComposing
                                ) {
                                    e.preventDefault();
                                    handleSubmit();
                                }
                            }}
                            placeholder={$t("gemini.ask_question_placeholder")}
                            aria-label={$t("gemini.ask_question_placeholder")}
                            rows="3"
                            class="w-full pl-4 pr-12 py-3 bg-background dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary transition-all text-sm resize-none dark:text-dark-text"
                        ></textarea>
                        <button
                            onclick={handleSubmit}
                            disabled={!prompt.trim() || loading}
                            class="absolute right-2 bottom-2 p-2 text-primary hover:bg-accent/70 rounded-lg transition-all disabled:opacity-50 disabled:hover:bg-transparent"
                            aria-label={$t("editor.send_dm")}
                        >
                            <Send size={18} />
                        </button>
                    </div>
                </div>
            {/if}
        </div>
    </FocusTrap>
</div>

<style>
    /* Add GitHub-like markdown styles for the response */
    :global(.gemini-response pre) {
        background-color: var(--color-muted);
        border: 1px solid var(--color-border);
        border-radius: 6px;
        padding: 16px;
        overflow: auto;
        font-size: 85%;
        line-height: 1.45;
        color: var(--color-foreground);
        transition: background-color 0.2s;
    }
    :global(html.dark .gemini-response pre) {
        background-color: var(--color-dark-bg);
        border-color: var(--color-dark-border);
        color: var(--color-dark-text);
    }
    :global(.gemini-response code) {
        background-color: var(--color-accent);
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
        color: var(--color-foreground);
    }
    :global(html.dark .gemini-response code) {
        background-color: var(--color-dark-hover);
        color: var(--color-dark-text);
    }
    :global(.gemini-response pre code) {
        background-color: transparent;
        padding: 0;
        color: inherit;
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
