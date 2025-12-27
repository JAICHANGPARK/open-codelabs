<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import {
        getCodelab,
        saveSteps,
        exportCodelab,
        getAttendees,
        getHelpRequests,
        resolveHelpRequest,
        getWsUrl,
        getChatHistory,
        ASSET_URL,
        uploadImage,
        getFeedback,
        type Codelab,
        type Step,
        type Attendee,
        type HelpRequest,
        type ChatMessage,
        type Feedback,
    } from "$lib/api";
    import { streamGeminiResponseRobust } from "$lib/gemini";
    import { decrypt } from "$lib/crypto";
    // @ts-ignore
    import QRCode from "svelte-qrcode";
    import { marked } from "marked";
    import { markedHighlight } from "marked-highlight";
    import hljs from "highlight.js";
    import "highlight.js/styles/github.css";
    import DOMPurify from "dompurify";
    // ... icons imports ...
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
        Users,
        Bell,
        MessageSquare,
        Send,
        Copy,
        Check,
        X,
        FileText,
        Sparkles,
        Loader2,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let id = page.params.id as string;
    let activeStepIndex = $state(0);

    // Initialize mode from URL or default to 'edit'
    let initialMode = page.url.searchParams.get("mode");
    let mode = $state<"edit" | "preview" | "live" | "feedback">(
        initialMode === "preview" ||
            initialMode === "live" ||
            initialMode === "feedback"
            ? initialMode
            : "edit",
    );

    let isSaving = $state(false);
    let codelab = $state<Codelab | null>(null);
    let steps = $state<Step[]>([]);
    let saveSuccess = $state(false);
    let loading = $state(true);
    let copySuccess = $state(false);

    let attendees = $state<Attendee[]>([]);
    let helpRequests = $state<HelpRequest[]>([]);
    let feedbacks = $state<Feedback[]>([]); // Feedback
    let ws = $state<WebSocket | null>(null);
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
    let chatTab = $state<"public" | "direct">("public");
    let dmTarget = $state<Attendee | null>(null);
    let dmMessage = $state("");
    let fileInput: HTMLInputElement; // File input ref

    // AI State
    let geminiApiKey = $state("");
    let showAiMenu = $state(false);
    let menuPos = $state({ x: 0, y: 0 });
    let selectedText = $state("");
    let selectionRange = $state<{ start: number; end: number } | null>(null);
    let aiLoading = $state(false);

    // Drag & Drop State
    let draggedStepIndex = $state<number | null>(null);
    let dragOverIndex = $state<number | null>(null);

    let filteredMessages = $derived(
        chatTab === "public"
            ? messages.filter((m) => m.type === "chat")
            : messages.filter((m) => m.type === "dm"),
    );

    // Sync mode to URL and load data
    $effect(() => {
        const url = new URL(window.location.href);
        if (url.searchParams.get("mode") !== mode) {
            url.searchParams.set("mode", mode);
            window.history.replaceState({}, "", url);
        }

        if (mode === "feedback") {
            loadFeedback();
        } else if (mode === "live") {
            refreshLiveData();
            scrollToBottom();
        }
    });

    onMount(async () => {
        // Configure marked with highlight.js
        marked.use(
            markedHighlight({
                emptyLangClass: "hljs",
                langPrefix: "hljs language-",
                highlight(code, lang) {
                    const language = hljs.getLanguage(lang)
                        ? lang
                        : "plaintext";
                    return hljs.highlight(code, { language }).value;
                },
            }),
        );

        try {
            const data = await getCodelab(id);
            codelab = data[0];
            steps = data[1];

            // Initial fetch of live data
            await refreshLiveData();
            await loadChatHistory();
            initWebSocket();

            // Load API Key
            const encryptedKey = localStorage.getItem("gemini_api_key");
            if (encryptedKey) {
                const decrypted = decrypt(encryptedKey);
                if (decrypted) geminiApiKey = decrypted;
            }

            document.addEventListener("selectionchange", handleSelectionChange);
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    // Cleanup
    $effect(() => {
        return () => {
            document.removeEventListener(
                "selectionchange",
                handleSelectionChange,
            );
        };
    });

    function handleSelectionChange() {
        if (mode !== "edit" || aiLoading) return;

        const activeElement = document.activeElement as HTMLTextAreaElement;
        if (activeElement && activeElement.tagName === "TEXTAREA") {
            const start = activeElement.selectionStart;
            const end = activeElement.selectionEnd;

            if (start !== end) {
                // We have a selection
                const text = activeElement.value.substring(start, end);
                if (text.trim().length > 0) {
                    // Get coordinates for the menu
                    // This is tricky with textarea. simpler to just show near mouse or fixed?
                    // Let's use mouseup to get coordinates, selectionchange for state

                    selectedText = text;
                    selectionRange = { start, end };
                    return;
                }
            }
        }
        // If we are here, no valid selection or lost focus
        if (!showAiMenu) {
            // Only hide if not already open/interacting?
            // Actually, we should use handleMouseUp for the UI part
        }
    }

    function handleMouseUp(e: MouseEvent) {
        if (mode !== "edit") return;

        // Timeout to let selection settle
        setTimeout(() => {
            const activeElement = document.activeElement as HTMLTextAreaElement;
            if (
                activeElement &&
                activeElement.tagName === "TEXTAREA" &&
                activeElement.contains(e.target as Node)
            ) {
                const start = activeElement.selectionStart;
                const end = activeElement.selectionEnd;

                if (start !== end) {
                    const text = activeElement.value.substring(start, end);
                    if (text.trim().length > 0) {
                        selectedText = text;
                        selectionRange = { start, end };

                        // Calculate position relative to viewport
                        // A bit hacky for textarea, but we can just use mouse coordinates
                        menuPos = { x: e.clientX, y: e.clientY - 40 };
                        showAiMenu = true;
                        return;
                    }
                }
            }
            // Hide if clicked elsewhere and not loading
            if (!aiLoading) {
                showAiMenu = false;
            }
        }, 10);
    }

    async function improveWithAi() {
        if (!geminiApiKey) {
            alert("Please set your Gemini API Key in the Dashboard first.");
            return;
        }
        if (!selectedText || !selectionRange) return;

        aiLoading = true;
        showAiMenu = false; // Hide menu, show loading state inline maybe?

        // We will replace the text with a placeholder or just start streaming content into it?
        // Let's stream directly into the replacement.

        const prompt = `Improve the following technical writing/markdown content. Make it clearer, correct grammar, and better formatted. Maintain the original meaning. Only return the improved content, no explanations.\n\nContent:\n${selectedText}`;
        const systemPrompt = "You are a helpful technical editor.";

        let newContent = "";

        try {
            // 1. We replace the selection with "Generating..." or keep it and replace at end?
            // User requested "stream response". Replacing in real-time is cool.

            // Initial replacement to clear selection
            const textarea = document.querySelector("textarea");
            if (textarea) {
                textarea.setRangeText(
                    "",
                    selectionRange.start,
                    selectionRange.end,
                    "select",
                );
            }

            let currentCursor = selectionRange.start;

            const stream = streamGeminiResponseRobust(prompt, systemPrompt, {
                apiKey: geminiApiKey,
            });

            for await (const chunk of stream) {
                steps[activeStepIndex].content_markdown =
                    steps[activeStepIndex].content_markdown.substring(
                        0,
                        currentCursor,
                    ) +
                    chunk +
                    steps[activeStepIndex].content_markdown.substring(
                        currentCursor,
                    );

                currentCursor += chunk.length;

                // Force update textarea if needed? Svelte bind should handle it but might lose cursor?
                // We need to keep focus?
            }

            // Update selection to the new content
            selectionRange = {
                start: selectionRange.start,
                end: currentCursor,
            };
        } catch (e: any) {
            console.error(e);
            alert("AI Improvement failed: " + e.message);
            // Restore? Pushing undo stack would be nice but complex.
            // For now, assuming it just appended or failed mid-way.
        } finally {
            aiLoading = false;
        }
    }

    async function loadFeedback() {
        try {
            feedbacks = await getFeedback(id);
        } catch (e) {
            console.error("Failed to load feedback", e);
        }
    }

    async function loadChatHistory() {
        try {
            const history = await getChatHistory(id);
            messages = history.map((msg) => {
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
                        self: msg.sender_name === "Facilitator",
                        type: "chat",
                    };
                } else {
                    // DM
                    if (msg.sender_name === "Facilitator") {
                        const target = attendees.find(
                            (a) => a.id === msg.target_id,
                        );
                        return {
                            sender: `To: ${target?.name || msg.target_id}`,
                            text: msg.message,
                            time: timeStr,
                            self: true,
                            type: "dm",
                        };
                    } else {
                        return {
                            sender: `[DM] ${msg.sender_name}`,
                            text: msg.message,
                            time: timeStr,
                            self: false,
                            type: "dm",
                        };
                    }
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

    async function refreshLiveData() {
        try {
            const [att, help] = await Promise.all([
                getAttendees(id),
                getHelpRequests(id),
            ]);
            attendees = att;
            helpRequests = help;
        } catch (e) {
            console.error("Failed to refresh live data:", e);
        }
    }

    function scrollToBottom() {
        setTimeout(() => {
            const chatContainer = document.getElementById("chat-messages");
            if (chatContainer)
                chatContainer.scrollTop = chatContainer.scrollHeight;
        }, 100);
    }

    function initWebSocket() {
        const wsUrl = getWsUrl(id);
        const newWs = new WebSocket(wsUrl);

        newWs.onopen = () => {
            // Identify as facilitator
            newWs.send(JSON.stringify({ type: "facilitator" }));
        };

        newWs.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === "chat") {
                    messages = [
                        ...messages,
                        {
                            sender: data.sender,
                            text: data.message,
                            time: data.timestamp,
                            self: data.sender === "Facilitator",
                            type: "chat",
                        },
                    ];
                    if (chatTab === "public") scrollToBottom();
                } else if (data.type === "dm") {
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
                    if (chatTab === "direct") scrollToBottom();
                } else if (
                    data.type === "help_request" ||
                    data.type === "help_resolved"
                ) {
                    refreshLiveData();
                } else if (data.type === "step_progress") {
                    attendees = attendees.map((a) =>
                        a.id === data.attendee_id
                            ? { ...a, current_step: data.step_number }
                            : a,
                    );
                }
            } catch (e) {
                console.error("WS error:", e);
            }
        };

        newWs.onclose = () => {
            setTimeout(initWebSocket, 3000);
        };
        ws = newWs;
    }

    function sendBroadcast() {
        if (!chatMessage.trim() || !ws) return;
        const msg = {
            type: "chat",
            sender: "Facilitator",
            message: chatMessage.trim(),
            timestamp: new Date().toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
            }),
        };
        ws.send(JSON.stringify(msg));
        chatMessage = "";
    }

    function sendDM() {
        if (!dmTarget || !dmMessage.trim() || !ws) return;
        const msg = {
            type: "dm",
            target_id: dmTarget.id,
            sender: "Facilitator",
            message: dmMessage.trim(),
            timestamp: new Date().toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
            }),
        };
        ws.send(JSON.stringify(msg));

        // Also add to local messages for visibility
        messages = [
            ...messages,
            {
                sender: `To: ${dmTarget.name}`,
                text: dmMessage.trim(),
                time: msg.timestamp,
                self: true,
                type: "dm",
            },
        ];

        dmMessage = "";
        dmTarget = null;
        scrollToBottom();
    }

    async function handleResolveHelp(helpId: string) {
        try {
            await resolveHelpRequest(id, helpId);
            // WebSocket will trigger refresh for peers, but we refresh locally too
            await refreshLiveData();
        } catch (e) {
            alert("Failed to resolve help request");
        }
    }

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

    function removeStep(index: number) {
        if (!confirm("Are you sure you want to delete this step?")) return;
        steps = steps.filter((_, i) => i !== index);
        if (activeStepIndex >= steps.length) {
            activeStepIndex = Math.max(0, steps.length - 1);
        }
    }

    // Drag & Drop Handlers
    function handleDragStart(e: DragEvent, index: number) {
        draggedStepIndex = index;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = "move";
        }
    }

    function handleDragOver(e: DragEvent, index: number) {
        e.preventDefault();
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = "move";
        }
        dragOverIndex = index;
    }

    function handleDragLeave() {
        dragOverIndex = null;
    }

    function handleDrop(e: DragEvent, dropIndex: number) {
        e.preventDefault();

        if (draggedStepIndex === null || draggedStepIndex === dropIndex) {
            draggedStepIndex = null;
            dragOverIndex = null;
            return;
        }

        // Reorder steps array
        const newSteps = [...steps];
        const [draggedStep] = newSteps.splice(draggedStepIndex, 1);
        newSteps.splice(dropIndex, 0, draggedStep);

        steps = newSteps;

        // Update active step index if needed
        if (activeStepIndex === draggedStepIndex) {
            activeStepIndex = dropIndex;
        } else if (
            draggedStepIndex < activeStepIndex &&
            dropIndex >= activeStepIndex
        ) {
            activeStepIndex--;
        } else if (
            draggedStepIndex > activeStepIndex &&
            dropIndex <= activeStepIndex
        ) {
            activeStepIndex++;
        }

        draggedStepIndex = null;
        dragOverIndex = null;
    }

    function handleDragEnd() {
        draggedStepIndex = null;
        dragOverIndex = null;
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

        // Handle image special case
        if (type === "image") {
            fileInput?.click();
            return;
        }

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

    async function handleFileSelect(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files[0]) {
            await uploadAndInsertImage(input.files[0]);
        }
        input.value = ""; // reset
    }

    async function uploadAndInsertImage(file: File) {
        try {
            const { url } = await uploadImage(file);
            const textarea = document.querySelector("textarea");
            if (!textarea) return;

            const start = textarea.selectionStart;
            const end = textarea.selectionEnd;
            const text = steps[activeStepIndex].content_markdown;
            const fullUrl = url.startsWith("http") ? url : `${ASSET_URL}${url}`;
            const replacement = `![image](${fullUrl})`;

            steps[activeStepIndex].content_markdown =
                text.substring(0, start) + replacement + text.substring(end);

            setTimeout(() => {
                textarea.focus();
                const newCursorPos = start + replacement.length;
                textarea.setSelectionRange(newCursorPos, newCursorPos);
            }, 0);
        } catch (e) {
            console.error(e);
            alert("Image upload failed");
        }
    }

    async function handlePaste(event: ClipboardEvent) {
        const items = event.clipboardData?.items;
        if (!items) return;

        for (const item of items) {
            if (item.type.indexOf("image") !== -1) {
                const file = item.getAsFile();
                if (file) {
                    event.preventDefault();
                    await uploadAndInsertImage(file);
                }
            }
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (mode !== "edit") return;

        if (e.metaKey || e.ctrlKey) {
            switch (e.key.toLowerCase()) {
                case "b":
                    e.preventDefault();
                    insertMarkdown("bold");
                    break;
                case "i":
                    e.preventDefault();
                    insertMarkdown("italic");
                    break;
            }
        }
    }

    async function copyUrl() {
        const url = `${window.location.origin}/codelabs/${id}`;

        try {
            if (navigator.clipboard && navigator.clipboard.writeText) {
                await navigator.clipboard.writeText(url);
                copySuccess = true;
            } else {
                throw new Error("clipboard API unavailable");
            }
        } catch (e) {
            // Fallback for non-secure contexts or older browsers
            try {
                const textArea = document.createElement("textarea");
                textArea.value = url;
                textArea.style.position = "fixed";
                textArea.style.left = "-9999px";
                textArea.style.top = "0";
                document.body.appendChild(textArea);
                textArea.focus();
                textArea.select();
                const successful = document.execCommand("copy");
                document.body.removeChild(textArea);
                if (successful) {
                    copySuccess = true;
                }
            } catch (err) {
                console.error("Fallback copy failed", err);
            }
        }

        if (copySuccess) {
            setTimeout(() => (copySuccess = false), 2000);
        }
    }

    let currentStep = $derived(steps[activeStepIndex]);
    let renderedContent = $derived.by(() => {
        if (!currentStep) return "";
        try {
            // @ts-ignore
            return DOMPurify.sanitize(
                marked.parse(currentStep.content_markdown) as string,
            );
        } catch (e) {
            console.error("Markdown parse error", e);
            return "Error parsing markdown";
        }
    });

    let attendeeUrl = $derived(
        `${typeof window !== "undefined" ? window.location.origin : ""}/codelabs/${id}`,
    );
</script>

<svelte:window
    onkeydown={(e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            e.preventDefault();
            handleSave();
        }
    }}
/>

<div class="min-h-screen bg-[#F8F9FA] flex flex-col font-sans text-[#3C4043]">
    <header
        class="bg-white border-b border-[#E8EAED] py-4 px-8 sticky top-0 z-30 shadow-sm"
    >
        <div class="max-w-7xl mx-auto flex justify-between items-center gap-3">
            <div class="flex items-center gap-2 md:gap-6 flex-1 min-w-0">
                <a
                    href="/admin"
                    class="text-[#5F6368] hover:text-[#202124] hover:bg-[#F1F3F4] p-2 rounded-full transition-all shrink-0"
                    aria-label="Back to dashboard"
                >
                    <ChevronLeft size={24} />
                </a>
                <div class="min-w-0 flex-1">
                    {#if loading}
                        <div
                            class="h-6 w-32 md:w-48 bg-[#F1F3F4] animate-pulse rounded"
                        ></div>
                    {:else}
                        <h1
                            class="text-base md:text-xl font-bold text-[#202124] flex items-center gap-2 truncate"
                        >
                            <span class="truncate">{codelab?.title}</span>
                            <a
                                href="/codelabs/{id}"
                                target="_blank"
                                class="text-[#4285F4] hover:text-[#1A73E8] shrink-0"
                                title={$t("editor.view_live")}
                            >
                                <ExternalLink size={16} />
                            </a>
                        </h1>
                        <p
                            class="text-xs text-[#5F6368] font-medium mt-0.5 hidden sm:block"
                        >
                            ID: {id} &bull; {$t("common.facilitator")} Mode
                        </p>
                    {/if}
                </div>
            </div>
            <div class="flex items-center gap-2 md:gap-4 shrink-0">
                <button
                    onclick={handleExport}
                    class="p-2 md:p-2.5 text-[#5F6368] hover:text-[#4285F4] hover:bg-[#E8F0FE] rounded-full transition-all"
                    title="Export Codelab"
                >
                    <Download size={20} class="md:hidden" />
                    <Download size={24} class="hidden md:block" />
                </button>
                <div
                    class="flex bg-[#F1F3F4] p-1 rounded-full border border-[#E8EAED]"
                >
                    <button
                        onclick={() => (mode = "edit")}
                        class="px-2 md:px-5 py-1.5 rounded-full flex items-center gap-1 md:gap-2 text-xs md:text-sm font-bold transition-all {mode ===
                        'edit'
                            ? 'bg-white shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] hover:text-[#202124]'}"
                    >
                        <Edit3 size={16} />
                        <span class="hidden sm:inline">Edit</span>
                    </button>
                    <button
                        onclick={() => (mode = "preview")}
                        class="px-2 md:px-5 py-1.5 rounded-full flex items-center gap-1 md:gap-2 text-xs md:text-sm font-bold transition-all {mode ===
                        'preview'
                            ? 'bg-white shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] hover:text-[#202124]'}"
                    >
                        <Eye size={16} />
                        <span class="hidden sm:inline">Preview</span>
                    </button>
                    <button
                        onclick={() => (mode = "live")}
                        class="px-2 md:px-5 py-1.5 rounded-full flex items-center gap-1 md:gap-2 text-xs md:text-sm font-bold transition-all {mode ===
                        'live'
                            ? 'bg-white shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] hover:text-[#202124]'}"
                    >
                        <Users size={16} />
                        <span class="hidden sm:inline">Live</span>
                        {#if helpRequests.length > 0}
                            <span class="w-2 h-2 bg-[#EA4335] rounded-full"
                            ></span>
                        {/if}
                    </button>
                    <button
                        onclick={() => (mode = "feedback")}
                        class="px-5 py-1.5 rounded-full flex items-center gap-2 text-sm font-bold transition-all {mode ===
                        'feedback'
                            ? 'bg-white shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] hover:text-[#202124]'}"
                    >
                        <MessageSquare size={16} /> Feedback
                    </button>
                </div>
                <button
                    onclick={handleSave}
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
                            onclick={addStep}
                            class="text-[#4285F4] hover:bg-[#E8F0FE] p-1.5 rounded-full transition-colors"
                            title={$t("editor.add_step")}
                        >
                            <Plus size={18} />
                        </button>
                    </div>
                    <div class="max-h-[50vh] overflow-y-auto">
                        {#each steps as step, i}
                            <div
                                class="group relative {dragOverIndex === i
                                    ? 'border-t-4 border-[#4285F4]'
                                    : ''}"
                                draggable="true"
                                ondragstart={(e) => handleDragStart(e, i)}
                                ondragover={(e) => handleDragOver(e, i)}
                                ondragleave={handleDragLeave}
                                ondrop={(e) => handleDrop(e, i)}
                                ondragend={handleDragEnd}
                            >
                                <button
                                    onclick={() => (activeStepIndex = i)}
                                    class="w-full text-left px-5 py-4 hover:bg-[#F8F9FA] transition-all flex items-start gap-4 border-l-4 cursor-pointer {activeStepIndex ===
                                    i
                                        ? 'border-[#4285F4] bg-[#E8F0FE]/30'
                                        : 'border-transparent'} {draggedStepIndex ===
                                    i
                                        ? 'opacity-50'
                                        : ''}"
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
                                            : 'text-[#5F6368]'} line-clamp-1 pt-0.5 pr-6"
                                        >{step.title}</span
                                    >
                                </button>
                                <button
                                    onclick={() => removeStep(i)}
                                    class="absolute right-3 top-1/2 -translate-y-1/2 p-1.5 text-[#BDC1C6] hover:text-red-500 hover:bg-red-50 rounded-lg opacity-0 group-hover:opacity-100 transition-all"
                                    title="Delete Step"
                                >
                                    <Trash2 size={14} />
                                </button>
                            </div>
                        {/each}
                    </div>

                    <div
                        class="p-6 border-t border-[#F1F3F4] bg-[#F8F9FA]/50 flex flex-col items-center"
                    >
                        <div
                            class="bg-white p-3 rounded-2xl border border-[#E8EAED] shadow-sm mb-4"
                        >
                            <QRCode value={attendeeUrl} size={140} />
                        </div>
                        <p
                            class="text-[11px] text-[#5F6368] text-center uppercase tracking-widest font-bold mb-3"
                        >
                            {$t("editor.attendee_access")}
                        </p>

                        <div
                            class="w-full flex items-center gap-2 p-2 bg-white border border-[#E8EAED] rounded-xl shadow-sm overflow-hidden"
                        >
                            <input
                                type="text"
                                readonly
                                value={attendeeUrl}
                                class="flex-1 text-[10px] text-[#5F6368] font-mono outline-none bg-transparent overflow-hidden text-ellipsis"
                            />
                            <button
                                onclick={copyUrl}
                                class="p-2 hover:bg-[#F1F3F4] rounded-lg transition-colors {copySuccess
                                    ? 'text-[#1E8E3E]'
                                    : 'text-[#4285F4]'}"
                                title="Copy URL"
                            >
                                {#if copySuccess}
                                    <Check size={14} />
                                {:else}
                                    <Copy size={14} />
                                {/if}
                            </button>
                        </div>
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
                                        onclick={() => insertMarkdown("h1")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Heading"
                                        ><Heading1 size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("bold")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Bold"><Bold size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("italic")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Italic"
                                        ><Italic size={20} /></button
                                    >
                                    <div
                                        class="w-px h-6 bg-[#DADCE0] mx-1"
                                    ></div>
                                    <button
                                        onclick={() => insertMarkdown("list")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="List"><List size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("code")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Code Block"
                                        ><Code size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("image")}
                                        class="p-2 hover:bg-white rounded-lg transition-colors text-[#5F6368]"
                                        title="Image"
                                        ><ImageIcon size={20} /></button
                                    >
                                </div>
                                <input
                                    type="file"
                                    accept="image/*"
                                    class="hidden"
                                    bind:this={fileInput}
                                    onchange={handleFileSelect}
                                />
                                <textarea
                                    bind:value={
                                        steps[activeStepIndex].content_markdown
                                    }
                                    onkeydown={handleKeydown}
                                    onpaste={handlePaste}
                                    class="w-full flex-1 min-h-[50vh] outline-none text-[#3C4043] font-mono text-base leading-relaxed resize-none bg-transparent"
                                    placeholder="Write your markdown here..."
                                    onmouseup={handleMouseUp}
                                ></textarea>

                                {#if showAiMenu}
                                    <div
                                        class="fixed z-50 animate-in fade-in zoom-in-95 duration-200"
                                        style="top: {menuPos.y}px; left: {menuPos.x}px;"
                                    >
                                        <button
                                            onclick={improveWithAi}
                                            class="bg-white text-[#4285F4] px-4 py-2 rounded-full shadow-xl border border-[#D2E3FC] flex items-center gap-2 font-bold text-sm hover:bg-[#F8F9FA] transition-all hover:scale-105"
                                        >
                                            <Sparkles size={16} />
                                            Improve with Gemini
                                        </button>
                                    </div>
                                {/if}

                                {#if aiLoading}
                                    <div
                                        class="fixed z-50 top-4 right-4 bg-white px-4 py-3 rounded-xl shadow-lg border border-[#E8EAED] flex items-center gap-3 animate-in slide-in-from-right"
                                    >
                                        <Loader2
                                            class="animate-spin text-[#4285F4]"
                                            size={20}
                                        />
                                        <div>
                                            <p
                                                class="text-sm font-bold text-[#202124]"
                                            >
                                                Gemini is writing...
                                            </p>
                                            <p class="text-xs text-[#5F6368]">
                                                Improving your content
                                            </p>
                                        </div>
                                    </div>
                                {/if}
                            {:else if mode === "preview"}
                                <div
                                    class="prose prose-blue max-w-none flex-1 markdown-body"
                                    in:fade
                                >
                                    {@html renderedContent}
                                </div>
                            {:else if mode === "live"}
                                <div
                                    class="grid grid-cols-2 gap-8 h-full"
                                    in:fade
                                >
                                    <!-- Left: Activity & Help -->
                                    <div class="space-y-6 flex flex-col h-full">
                                        <div
                                            class="bg-white border border-[#E8EAED] rounded-2xl overflow-hidden shadow-sm flex flex-col"
                                        >
                                            <div
                                                class="p-4 bg-red-50 border-b border-red-100 flex items-center gap-2"
                                            >
                                                <Bell
                                                    size={18}
                                                    class="text-[#EA4335]"
                                                />
                                                <h3
                                                    class="font-bold text-[#EA4335]"
                                                >
                                                    Help Requests ({helpRequests.length})
                                                </h3>
                                            </div>
                                            <div
                                                class="p-4 space-y-3 max-h-60 overflow-y-auto"
                                            >
                                                {#each helpRequests as hr}
                                                    <div
                                                        class="p-3 bg-red-50/50 rounded-xl border border-red-100 flex justify-between items-center"
                                                        in:slide
                                                    >
                                                        <div>
                                                            <p
                                                                class="font-bold text-[#202124] text-sm"
                                                            >
                                                                {hr.attendee_name}
                                                            </p>
                                                            <p
                                                                class="text-xs text-[#EA4335]"
                                                            >
                                                                Stuck on Step {hr.step_number}
                                                            </p>
                                                        </div>
                                                        <button
                                                            onclick={() =>
                                                                handleResolveHelp(
                                                                    hr.id,
                                                                )}
                                                            class="text-xs font-bold text-white bg-[#EA4335] px-3 py-1.5 rounded-full hover:bg-[#D93025] transition-colors"
                                                            >Resolve</button
                                                        >
                                                    </div>
                                                {:else}
                                                    <p
                                                        class="text-center py-6 text-[#9AA0A6] text-sm"
                                                    >
                                                        No pending help requests
                                                    </p>
                                                {/each}
                                            </div>
                                        </div>

                                        <div
                                            class="flex-1 bg-white border border-[#E8EAED] rounded-2xl overflow-hidden shadow-sm flex flex-col min-h-[400px]"
                                        >
                                            <div
                                                class="p-4 bg-[#F8F9FA] border-b border-[#E8EAED] flex items-center gap-2"
                                            >
                                                <Users
                                                    size={18}
                                                    class="text-[#4285F4]"
                                                />
                                                <h3
                                                    class="font-bold text-[#3C4043]"
                                                >
                                                    Active Attendees ({attendees.length})
                                                </h3>
                                            </div>
                                            <div
                                                class="p-4 space-y-2 overflow-y-auto"
                                            >
                                                {#each attendees as attendee}
                                                    <div
                                                        class="flex items-center justify-between p-2 hover:bg-[#F8F9FA] rounded-lg transition-colors group"
                                                    >
                                                        <div
                                                            class="flex items-center gap-3"
                                                        >
                                                            <div
                                                                class="w-8 h-8 rounded-full bg-[#E8EAED] flex items-center justify-center text-[#5F6368] text-xs font-bold uppercase"
                                                            >
                                                                {attendee.name.charAt(
                                                                    0,
                                                                )}
                                                            </div>
                                                            <div>
                                                                <p
                                                                    class="text-sm font-bold text-[#202124]"
                                                                >
                                                                    {attendee.name}
                                                                </p>
                                                                <p
                                                                    class="text-[10px] text-[#9AA0A6]"
                                                                >
                                                                    Code: {attendee.code}
                                                                    {#if attendee.current_step}
                                                                        &bull;
                                                                        Step {attendee.current_step}
                                                                    {/if}
                                                                </p>
                                                            </div>
                                                        </div>
                                                        <button
                                                            onclick={() =>
                                                                (dmTarget =
                                                                    attendee)}
                                                            class="p-2 text-[#4285F4] hover:bg-[#E8F0FE] rounded-lg opacity-0 group-hover:opacity-100 transition-all"
                                                            title="Message"
                                                        >
                                                            <MessageSquare
                                                                size={16}
                                                            />
                                                        </button>
                                                    </div>
                                                {/each}
                                            </div>
                                        </div>
                                    </div>

                                    <!-- Right: Live Chat -->
                                    <div
                                        class="bg-white border border-[#E8EAED] rounded-2xl overflow-hidden shadow-sm flex flex-col h-full min-h-[600px]"
                                    >
                                        <div
                                            class="flex border-b border-[#E8EAED]"
                                        >
                                            <button
                                                onclick={() =>
                                                    (chatTab = "public")}
                                                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab ===
                                                'public'
                                                    ? 'text-[#4285F4] border-b-2 border-[#4285F4] bg-[#F8F9FA]'
                                                    : 'text-[#5F6368] hover:bg-[#F1F3F4]'}"
                                            >
                                                <Users size={16} /> Public Chat
                                            </button>
                                            <button
                                                onclick={() =>
                                                    (chatTab = "direct")}
                                                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab ===
                                                'direct'
                                                    ? 'text-[#4285F4] border-b-2 border-[#4285F4] bg-[#F8F9FA]'
                                                    : 'text-[#5F6368] hover:bg-[#F1F3F4]'}"
                                            >
                                                <MessageSquare size={16} /> Direct
                                                Messages
                                            </button>
                                        </div>

                                        <div
                                            class="flex-1 p-4 space-y-4 overflow-y-auto bg-[#F8F9FA]"
                                            id="chat-messages"
                                        >
                                            {#each filteredMessages as msg}
                                                <div
                                                    class="flex flex-col {msg.self
                                                        ? 'items-end'
                                                        : 'items-start'}"
                                                >
                                                    <span
                                                        class="text-[10px] text-[#5F6368] font-bold mb-1 mx-1 uppercase"
                                                        >{msg.sender} &bull; {msg.time}</span
                                                    >
                                                    <div
                                                        class="px-4 py-2 rounded-2xl text-sm max-w-[85%] whitespace-pre-wrap break-words {msg.self
                                                            ? 'bg-[#4285F4] text-white rounded-tr-none'
                                                            : 'bg-white border border-[#E8EAED] text-[#3C4043] shadow-sm rounded-tl-none'}"
                                                    >
                                                        {msg.text}
                                                    </div>
                                                </div>
                                            {/each}
                                            {#if filteredMessages.length === 0}
                                                <div
                                                    class="h-full flex flex-col items-center justify-center text-[#9AA0A6]"
                                                >
                                                    <MessageSquare
                                                        size={32}
                                                        class="mb-2 opacity-50"
                                                    />
                                                    <p class="text-sm">
                                                        No messages yet
                                                    </p>
                                                </div>
                                            {/if}
                                        </div>

                                        <div
                                            class="p-4 border-t border-[#E8EAED] bg-white"
                                        >
                                            <form
                                                onsubmit={(e) => {
                                                    e.preventDefault();
                                                    if (chatTab === "public") {
                                                        sendBroadcast();
                                                    } else {
                                                        sendDM();
                                                    }
                                                }}
                                                class="relative"
                                            >
                                                {#if chatTab === "direct" && !dmTarget}
                                                    <div
                                                        class="absolute inset-0 bg-white/80 z-10 flex items-center justify-center text-sm text-[#5F6368] font-bold"
                                                    >
                                                        Select an attendee to
                                                        message
                                                    </div>
                                                {/if}
                                                <div
                                                    class="flex items-center gap-2"
                                                >
                                                    {#if chatTab === "direct" && dmTarget}
                                                        <span
                                                            class="bg-[#E8F0FE] text-[#1967D2] px-2 py-1 rounded text-xs font-bold whitespace-nowrap"
                                                        >
                                                            To: {dmTarget.name}
                                                        </span>
                                                    {/if}
                                                    {#if chatTab === "public"}
                                                        <input
                                                            type="text"
                                                            bind:value={
                                                                chatMessage
                                                            }
                                                            placeholder="Broadcast to all..."
                                                            class="flex-1 pl-4 pr-12 py-3 bg-[#F8F9FA] border border-[#DADCE0] rounded-xl outline-none focus:border-[#4285F4] text-sm"
                                                        />
                                                    {:else}
                                                        <input
                                                            type="text"
                                                            bind:value={
                                                                dmMessage
                                                            }
                                                            placeholder="Type a message..."
                                                            class="flex-1 pl-4 pr-12 py-3 bg-[#F8F9FA] border border-[#DADCE0] rounded-xl outline-none focus:border-[#4285F4] text-sm"
                                                        />
                                                    {/if}
                                                    <button
                                                        type="submit"
                                                        class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-[#4285F4] hover:bg-[#E8F0FE] rounded-lg transition-all"
                                                        disabled={chatTab ===
                                                            "direct" &&
                                                            !dmTarget}
                                                    >
                                                        <Send size={18} />
                                                    </button>
                                                </div>
                                            </form>
                                        </div>
                                    </div>
                                </div>
                            {:else if mode === "feedback"}
                                <div
                                    class="bg-white rounded-2xl border border-[#E8EAED] shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
                                    in:fade
                                >
                                    <div
                                        class="p-8 border-b border-[#F1F3F4] bg-[#F8F9FA]/30 grid grid-cols-1 md:grid-cols-3 gap-8"
                                    >
                                        <div
                                            class="bg-white p-4 rounded-xl border border-[#E8EAED] shadow-sm"
                                        >
                                            <p
                                                class="text-xs text-[#5F6368] font-bold uppercase tracking-wider mb-2"
                                            >
                                                Avg Satisfaction
                                            </p>
                                            <div
                                                class="text-3xl font-bold text-[#1E8E3E]"
                                            >
                                                {feedbacks.length > 0
                                                    ? (
                                                          feedbacks.reduce(
                                                              (acc, f) =>
                                                                  acc +
                                                                  parseInt(
                                                                      f.satisfaction,
                                                                  ),
                                                              0,
                                                          ) / feedbacks.length
                                                      ).toFixed(1)
                                                    : "N/A"}<span
                                                    class="text-base text-[#5F6368] font-normal"
                                                    >/5</span
                                                >
                                            </div>
                                        </div>
                                        <div
                                            class="bg-white p-4 rounded-xl border border-[#E8EAED] shadow-sm"
                                        >
                                            <p
                                                class="text-xs text-[#5F6368] font-bold uppercase tracking-wider mb-2"
                                            >
                                                Avg Difficulty
                                            </p>
                                            <div
                                                class="text-3xl font-bold text-[#F9AB00]"
                                            >
                                                {feedbacks.length > 0
                                                    ? (
                                                          feedbacks.reduce(
                                                              (acc, f) =>
                                                                  acc +
                                                                  parseInt(
                                                                      f.difficulty,
                                                                  ),
                                                              0,
                                                          ) / feedbacks.length
                                                      ).toFixed(1)
                                                    : "N/A"}<span
                                                    class="text-base text-[#5F6368] font-normal"
                                                    >/5</span
                                                >
                                            </div>
                                        </div>
                                        <div
                                            class="bg-white p-4 rounded-xl border border-[#E8EAED] shadow-sm"
                                        >
                                            <p
                                                class="text-xs text-[#5F6368] font-bold uppercase tracking-wider mb-2"
                                            >
                                                Total Responses
                                            </p>
                                            <div
                                                class="text-3xl font-bold text-[#4285F4]"
                                            >
                                                {feedbacks.length}
                                            </div>
                                        </div>
                                    </div>

                                    <div
                                        class="flex-1 p-8 overflow-y-auto space-y-4"
                                    >
                                        {#each feedbacks as f}
                                            <div
                                                class="p-6 bg-white border border-[#E8EAED] rounded-xl shadow-sm hover:shadow-md transition-shadow"
                                            >
                                                <div
                                                    class="flex justify-between items-start mb-4"
                                                >
                                                    <div class="flex gap-4">
                                                        <div
                                                            class="bg-[#E6F4EA] text-[#137333] px-3 py-1 rounded-full text-xs font-bold"
                                                        >
                                                            Satisfaction: {f.satisfaction}/5
                                                        </div>
                                                        <div
                                                            class="bg-[#FEF7E0] text-[#B06000] px-3 py-1 rounded-full text-xs font-bold"
                                                        >
                                                            Difficulty: {f.difficulty}/5
                                                        </div>
                                                    </div>
                                                    <span
                                                        class="text-xs text-[#5F6368]"
                                                    >
                                                        {f.created_at
                                                            ? new Date(
                                                                  f.created_at,
                                                              ).toLocaleString()
                                                            : ""}
                                                    </span>
                                                </div>
                                                {#if f.comment}
                                                    <p
                                                        class="text-[#3C4043] text-sm leading-relaxed bg-[#F8F9FA] p-4 rounded-lg"
                                                    >
                                                        "{f.comment}"
                                                    </p>
                                                {:else}
                                                    <p
                                                        class="text-[#9AA0A6] text-sm italic"
                                                    >
                                                        No comments provided
                                                    </p>
                                                {/if}
                                            </div>
                                        {:else}
                                            <div
                                                class="text-center py-20 text-[#5F6368]"
                                            >
                                                <MessageSquare
                                                    size={48}
                                                    class="mx-auto mb-4 opacity-20"
                                                />
                                                <p class="text-lg font-medium">
                                                    No feedback yet
                                                </p>
                                                <p class="text-sm opacity-70">
                                                    Wait for attendees to finish
                                                    the codelab.
                                                </p>
                                            </div>
                                        {/each}
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
                            onclick={addStep}
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
        padding: 24px;
        margin: 24px 0;
        overflow-x: auto;
    }
    :global(.markdown-body code:not(pre code)) {
        font-family: inherit;
        color: #c5221f;
        background-color: #fce8e6;
        padding: 2px 5px;
        border-radius: 4px;
        font-size: 0.9em;
    }
    :global(.markdown-body pre code) {
        font-family: "JetBrains Mono", "Google Sans Mono", monospace;
        background-color: transparent;
        padding: 0;
        color: inherit;
        font-size: 0.95rem;
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
