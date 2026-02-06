<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";
    import {
        streamGeminiChat,
        type GeminiResponseChunk,
        withCsrf,
    } from "$lib/gemini";
    import {
        X,
        Send,
        Key,
        Sparkles,
        Loader2,
        ExternalLink,
        Search,
        Trash2,
        User as UserIcon,
        Bot,
        History,
        Plus,
        ChevronRight,
        MessageSquare,
    } from "lucide-svelte";
    import { adminMarked as marked } from "$lib/markdown";
    import DOMPurify from "dompurify";
    import { fly, fade } from "svelte/transition";
    import { t, locale } from "svelte-i18n";
    import { encrypt, decrypt } from "$lib/crypto";
    import FocusTrap from "$lib/components/FocusTrap.svelte";
    import referenceCodelabs from "$lib/data/codelabs.csv?raw";

    let { onClose } = $props<{
        onClose: () => void;
    }>();

    type Message = {
        role: "user" | "assistant";
        content: string;
        groundingMetadata?: any;
        usageMetadata?: {
            promptTokenCount: number;
            candidatesTokenCount: number;
            totalTokenCount: number;
        };
    };
    type Thread = {
        id: string;
        title: string;
        created_at: string;
        updated_at: string;
    };

    let messages = $state<Message[]>([]);
    let threads = $state<Thread[]>([]);
    let currentThreadId = $state<string | null>(null);
    let showHistory = $state(false);

    let apiKey = $state("");
    let prompt = $state("");
    let loading = $state(false);
    let hasKey = $state(false);
    let inputRef = $state<HTMLTextAreaElement | null>(null);
    let scrollContainer = $state<HTMLDivElement | null>(null);
    let dialogRef: HTMLDivElement | null = null;

    const BASE_SYSTEM_PROMPT = `You are a very strong reasoner and planner. Use these critical instructions to structure your plans, thoughts, and responses.

    Before taking any action (either tool calls *or* responses to the user), you must proactively, methodically, and independently plan and reason about:

    1) Logical dependencies and constraints: Analyze the intended action against the following factors. Resolve conflicts in order of importance:
        1.1) Policy-based rules, mandatory prerequisites, and constraints.
        1.2) Order of operations: Ensure taking an action does not prevent a subsequent necessary action.
        1.3) Other prerequisites (information and/or actions needed).
        1.4) Explicit user constraints or preferences.

    2) Risk assessment: What are the consequences of taking the action? Will the new state cause any future issues?
        2.1) For exploratory tasks (like searches), missing *optional* parameters is a LOW risk.

    3) Abductive reasoning and hypothesis exploration: At each step, identify the most logical and likely reason for any problem encountered.
        3.1) Look beyond immediate or obvious causes.
        3.2) Hypotheses may require additional research.
        3.3) Prioritize hypotheses based on likelihood.

    4) Outcome evaluation and adaptability: Does the previous observation require any changes to your plan?
        4.1) If your initial hypotheses are disproven, actively generate new ones.

    5) Information availability: Incorporate all applicable and alternative sources of information.

    6) Precision and Grounding: Ensure your reasoning is extremely precise and relevant to each exact ongoing situation.

    7) Completeness: Ensure that all requirements, constraints, options, and preferences are exhaustively incorporated into your plan.

    8) Persistence and patience: Do not give up unless all the reasoning above is exhausted.

    9) Inhibit your response: only take an action after all the above reasoning is completed. Once you've taken an action, you cannot take it back.

    ---
    
    You are also a world-class Technical Content Consultant and Developer Advocate. 
    Your mission is to help facilitators design high-quality, professional "Hands-on Codelabs". 
    You provide expert advice on:
    1. Defining clear learning objectives.
    2. Structuring steps from zero to hero.
    3. Technical accuracy and environment setup.
    4. Engaging narrative and "The Why before the How".
    5. Modern best practices for specific technologies. 

    When you provide information that you found via Google Search, make sure to mention that you are citing external sources.
    Keep your advice actionable, professional, and encouraging.
    Respond in the user's language (default to English if unsure).`;

    const SYSTEM_PROMPT = $derived.by(() => {
        if (!referenceCodelabs) return BASE_SYSTEM_PROMPT;
        return `${BASE_SYSTEM_PROMPT}

    You have access to a reference list of existing Codelabs. If the user asks for suggestions, similar topics, or what's already available, please refer to this list:
    
    \`\`\`csv
    ${referenceCodelabs}
    \`\`\``;
    });

    onMount(async () => {
        const storedKey = localStorage.getItem("gemini_api_key");
        if (storedKey) {
            const decrypted = decrypt(storedKey);
            if (decrypted) {
                apiKey = decrypted;
                hasKey = true;
            }
        }

        await fetchThreads();
        // await fetchReferenceCodelabs(); // Now using direct raw import

        // Auto-focus input
        setTimeout(() => inputRef?.focus(), 100);
    });

    async function fetchThreads() {
        try {
            const res = await fetch("/api/ai/threads");
            if (res.ok) {
                threads = await res.json();
            }
        } catch (e) {
            console.error("Failed to fetch threads", e);
        }
    }

    /* Now using direct raw import from $lib/data/codelabs.csv?raw
    async function fetchReferenceCodelabs() {
        try {
            const res = await fetch("/api/codelabs/reference");
            if (res.ok) {
                referenceCodelabs = await res.text();
            }
        } catch (e) {
            console.error("Failed to fetch reference codelabs", e);
        }
    }
    */

    async function loadThread(threadId: string) {
        loading = true;
        currentThreadId = threadId;
        showHistory = false;
        try {
            const res = await fetch(`/api/ai/threads/${threadId}`);
            if (res.ok) {
                const data = await res.json();
                messages = data.map((m: any) => ({
                    role: m.role === "model" ? "assistant" : "user",
                    content: m.content,
                    groundingMetadata: m.grounding_metadata
                        ? JSON.parse(m.grounding_metadata)
                        : null,
                    usageMetadata: m.usage_metadata
                        ? JSON.parse(m.usage_metadata)
                        : null,
                }));
            }
        } catch (e) {
            console.error("Failed to load messages", e);
        } finally {
            loading = false;
        }
    }

    async function deleteThread(threadId: string) {
        if (
            !confirm(
                $t("admin.consultant.confirm_delete_thread") ||
                    "Delete this conversation?",
            )
        )
            return;
        try {
            const res = await fetch(`/api/ai/threads/${threadId}`, {
                method: "DELETE",
                headers: withCsrf(),
            });
            if (res.ok) {
                threads = threads.filter((t) => t.id !== threadId);
                if (currentThreadId === threadId) {
                    messages = [];
                    currentThreadId = null;
                }
            }
        } catch (e) {
            console.error("Failed to delete thread", e);
        }
    }

    function startNewChat() {
        messages = [];
        currentThreadId = null;
        showHistory = false;
        setTimeout(() => inputRef?.focus(), 100);
    }

    $effect(() => {
        if (messages.length > 0 && !showHistory) {
            // Auto-scroll
            if (scrollContainer) {
                scrollContainer.scrollTo({
                    top: scrollContainer.scrollHeight,
                    behavior: "smooth",
                });
            }
        }
    });

    async function handleSubmit() {
        if (!prompt.trim() || loading) return;

        const userPrompt = prompt.trim();
        prompt = "";
        loading = true;

        // 1. Ensure thread exists
        if (!currentThreadId) {
            try {
                const title =
                    userPrompt.slice(0, 40) +
                    (userPrompt.length > 40 ? "..." : "");
                const res = await fetch("/api/ai/threads", {
                    method: "POST",
                    headers: withCsrf({ "Content-Type": "application/json" }),
                    body: JSON.stringify({ title }),
                });
                if (res.ok) {
                    const thread = await res.json();
                    currentThreadId = thread.id;
                    threads = [thread, ...threads];
                }
            } catch (e) {
                console.error("Failed to create thread", e);
            }
        }

        const userMsg: Message = { role: "user", content: userPrompt };
        messages = [...messages, userMsg];

        // Save user message to DB
        if (currentThreadId) {
            fetch(`/api/ai/threads/${currentThreadId}`, {
                method: "POST",
                headers: withCsrf({ "Content-Type": "application/json" }),
                body: JSON.stringify({ role: "user", content: userPrompt }),
            });
        }

        const assistantMsg: Message = { role: "assistant", content: "" };
        messages = [...messages, assistantMsg];

        try {
            const stream = streamGeminiChat(
                messages.slice(0, -1),
                SYSTEM_PROMPT,
                {
                    apiKey,
                    model: "gemini-3-flash-preview",
                    tools: [{ googleSearch: {} }],
                },
            );

            let finalContent = "";
            let finalMetadata = null;

            for await (const chunk of stream) {
                if (chunk.text) {
                    finalContent += chunk.text;
                    messages[messages.length - 1].content = finalContent;
                }
                if (chunk.groundingMetadata) {
                    finalMetadata = chunk.groundingMetadata;
                    messages[messages.length - 1].groundingMetadata =
                        finalMetadata;
                }
                if (chunk.usageMetadata) {
                    messages[messages.length - 1].usageMetadata =
                        chunk.usageMetadata;
                }
            }

            const finalUsage = messages[messages.length - 1].usageMetadata;

            // Save assistant message to DB
            if (currentThreadId) {
                fetch(`/api/ai/threads/${currentThreadId}`, {
                    method: "POST",
                    headers: withCsrf({ "Content-Type": "application/json" }),
                    body: JSON.stringify({
                        role: "model",
                        content: finalContent,
                        grounding_metadata: finalMetadata,
                        usage_metadata: finalUsage,
                    }),
                });
            }
        } catch (e: any) {
            console.error(e);
            messages[messages.length - 1].content =
                `**Error:** ${e.message}. Please check your API key.`;
        } finally {
            loading = false;
        }
    }

    function renderMarkdown(content: string) {
        const html = marked.parse(content) as string;
        if (browser) {
            return DOMPurify.sanitize(html);
        }
        return html;
    }

    function getSources(groundingMetadata: any) {
        if (!groundingMetadata?.groundingChunks) return [];
        return groundingMetadata.groundingChunks
            .map((chunk: any) => {
                if (chunk.web) {
                    return {
                        title: chunk.web.title,
                        uri: chunk.web.uri,
                    };
                }
                return null;
            })
            .filter(Boolean);
    }
</script>

<div
    class="fixed inset-0 bg-black/20 z-[100] flex justify-end"
    transition:fade={{ duration: 200 }}
>
    <button
        class="absolute inset-0 cursor-default"
        onclick={onClose}
        onkeydown={(e) => e.key === "Escape" && onClose()}
        aria-label={$t("common.close")}
    ></button>

    <FocusTrap>
        <div
            bind:this={dialogRef}
            class="bg-white dark:bg-dark-surface shadow-2xl w-[600px] max-w-full h-full flex flex-col overflow-hidden border-l border-border dark:border-dark-border relative"
            in:fly|local={{ x: 600, duration: 400 }}
            out:fly|local={{ x: 600, duration: 400 }}
            role="dialog"
            aria-modal="true"
            aria-labelledby="consultant-title"
        >
            <!-- Header -->
            <div
                class="flex items-center justify-between p-4 sm:p-6 border-b border-border dark:border-dark-border bg-white dark:bg-dark-surface"
            >
                <div class="flex items-center gap-3 text-primary">
                    <div
                        class="w-10 h-10 bg-accent/70 dark:bg-primary/10 rounded-xl flex items-center justify-center"
                    >
                        <Sparkles size={24} />
                    </div>
                    <div>
                        <h2
                            id="consultant-title"
                            class="font-bold text-lg text-foreground dark:text-dark-text"
                        >
                            {$t("admin.consultant.title") ||
                                "Codelab Consultant"}
                        </h2>
                        <p
                            class="text-xs text-muted-foreground dark:text-dark-text-muted"
                        >
                            {$t("admin.consultant.subtitle") ||
                                "Designing your next hands-on session"}
                        </p>
                    </div>
                </div>
                <div class="flex items-center gap-1 sm:gap-2">
                    <button
                        onclick={() => {
                            showHistory = !showHistory;
                            if (showHistory) fetchThreads();
                        }}
                        class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full text-muted-foreground dark:text-dark-text-muted transition-colors {showHistory
                            ? 'text-primary bg-accent/70'
                            : ''}"
                        title={$t("admin.consultant.history")}
                    >
                        <History size={20} />
                    </button>
                    {#if messages.length > 0 && !showHistory}
                        <button
                            onclick={startNewChat}
                            class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full text-muted-foreground dark:text-dark-text-muted transition-colors"
                            title={$t("admin.consultant.new_chat")}
                        >
                            <Plus size={20} />
                        </button>
                    {/if}
                    <button
                        onclick={onClose}
                        class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full text-muted-foreground dark:text-dark-text-muted transition-colors"
                        aria-label={$t("common.close")}
                    >
                        <X size={20} />
                    </button>
                </div>
            </div>

            <!-- Content -->
            <div
                bind:this={scrollContainer}
                class="flex-1 overflow-y-auto p-6 bg-muted dark:bg-dark-bg"
            >
                {#if !hasKey}
                    <!-- API Key Entry UI (Same as before) -->
                    <div
                        class="flex flex-col items-center justify-center h-full text-center space-y-6 max-w-sm mx-auto"
                    >
                        <div
                            class="w-16 h-16 bg-white dark:bg-dark-surface rounded-2xl shadow-sm flex items-center justify-center text-primary"
                        >
                            <Key size={32} />
                        </div>
                        <div>
                            <h3
                                class="font-bold text-xl text-foreground dark:text-dark-text"
                            >
                                {$t("gemini.enter_api_key")}
                            </h3>
                            <p
                                class="text-muted-foreground dark:text-dark-text-muted text-sm mt-2"
                            >
                                {$t("admin.consultant.api_key_desc") ||
                                    "Please configure your Gemini API key in the dashboard settings to use the consultant."}
                            </p>
                        </div>
                    </div>
                {:else if showHistory}
                    <div class="space-y-4 max-w-2xl mx-auto" in:fade>
                        <div class="flex items-center justify-between mb-6">
                            <h3
                                class="font-bold text-xl text-foreground dark:text-dark-text flex items-center gap-2"
                            >
                                <History size={24} class="text-primary" />
                                {$t("admin.consultant.history")}
                            </h3>
                            <button
                                onclick={startNewChat}
                                class="flex items-center gap-2 px-4 py-2 bg-primary text-white rounded-xl hover:bg-primary/90 transition-colors text-sm font-medium shadow-sm"
                            >
                                <Plus size={18} />
                                {$t("admin.consultant.new_chat")}
                            </button>
                        </div>

                        {#if threads.length === 0}
                            <div
                                class="text-center py-20 bg-white dark:bg-dark-surface rounded-3xl border border-border dark:border-dark-border"
                            >
                                <MessageSquare
                                    size={48}
                                    class="text-muted-foreground/60 mx-auto mb-4"
                                />
                                <p
                                    class="text-muted-foreground dark:text-dark-text-muted"
                                >
                                    {$t("admin.consultant.no_history")}
                                </p>
                            </div>
                        {:else}
                            {#each threads as thread}
                                <div class="group flex items-center gap-3">
                                    <button
                                        onclick={() => loadThread(thread.id)}
                                        class="flex-1 flex items-center justify-between p-5 bg-white dark:bg-dark-surface hover:bg-muted dark:hover:bg-white/5 border border-border dark:border-dark-border rounded-2xl transition-all text-left shadow-sm group-hover:shadow-md"
                                    >
                                        <div class="flex items-center gap-4">
                                            <div
                                                class="w-10 h-10 bg-accent/70 dark:bg-primary/10 rounded-lg flex items-center justify-center text-primary"
                                            >
                                                <MessageSquare size={20} />
                                            </div>
                                            <div>
                                                <h4
                                                    class="font-medium text-foreground dark:text-dark-text line-clamp-1"
                                                >
                                                    {thread.title}
                                                </h4>
                                                <p
                                                    class="text-[10px] text-muted-foreground dark:text-dark-text-muted mt-1 uppercase tracking-wider"
                                                >
                                                    {new Date(
                                                        thread.updated_at,
                                                    ).toLocaleDateString()}
                                                </p>
                                            </div>
                                        </div>
                                        <ChevronRight
                                            size={18}
                                            class="text-muted-foreground/60 group-hover:text-primary transition-colors"
                                        />
                                    </button>
                                    <button
                                        onclick={() => deleteThread(thread.id)}
                                        class="p-3 text-muted-foreground/60 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-xl transition-all opacity-0 group-hover:opacity-100"
                                        title={$t("common.delete")}
                                    >
                                        <Trash2 size={18} />
                                    </button>
                                </div>
                            {/each}
                        {/if}
                    </div>
                {:else}
                    <div class="space-y-8 w-full max-w-4xl mx-auto pb-10">
                        {#if messages.length === 0}
                            <div
                                class="bg-white dark:bg-dark-surface rounded-3xl p-10 border border-border dark:border-dark-border shadow-sm text-center"
                                in:fade
                            >
                                <div
                                    class="w-20 h-20 bg-accent/70 dark:bg-primary/10 rounded-3xl flex items-center justify-center text-primary mx-auto mb-6"
                                >
                                    <Sparkles size={40} />
                                </div>
                                <h3
                                    class="font-bold text-2xl text-foreground dark:text-dark-text"
                                >
                                    {$t("admin.consultant.how_can_i_help") ||
                                        "How can I help you today?"}
                                </h3>
                                <p
                                    class="text-muted-foreground dark:text-dark-text-muted text-base mt-3 max-w-md mx-auto"
                                >
                                    {$t("admin.consultant.help_desc") ||
                                        "Ask about codelab structure, choosing a topic, or technical implementation details."}
                                </p>
                            </div>
                        {/if}

                        {#each messages as msg, i}
                            <div
                                class="flex flex-col gap-4 {msg.role === 'user'
                                    ? 'items-end'
                                    : 'items-start'}"
                                in:fly={{ y: 20, duration: 300 }}
                            >
                                <div class="flex items-center gap-2 px-1">
                                    {#if msg.role === "assistant"}
                                        <div
                                            class="w-6 h-6 bg-primary text-white rounded-full flex items-center justify-center"
                                        >
                                            <Bot size={14} />
                                        </div>
                                        <span
                                            class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
                                            >{$t(
                                                "admin.consultant.role_consultant",
                                            )}</span
                                        >
                                    {:else}
                                        <span
                                            class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider"
                                            >{$t(
                                                "admin.consultant.role_you",
                                            )}</span
                                        >
                                        <div
                                            class="w-6 h-6 bg-border dark:bg-white/10 text-muted-foreground dark:text-dark-text rounded-full flex items-center justify-center"
                                        >
                                            <UserIcon size={14} />
                                        </div>
                                    {/if}
                                </div>

                                <div
                                    class="max-w-[90%] sm:max-w-[85%] rounded-3xl p-5 sm:p-6 shadow-sm border {msg.role ===
                                    'user'
                                        ? 'bg-accent/70 dark:bg-primary/20 border-primary/20 dark:border-primary/30 text-primary dark:text-blue-200'
                                        : 'bg-white dark:bg-dark-surface border-border dark:border-dark-border text-foreground dark:text-dark-text'}"
                                >
                                    <div
                                        class="prose dark:prose-invert max-w-none text-sm leading-relaxed markdown-content"
                                    >
                                        {#if msg.role === "assistant" && !msg.content && loading && i === messages.length - 1}
                                            <div
                                                class="flex items-center gap-3 py-2"
                                            >
                                                <Loader2
                                                    size={18}
                                                    class="animate-spin text-primary"
                                                />
                                                <span
                                                    class="text-xs text-muted-foreground dark:text-dark-text-muted italic"
                                                    >{$t(
                                                        "gemini.thinking",
                                                    )}...</span
                                                >
                                            </div>
                                        {:else}
                                            {@html renderMarkdown(msg.content)}
                                        {/if}
                                    </div>

                                    {#if msg.groundingMetadata}
                                        {@const sources = getSources(
                                            msg.groundingMetadata,
                                        )}
                                        {#if sources.length > 0}
                                            <div
                                                class="mt-4 pt-4 border-t border-border dark:border-dark-border/50"
                                            >
                                                <p
                                                    class="text-[10px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase mb-3 flex items-center gap-2"
                                                >
                                                    <Search size={12} />
                                                    {$t(
                                                        "admin.consultant.sources",
                                                    )}
                                                </p>
                                                <div
                                                    class="flex flex-wrap gap-2"
                                                >
                                                    {#each sources as source}
                                                        <a
                                                            href={source.uri}
                                                            target="_blank"
                                                            rel="noopener noreferrer"
                                                            class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-accent/60 dark:bg-white/5 hover:bg-border dark:hover:bg-white/10 rounded-full text-[10px] font-medium transition-colors"
                                                        >
                                                            {source.title}
                                                            <ExternalLink
                                                                size={10}
                                                            />
                                                        </a>
                                                    {/each}
                                                </div>
                                            </div>
                                        {/if}
                                    {/if}

                                    {#if msg.role === "assistant" && msg.usageMetadata}
                                        <div
                                            class="mt-3 pt-3 border-t border-border dark:border-dark-border/50 flex items-center justify-between text-[10px] text-muted-foreground dark:text-dark-text-muted"
                                        >
                                            <div
                                                class="flex items-center gap-2"
                                            >
                                                <Sparkles
                                                    size={12}
                                                    class="text-primary"
                                                />
                                                <span>Tokens:</span>
                                            </div>
                                            <div class="flex gap-3 font-medium">
                                                <span
                                                    >Input: {msg.usageMetadata
                                                        .promptTokenCount}</span
                                                >
                                                <span
                                                    >Output: {msg.usageMetadata
                                                        .candidatesTokenCount}</span
                                                >
                                                <span class="text-primary"
                                                    >Total: {msg.usageMetadata
                                                        .totalTokenCount}</span
                                                >
                                            </div>
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>

            <!-- Footer Input -->
            {#if hasKey && !showHistory}
                <div
                    class="p-4 sm:p-6 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface"
                >
                    <div
                        class="max-w-3xl mx-auto relative flex items-end gap-3"
                    >
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
                                    e.stopPropagation();
                                    handleSubmit();
                                }
                            }}
                            placeholder={$t("admin.consultant.placeholder") ||
                                "Ask for advice on your codelab..."}
                            rows="3"
                            class="w-full pl-4 pr-14 py-4 bg-muted dark:bg-dark-bg border border-border dark:border-dark-border rounded-2xl outline-none focus:border-primary focus:ring-2 focus:ring-primary/10 text-sm resize-none dark:text-dark-text"
                        ></textarea>
                        <button
                            onclick={handleSubmit}
                            disabled={!prompt.trim() || loading}
                            class="absolute right-3 bottom-3 p-3 bg-primary text-white rounded-xl hover:bg-primary/90 disabled:opacity-50 disabled:grayscale transition-all shadow-md shadow-blue-500/20"
                            aria-label={$t("common.send")}
                        >
                            <Send size={20} />
                        </button>
                    </div>
                </div>
            {/if}
        </div>
    </FocusTrap>
</div>

<style>
    :global(.markdown-content pre) {
        background-color: var(--color-muted);
        border-radius: 8px;
        padding: 16px;
        overflow: auto;
        font-size: 85%;
        line-height: 1.45;
        border: 1px solid var(--color-border);
    }
    :global(html.dark .markdown-content pre) {
        background-color: var(--color-dark-bg);
        border-color: var(--color-dark-border);
    }
    :global(.markdown-content code) {
        background-color: var(--color-accent);
        padding: 0.2em 0.4em;
        border-radius: 6px;
        font-family:
            ui-monospace,
            SFMono-Regular,
            SF Mono,
            Menlo,
            monospace;
        font-size: 85%;
    }
    :global(html.dark .markdown-content code) {
        background-color: var(--color-dark-hover);
    }
    :global(.markdown-content p) {
        margin-bottom: 1rem;
    }
    :global(.markdown-content ul, .markdown-content ol) {
        margin-bottom: 1rem;
        padding-left: 1.5rem;
    }
</style>
