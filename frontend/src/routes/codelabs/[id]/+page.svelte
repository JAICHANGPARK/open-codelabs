<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import {
        getCodelab,
        requestHelp,
        getWsUrl,
        getChatHistory,
        submitFeedback,
        type Codelab,
        type Step,
        type Attendee,
        type ChatMessage,
    } from "$lib/api";
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
        Check,
        Home,
        MessageSquare,
        Send,
        HelpCircle,
        AlertCircle,
        Star,
        Users,
        Sparkles,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import AskGemini from "$lib/components/AskGemini.svelte";

    // Prism.js for syntax highlighting
    import Prism from "prismjs";
    import "prismjs/themes/prism-tomorrow.css";
    import "prismjs/components/prism-javascript";
    import "prismjs/components/prism-typescript";
    import "prismjs/components/prism-python";
    import "prismjs/components/prism-java";
    import "prismjs/components/prism-rust";
    import "prismjs/components/prism-dart";
    import "prismjs/components/prism-swift";
    import "prismjs/components/prism-kotlin";
    import "prismjs/components/prism-go";
    import "prismjs/components/prism-bash";
    import "prismjs/components/prism-json";
    import "prismjs/components/prism-yaml";
    import "prismjs/components/prism-markdown";

    let id = page.params.id as string;
    let codelab = $state<Codelab | null>(null);
    let steps = $state<Step[]>([]);
    let loading = $state(true);
    let currentStepIndex = $state(0);
    let showSidebar = $state(true);
    let showChat = $state(false);
    let isFinished = $state(false);

    let attendee = $state<Attendee | null>(null);

    // Feedback State
    let feedbackDifficulty = $state(3);
    let feedbackSatisfaction = $state(5);
    let feedbackComment = $state("");
    let feedbackSubmitted = $state(false);
    let feedbackSubmitting = $state(false);
    let chatMessage = $state("");
    let messages = $state<
        {
            sender: string;
            text: string;
            time: string;
            self?: boolean;
            type: "chat" | "dm";
        }[]
    >([]);
    let ws = $state<WebSocket | null>(null);
    let helpSent = $state(false);
    let chatTab = $state<"public" | "direct">("public");
    let hasNewDm = $state(false);

    // Gemini State
    let showGeminiButton = $state(false);
    let showGeminiModal = $state(false);
    let selectedContext = $state("");
    let geminiButtonPos = $state({ x: 0, y: 0 });

    let filteredMessages = $derived(
        chatTab === "public"
            ? messages.filter((m) => m.type === "chat")
            : messages.filter((m) => m.type === "dm"),
    );

    $effect(() => {
        if (ws && ws.readyState === WebSocket.OPEN && attendee) {
            ws.send(
                JSON.stringify({
                    type: "step_progress",
                    attendee_id: attendee.id,
                    step_number: currentStepIndex + 1,
                }),
            );
        }
    });

    // Apply syntax highlighting when step content changes
    $effect(() => {
        if (currentStepIndex >= 0 && steps.length > 0) {
            // Wait for DOM to update then highlight
            setTimeout(() => {
                Prism.highlightAll();
            }, 100);
        }
    });

    onMount(async () => {
        // Check for registration
        const savedAttendee = localStorage.getItem(`attendee_${id}`);
        if (!savedAttendee) {
            goto(`/codelabs/${id}/entry`);
            return;
        }
        attendee = JSON.parse(savedAttendee);

        try {
            const data = await getCodelab(id);
            codelab = data[0];
            steps = data[1];
            currentStepIndex = loadProgress(id);
            if (currentStepIndex >= steps.length) currentStepIndex = 0;

            await loadChatHistory();
            await loadChatHistory();
            initWebSocket();

            // Selection listener
            document.addEventListener("mouseup", handleSelection);
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    onDestroy(() => {
        if (ws) ws.close();
        document.removeEventListener("mouseup", handleSelection);
    });

    function handleSelection(e: MouseEvent) {
        // Debounce or just wait a tick
        setTimeout(() => {
            const selection = window.getSelection();
            if (selection && selection.toString().trim().length > 0) {
                // Check if selection is inside markdown-body
                const range = selection.getRangeAt(0);
                const container = range.commonAncestorContainer;
                const element =
                    container.nodeType === 1
                        ? (container as Element)
                        : container.parentElement;

                if (element?.closest(".markdown-body")) {
                    const rect = range.getBoundingClientRect();
                    selectedContext = selection.toString();
                    geminiButtonPos = {
                        x: rect.right + 10, // Show to the right of selection
                        y: rect.top - 40, // Show slightly above
                    };
                    // Ensure it stays on screen
                    if (geminiButtonPos.x + 150 > window.innerWidth) {
                        geminiButtonPos.x = window.innerWidth - 160;
                    }
                    if (geminiButtonPos.y < 0) geminiButtonPos.y = 10;

                    showGeminiButton = true;
                    return;
                }
            }
            // If we click outside or empty selection, hide button
            if (!showGeminiModal) {
                showGeminiButton = false;
            }
        }, 10);
    }

    async function loadChatHistory() {
        if (!attendee) return;
        try {
            const history = await getChatHistory(id);
            messages = history
                .filter(
                    (msg) =>
                        msg.msg_type === "chat" ||
                        msg.target_id === attendee?.id ||
                        msg.sender_name === attendee?.name,
                )
                .map((msg) => {
                    const timeStr = msg.created_at
                        ? new Date(msg.created_at).toLocaleTimeString([], {
                              hour: "2-digit",
                              minute: "2-digit",
                          })
                        : "";

                    if (msg.msg_type === "chat") {
                        return {
                            sender: msg.sender_name,
                            text: msg.message,
                            time: timeStr,
                            self: msg.sender_name === attendee?.name,
                            type: "chat",
                        };
                    } else {
                        // DM for or from me
                        const isSelf = msg.sender_name === attendee?.name;
                        return {
                            sender: isSelf
                                ? `To: Facilitator`
                                : `[DM] ${msg.sender_name}`,
                            text: msg.message,
                            time: timeStr,
                            self: isSelf,
                            type: "dm",
                        };
                    }
                });

            // Scroll to bottom
            setTimeout(() => {
                const chatContainer = document.getElementById("chat-messages");
                if (chatContainer)
                    chatContainer.scrollTop = chatContainer.scrollHeight;
            }, 100);
        } catch (e) {
            console.error("Failed to load chat history:", e);
        }
    }

    function initWebSocket() {
        const wsUrl = getWsUrl(id);
        ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            if (attendee) {
                ws?.send(JSON.stringify({ attendee_id: attendee.id }));
                // Send initial progress
                ws?.send(
                    JSON.stringify({
                        type: "step_progress",
                        attendee_id: attendee.id,
                        step_number: currentStepIndex + 1,
                    }),
                );
            }
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === "chat") {
                    messages = [
                        ...messages,
                        {
                            sender: data.sender,
                            text: data.message,
                            time: data.timestamp,
                            self: data.sender === attendee?.name,
                            type: "chat",
                        },
                    ];
                    // Scroll to bottom of chat
                    setTimeout(() => {
                        const chatContainer =
                            document.getElementById("chat-messages");
                        if (chatContainer)
                            chatContainer.scrollTop =
                                chatContainer.scrollHeight;
                    }, 50);
                } else if (data.type === "dm") {
                    // Show DM in chat
                    messages = [
                        ...messages,
                        {
                            sender: `[DM] ${data.sender}`,
                            text: data.message,
                            time: data.timestamp,
                            self: false,
                            type: "dm",
                        },
                    ];
                    if (chatTab !== "direct") {
                        hasNewDm = true;
                    }
                    showChat = true; // Auto-open chat for DM
                } else if (data.type === "help_resolved") {
                    helpSent = false;
                }
            } catch (e) {
                console.error("WS Message error:", e);
            }
        };

        ws.onclose = () => {
            console.log("WS closed, retrying...");
            setTimeout(initWebSocket, 3000);
        };
    }

    function sendChat() {
        if (!chatMessage.trim() || !ws || !attendee) return;

        const msg = {
            type: "chat",
            sender: attendee.name,
            message: chatMessage.trim(),
            timestamp: new Date().toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
            }),
        };

        ws.send(JSON.stringify(msg));
        chatMessage = "";
    }

    async function handleRequestHelp() {
        if (!attendee || helpSent) return;

        try {
            await requestHelp(id, attendee.id, currentStepIndex + 1);
            helpSent = true;
            setTimeout(() => (helpSent = false), 30000); // Prevent spamming
            alert("Help request sent to facilitator!");
        } catch (e) {
            alert("Failed to send help request.");
        }
    }

    async function handleFeedbackSubmit() {
        if (feedbackSubmitting || !attendee) return;
        feedbackSubmitting = true;
        try {
            await submitFeedback(id, {
                difficulty: feedbackDifficulty,
                satisfaction: feedbackSatisfaction,
                comments: feedbackComment,
                attendee_id: attendee.id,
            });
            feedbackSubmitted = true;
        } catch (e: any) {
            console.error("Feedback error", e);
            if (e.message === "ALREADY_SUBMITTED") {
                alert("You have already submitted feedback for this codelab.");
                feedbackSubmitted = true; // Show submitted state
            } else {
                alert("Failed to submit feedback");
            }
        } finally {
            feedbackSubmitting = false;
        }
    }

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

    let currentStep = $derived(steps[currentStepIndex]);

    $effect(() => {
        if (ws && ws.readyState === WebSocket.OPEN && attendee) {
            ws.send(
                JSON.stringify({
                    type: "step_progress",
                    attendee_id: attendee.id,
                    step_number: currentStepIndex + 1,
                }),
            );
        }
    });

    // @ts-ignore
    let renderedContent = $derived(
        currentStep
            ? (DOMPurify.sanitize(
                  marked.parse(currentStep.content_markdown) as string,
              ) as string)
            : "",
    );
    let progressPercent = $derived(
        steps.length > 0 ? ((currentStepIndex + 1) / steps.length) * 100 : 0,
    );
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
                onclick={() => (showSidebar = !showSidebar)}
                class="p-2 hover:bg-[#F1F3F4] rounded-full lg:hidden transition-colors"
                aria-label="Toggle sidebar"
            >
                {#if showSidebar}<X size={20} />{:else}<Menu size={20} />{/if}
            </button>
            <div class="flex items-center gap-3">
                <div
                    class="w-8 h-8 bg-[#4285F4] rounded flex items-center justify-center text-white font-bold"
                >
                    OC
                </div>
                <h1 class="font-bold text-lg hidden sm:block text-[#5F6368]">
                    Open-Codelabs
                </h1>
            </div>
        </div>

        <div class="flex-1 max-w-2xl px-8 text-center hidden md:block">
            <h2 class="font-medium text-[#3C4043] truncate text-base">
                {codelab?.title || "Loading..."}
            </h2>
        </div>

        <div class="flex items-center gap-4">
            <div
                class="hidden sm:flex items-center gap-2 text-[#5F6368] text-[11px] font-bold uppercase tracking-wider"
            >
                <Clock size={14} />
                <span>{steps.length * 5} mins remaining</span>
            </div>

            <button
                onclick={() => (showChat = !showChat)}
                class="p-2 hover:bg-[#F1F3F4] rounded-full relative transition-colors"
                title="Open Chat"
            >
                <MessageSquare
                    size={20}
                    class={showChat ? "text-[#4285F4]" : "text-[#5F6368]"}
                />
                {#if !showChat && messages.length > 0}
                    <span
                        class="absolute top-1 right-1 w-2 h-2 bg-[#EA4335] rounded-full border-2 border-white"
                    ></span>
                {/if}
            </button>

            <div
                class="w-8 h-8 rounded-full bg-[#E8EAED] flex items-center justify-center text-[#5F6368] border-2 border-white shadow-sm"
                title={attendee?.name}
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
                        onclick={() => {
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
                onclick={() => (showSidebar = false)}
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
                    <!-- ... (keep finish screen header) -->
                    <div
                        class="flex flex-col items-center justify-center py-20 text-center"
                        in:fly={{ y: 20, duration: 500 }}
                    >
                        <!-- ... (keep check circle and text) -->
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

                        {#if !feedbackSubmitted}
                            <div
                                class="max-w-md w-full bg-white border border-[#E8EAED] rounded-xl p-6 mb-8 text-left shadow-sm"
                            >
                                <h3
                                    class="font-bold text-lg mb-4 text-[#202124]"
                                >
                                    How was your experience?
                                </h3>

                                <!-- ... (keep feedback form inputs) -->
                                <div class="mb-4">
                                    <span
                                        class="block text-sm font-bold text-[#5F6368] mb-2"
                                        >Satisfaction</span
                                    >
                                    <div class="flex gap-2">
                                        {#each [1, 2, 3, 4, 5] as s}
                                            <button
                                                onclick={() =>
                                                    (feedbackSatisfaction = s)}
                                                class="p-1 rounded-lg transition-all hover:bg-yellow-50 focus:outline-none focus:ring-2 focus:ring-yellow-400"
                                            >
                                                <Star
                                                    size={28}
                                                    fill={feedbackSatisfaction >=
                                                    s
                                                        ? "#F9AB00"
                                                        : "none"}
                                                    class={feedbackSatisfaction >=
                                                    s
                                                        ? "text-[#F9AB00]"
                                                        : "text-[#BDC1C6]"}
                                                />
                                            </button>
                                        {/each}
                                    </div>
                                </div>

                                <div class="mb-4">
                                    <label
                                        for="difficulty-slider"
                                        class="block text-sm font-bold text-[#5F6368] mb-2"
                                        >Difficulty</label
                                    >
                                    <input
                                        id="difficulty-slider"
                                        type="range"
                                        min="1"
                                        max="5"
                                        step="1"
                                        bind:value={feedbackDifficulty}
                                        class="w-full accent-[#4285F4] h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                                    />
                                    <div
                                        class="flex justify-between text-xs text-[#9AA0A6] mt-2 font-medium"
                                    >
                                        <span>Too Easy</span>
                                        <span>Just Right</span>
                                        <span>Too Hard</span>
                                    </div>
                                </div>

                                <div class="mb-6">
                                    <label
                                        for="feedback-comments"
                                        class="block text-sm font-bold text-[#5F6368] mb-2"
                                        >Comments (Optional)</label
                                    >
                                    <textarea
                                        id="feedback-comments"
                                        bind:value={feedbackComment}
                                        class="w-full border border-[#DADCE0] rounded-lg p-3 text-sm focus:border-[#4285F4] outline-none transition-colors"
                                        rows="3"
                                        placeholder="Any additional feedback?"
                                    ></textarea>
                                </div>

                                <button
                                    onclick={handleFeedbackSubmit}
                                    disabled={feedbackSubmitting}
                                    class="w-full bg-[#4285F4] text-white py-3 rounded-full font-bold hover:bg-[#1A73E8] disabled:opacity-50 transition-all shadow-md active:scale-95"
                                >
                                    {feedbackSubmitting
                                        ? "Submitting..."
                                        : "Submit Feedback"}
                                </button>
                            </div>
                        {:else}
                            <div
                                class="bg-[#E6F4EA] text-[#137333] px-8 py-6 rounded-2xl mb-12 flex flex-col items-center gap-2 border border-[#CEEAD6]"
                            >
                                <CheckCircle2 size={32} />
                                <span class="font-bold text-lg"
                                    >Thank you for your feedback!</span
                                >
                            </div>
                        {/if}

                        <div class="flex flex-wrap justify-center gap-4">
                            <a
                                href="/codelabs/{id}/live"
                                class="bg-white border border-[#E8EAED] text-[#4285F4] hover:bg-[#F8F9FA] px-8 py-3 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                            >
                                <Users size={20} />
                                View Live Status
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

            <!-- Floating Help Button -->
            {#if !isFinished && !loading}
                <button
                    onclick={handleRequestHelp}
                    disabled={helpSent}
                    class="fixed bottom-24 right-8 p-4 border rounded-full shadow-lg hover:shadow-xl transition-all active:scale-95 group z-20 flex items-center gap-2 {helpSent
                        ? 'bg-[#34A853] border-[#34A853] text-white cursor-not-allowed'
                        : 'bg-white border-[#E8EAED] text-[#EA4335] hover:border-[#EA4335]'}"
                >
                    <div
                        class="p-2 rounded-full transition-colors {helpSent
                            ? 'bg-white/20'
                            : 'bg-[#EA4335]/10 group-hover:bg-[#EA4335] group-hover:text-white'}"
                    >
                        {#if helpSent}
                            <Check size={24} />
                        {:else}
                            <HelpCircle size={24} />
                        {/if}
                    </div>
                    {#if helpSent}
                        <span class="pr-2 text-sm font-bold animate-pulse"
                            >Help Requested ✓</span
                        >
                    {:else}
                        <span class="pr-2 text-sm font-bold">Request Help</span>
                    {/if}
                </button>
            {/if}

            <!-- Gemini Context Menu -->
            {#if showGeminiButton}
                <button
                    style="top: {geminiButtonPos.y}px; left: {geminiButtonPos.x}px;"
                    class="fixed z-50 bg-white text-[#4285F4] px-4 py-2 rounded-lg shadow-xl border border-[#D2E3FC] flex items-center gap-2 font-bold text-sm animate-in fade-in zoom-in-95 duration-200 hover:bg-[#F8F9FA] active:scale-95 cursor-pointer"
                    onmousedown={(e) => {
                        e.preventDefault();
                        showGeminiModal = true;
                        showGeminiButton = false;
                    }}
                >
                    <Sparkles size={16} />
                    Ask Gemini
                </button>
            {/if}

            {#if showGeminiModal}
                <AskGemini
                    context={selectedContext}
                    onClose={() => (showGeminiModal = false)}
                />
            {/if}
        </main>

        <!-- Chat Sidebar -->
        {#if showChat}
            <aside
                transition:fly={{ x: 320, duration: 300 }}
                class="fixed inset-y-0 right-0 z-40 w-80 bg-white border-l border-[#E8EAED] flex flex-col pt-16 lg:pt-0"
            >
                <div class="border-b border-[#E8EAED] bg-[#F8F9FA]">
                    <div class="p-4 flex items-center justify-between pb-2">
                        <h3
                            class="font-bold text-[#3C4043] flex items-center gap-2"
                        >
                            <MessageSquare size={18} />
                            {chatTab === "public"
                                ? $t("editor.public_chat")
                                : $t("editor.direct_messages")}
                        </h3>
                        <button
                            onclick={() => (showChat = false)}
                            class="p-1 hover:bg-[#E8EAED] rounded-full"
                        >
                            <X size={18} />
                        </button>
                    </div>

                    <div class="flex px-4 pb-2 gap-4">
                        <button
                            onclick={() => (chatTab = "public")}
                            class="pb-2 text-sm font-bold transition-all relative {chatTab ===
                            'public'
                                ? 'text-[#4285F4] border-b-2 border-[#4285F4]'
                                : 'text-[#5F6368] hover:text-[#3C4043]'}"
                        >
                            {$t("editor.public_chat")}
                        </button>
                        <button
                            onclick={() => {
                                chatTab = "direct";
                                hasNewDm = false;
                            }}
                            class="pb-2 text-sm font-bold transition-all relative {chatTab ===
                            'direct'
                                ? 'text-[#4285F4] border-b-2 border-[#4285F4]'
                                : 'text-[#5F6368] hover:text-[#3C4043]'}"
                        >
                            {$t("editor.direct_messages")}
                            {#if hasNewDm}
                                <span
                                    class="absolute -top-1 -right-2 w-2 h-2 bg-red-500 rounded-full border border-white"
                                ></span>
                            {/if}
                        </button>
                    </div>
                </div>

                <div
                    id="chat-messages"
                    class="flex-1 overflow-y-auto p-4 space-y-4"
                >
                    {#each filteredMessages as msg}
                        <div
                            class="flex flex-col {msg.self
                                ? 'items-end'
                                : 'items-start'}"
                        >
                            {#if chatTab === "public"}
                                <span
                                    class="text-[10px] text-[#5F6368] font-bold mb-1 ml-1 mr-1 uppercase tracking-tight"
                                >
                                    {msg.sender} &bull; {msg.time}
                                </span>
                            {:else}
                                <span
                                    class="text-[10px] text-[#5F6368] font-bold mb-1 ml-1 mr-1 uppercase tracking-tight {msg.self
                                        ? ''
                                        : 'text-[#D93025]'}"
                                >
                                    {msg.sender} &bull; {msg.time}
                                </span>
                            {/if}
                            <div
                                class="max-w-[90%] px-4 py-2.5 rounded-2xl text-sm leading-relaxed shadow-sm {msg.self
                                    ? 'bg-[#4285F4] text-white rounded-tr-none'
                                    : msg.type === 'dm'
                                      ? 'bg-[#FEEFC3] text-[#3C4043] rounded-tl-none border border-[#FAD2CF]'
                                      : 'bg-[#F1F3F4] text-[#3C4043] rounded-tl-none'}"
                            >
                                {msg.text}
                            </div>
                        </div>
                    {/each}
                    {#if filteredMessages.length === 0}
                        <div
                            class="flex flex-col items-center justify-center h-full text-center text-[#9AA0A6] px-6"
                        >
                            <div
                                class="w-12 h-12 bg-[#F1F3F4] rounded-full flex items-center justify-center mb-4"
                            >
                                <MessageSquare size={20} />
                            </div>
                            <p class="text-xs font-medium">
                                {chatTab === "public"
                                    ? "No public messages yet. Say hi!"
                                    : "No direct messages from the facilitator yet."}
                            </p>
                        </div>
                    {/if}
                </div>

                <div class="p-4 border-t border-[#E8EAED] bg-white">
                    <form
                        onsubmit={(e) => {
                            e.preventDefault();
                            sendChat();
                        }}
                        class="relative"
                    >
                        <input
                            type="text"
                            bind:value={chatMessage}
                            placeholder="Type a message..."
                            class="w-full pl-4 pr-12 py-3 bg-[#F8F9FA] border border-[#DADCE0] rounded-xl outline-none focus:border-[#4285F4] transition-all text-sm"
                        />
                        <button
                            type="submit"
                            class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-[#4285F4] hover:bg-[#4285F4] hover:text-white rounded-lg transition-all"
                        >
                            <Send size={18} />
                        </button>
                    </form>
                </div>
            </aside>
        {/if}
    </div>

    <!-- Footer Navigation -->
    <footer
        class="h-20 border-t border-[#E8EAED] bg-white flex items-center justify-center px-8 sticky bottom-0 z-30"
    >
        <div class="max-w-3xl w-full flex justify-between items-center">
            <button
                onclick={prevStep}
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
                    onclick={finishCodelab}
                    class="bg-[#1E8E3E] hover:bg-[#178037] text-white px-10 py-2.5 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                >
                    Finish
                </button>
            {:else if isFinished}
                <div class="w-[100px]"></div>
            {:else}
                <button
                    onclick={nextStep}
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
        /* Inline code (not in pre) - subtle gray background */
        background-color: rgba(175, 184, 193, 0.2);
        padding: 0.2em 0.4em;
        border-radius: 6px;
        color: #24292e;
    }
    :global(.markdown-body pre code) {
        /* Code in pre blocks - let Prism handle it */
        background-color: transparent;
        padding: 0;
        color: inherit;
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
