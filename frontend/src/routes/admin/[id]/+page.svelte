<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import {
        getCodelab,
        saveSteps,
        exportCodelab,
        getAttendees,
        getHelpRequests,
        resolveHelpRequest,
        getWsUrl,
        getChatHistory,
        uploadImage,
        type Codelab,
        type Step,
        type Attendee,
        type HelpRequest,
        type ChatMessage,
    } from "$lib/api";
    // @ts-ignore
    import QRCode from "svelte-qrcode";
    import { marked } from "marked";
    import { markedHighlight } from "marked-highlight";
    import hljs from "highlight.js";
    import "highlight.js/styles/github.css";
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
        Users,
        Bell,
        MessageSquare,
        Send,
        Copy,
        Check,
        X,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let id = page.params.id as string;
    let codelab = $state<Codelab | null>(null);
    let steps = $state<Step[]>([]);
    let loading = $state(true);
    let activeStepIndex = $state(0);
    let mode = $state<"edit" | "preview" | "live">("edit");
    let isSaving = $state(false);
    let saveSuccess = $state(false);
    let copySuccess = $state(false);

    let attendees = $state<Attendee[]>([]);
    let helpRequests = $state<HelpRequest[]>([]);
    let ws = $state<WebSocket | null>(null);
    let chatMessage = $state("");
    let messages = $state<
        {
            sender: string;
            text: string;
            time: string;
            self?: boolean;
        }[]
    >([]);
    let dmTarget = $state<Attendee | null>(null);
    let dmMessage = $state("");

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
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

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
                        };
                    } else {
                        return {
                            sender: `[DM] ${msg.sender_name}`,
                            text: msg.message,
                            time: timeStr,
                            self: false,
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
                        },
                    ];
                } else if (data.type === "dm") {
                    messages = [
                        ...messages,
                        {
                            sender: `[DM] ${data.sender}`,
                            text: data.message,
                            time: data.timestamp,
                            self: false,
                        },
                    ];
                } else if (
                    data.type === "help_request" ||
                    data.type === "help_resolved"
                ) {
                    refreshLiveData();
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
            },
        ];

        dmMessage = "";
        dmTarget = null;
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

    async function handlePaste(event: ClipboardEvent) {
        const items = event.clipboardData?.items;
        if (!items) return;

        for (const item of items) {
            if (item.type.indexOf("image") !== -1) {
                const file = item.getAsFile();
                if (file) {
                    event.preventDefault();
                    try {
                        const { url } = await uploadImage(file);

                        const textarea = document.querySelector("textarea");
                        if (!textarea) return;

                        const start = textarea.selectionStart;
                        const end = textarea.selectionEnd;
                        const text = steps[activeStepIndex].content_markdown;
                        const replacement = `![image](${url})`;

                        steps[activeStepIndex].content_markdown =
                            text.substring(0, start) +
                            replacement +
                            text.substring(end);

                        setTimeout(() => {
                            textarea.focus();
                            const newCursorPos = start + replacement.length;
                            textarea.setSelectionRange(
                                newCursorPos,
                                newCursorPos,
                            );
                        }, 0);
                    } catch (e) {
                        console.error("Upload failed", e);
                        alert("Image upload failed");
                    }
                }
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
                    <button
                        on:click={() => (mode = "live")}
                        class="px-5 py-1.5 rounded-full flex items-center gap-2 text-sm font-bold transition-all {mode ===
                        'live'
                            ? 'bg-white shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] hover:text-[#202124]'}"
                    >
                        <Users size={16} /> Live Status
                        {#if helpRequests.length > 0}
                            <span class="w-2 h-2 bg-[#EA4335] rounded-full"
                            ></span>
                        {/if}
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
                            <div class="group relative">
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
                                            : 'text-[#5F6368]'} line-clamp-1 pt-0.5 pr-6"
                                        >{step.title}</span
                                    >
                                </button>
                                <button
                                    on:click={() => removeStep(i)}
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
                                on:click={copyUrl}
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
                                                            on:click={() =>
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
                                            class="flex-1 bg-white border border-[#E8EAED] rounded-2xl overflow-hidden shadow-sm flex flex-col"
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
                                                                </p>
                                                            </div>
                                                        </div>
                                                        <button
                                                            on:click={() =>
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
                                        class="bg-white border border-[#E8EAED] rounded-2xl overflow-hidden shadow-sm flex flex-col h-full"
                                    >
                                        <div
                                            class="p-4 bg-[#4285F4] text-white flex items-center gap-2"
                                        >
                                            <MessageSquare size={18} />
                                            <h3 class="font-bold">
                                                Chat Broadcast
                                            </h3>
                                        </div>
                                        <div
                                            class="flex-1 p-4 space-y-4 overflow-y-auto bg-[#F8F9FA]"
                                        >
                                            {#each messages as msg}
                                                <div class="flex flex-col">
                                                    <span
                                                        class="text-[10px] text-[#5F6368] font-bold mb-1 ml-1 uppercase"
                                                        >{msg.sender} &bull; {msg.time}</span
                                                    >
                                                    <div
                                                        class="px-4 py-2 rounded-2xl text-sm {msg.sender ===
                                                        'Facilitator'
                                                            ? 'bg-[#4285F4] text-white'
                                                            : 'bg-white border border-[#E8EAED] text-[#3C4043] shadow-sm'}"
                                                    >
                                                        {msg.text}
                                                    </div>
                                                </div>
                                            {/each}
                                        </div>
                                        <div
                                            class="p-4 border-t border-[#E8EAED]"
                                        >
                                            <form
                                                on:submit|preventDefault={sendBroadcast}
                                                class="relative"
                                            >
                                                <input
                                                    type="text"
                                                    bind:value={chatMessage}
                                                    placeholder={dmTarget
                                                        ? `Message to ${dmTarget.name}...`
                                                        : "Broadcast to all attendees..."}
                                                    class="w-full pl-4 pr-24 py-3 bg-[#F8F9FA] border border-[#DADCE0] rounded-xl outline-none focus:border-[#4285F4] text-sm"
                                                />
                                                <div
                                                    class="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1"
                                                >
                                                    {#if dmTarget}
                                                        <button
                                                            type="button"
                                                            on:click={sendDM}
                                                            class="p-2 bg-[#4285F4] text-white rounded-lg hover:bg-[#1A73E8] transition-all"
                                                            title="Send DM"
                                                        >
                                                            <Send size={18} />
                                                        </button>
                                                        <button
                                                            type="button"
                                                            on:click={() =>
                                                                (dmTarget =
                                                                    null)}
                                                            class="p-2 text-[#5F6368] hover:bg-[#E8EAED] rounded-lg"
                                                            title="Cancel DM"
                                                        >
                                                            <X size={18} />
                                                        </button>
                                                    {:else}
                                                        <button
                                                            type="submit"
                                                            class="p-2 text-[#4285F4] hover:bg-[#E8F0FE] rounded-lg transition-all"
                                                            title="Broadcast"
                                                        >
                                                            <Send size={18} />
                                                        </button>
                                                    {/if}
                                                </div>
                                            </form>
                                        </div>
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
