<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { browser } from "$app/environment";
    import {
        getCodelab,
        updateCodelab,
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
        getMaterials,
        addMaterial,
        deleteMaterial,
        uploadMaterial,
        isFirebaseMode,
        listenToWsReplacement,
        sendChatMessage,
        type Codelab,
        type Step,
        type Attendee,
        type HelpRequest,
        type ChatMessage,
        type Feedback,
        type Material,
    } from "$lib/api";
    import { streamGeminiResponseRobust } from "$lib/gemini";
    import { decrypt } from "$lib/crypto";
    // @ts-ignore
    import QRCode from "svelte-qrcode";
    import { adminMarked as marked } from "$lib/markdown";
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
        Github,
        Sparkles,
        Loader2,
        Columns2,
        Paperclip,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let id = page.params.id as string;
    let activeStepIndex = $state(0);

    // Initialize mode from URL or default to 'edit'
    let initialMode = page.url.searchParams.get("mode");
    let mode = $state<"edit" | "preview" | "live" | "feedback" | "materials">(
        initialMode === "preview" ||
            initialMode === "live" ||
            initialMode === "feedback" ||
            initialMode === "materials"
            ? (initialMode as any)
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
    let materials = $state<Material[]>([]);
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
    let fileInput = $state<HTMLInputElement>(); // File input ref

    // AI State
    let geminiApiKey = $state("");
    let showAiMenu = $state(false);
    let menuPos = $state({ x: 0, y: 0 });
    let selectedText = $state("");
    let aiInstruction = $state("");
    let newMaterial = $state({
        title: "",
        material_type: "link" as "link" | "file",
        link_url: "",
        file_path: "",
    });
    let materialFileInput = $state<HTMLInputElement>();
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

    let isSidebarOpen = $state(false);
    let isSplitView = $state(false);

    let editorEl = $state<HTMLTextAreaElement | null>(null);
    let previewEl = $state<HTMLDivElement | null>(null);
    let isScrollingEditor = false;
    let isScrollingPreview = false;

    function syncEditorScroll() {
        if (isScrollingPreview || !editorEl || !previewEl) return;
        isScrollingEditor = true;
        const maxEditor = editorEl.scrollHeight - editorEl.clientHeight;
        const maxPreview = previewEl.scrollHeight - previewEl.clientHeight;
        
        if (maxEditor > 0 && maxPreview > 0) {
            const percentage = editorEl.scrollTop / maxEditor;
            previewEl.scrollTop = percentage * maxPreview;
        }
        
        // Debounce to prevent feedback loop
        setTimeout(() => (isScrollingEditor = false), 50);
    }

    function syncPreviewScroll() {
        if (isScrollingEditor || !editorEl || !previewEl) return;
        isScrollingPreview = true;
        const maxEditor = editorEl.scrollHeight - editorEl.clientHeight;
        const maxPreview = previewEl.scrollHeight - previewEl.clientHeight;
        
        if (maxEditor > 0 && maxPreview > 0) {
            const percentage = previewEl.scrollTop / maxPreview;
            editorEl.scrollTop = percentage * maxEditor;
        }
        
        // Debounce to prevent feedback loop
        setTimeout(() => (isScrollingPreview = false), 50);
    }

    // Sync mode to URL and load data
    $effect(() => {
        if (!browser) return;
        const url = new URL(window.location.href);
        if (url.searchParams.get("mode") !== mode) {
            url.searchParams.set("mode", mode);
            window.history.replaceState({}, "", url);
        }

        if (mode === "feedback") {
            loadFeedback();
        } else if (mode === "materials") {
            loadMaterials();
        } else if (mode === "live") {
            refreshLiveData();
            scrollToBottom();
        }
    });

    $effect(() => {
        return () => {
            if (ws) ws.close();
        };
    });

    $effect(() => {
        if (!browser) return;
        document.addEventListener("selectionchange", handleSelectionChange);
        return () => {
            document.removeEventListener(
                "selectionchange",
                handleSelectionChange,
            );
        };
    });

    let wsCleanup: any;
    onMount(async () => {
        try {
            const data = await getCodelab(id);
            codelab = data[0];
            steps = data[1];

            // Initial fetch of live data
            await refreshLiveData();
            await loadChatHistory();
            wsCleanup = initWebSocket();

            // Load API Key
            const encryptedKey = localStorage.getItem("gemini_api_key");
            if (encryptedKey) {
                const decrypted = decrypt(encryptedKey);
                if (decrypted) geminiApiKey = decrypted;
            }
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    $effect(() => {
        return () => {
            if (wsCleanup && typeof wsCleanup === 'function') wsCleanup();
            if (ws) ws.close();
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
            const isTextArea = activeElement && activeElement.tagName === "TEXTAREA";
            
            if (isTextArea) {
                const start = activeElement.selectionStart;
                const end = activeElement.selectionEnd;

                if (start !== end) {
                    const text = activeElement.value.substring(start, end);
                    if (text.trim().length > 0) {
                        selectedText = text;
                        selectionRange = { start, end };
                        aiInstruction = ""; // Reset instruction

                        // Calculate position relative to viewport
                        // If mouseup is outside textarea, we still show the menu near the mouse
                        let x = e.clientX;
                        let y = e.clientY - 40;

                        // Ensure menu stays within viewport
                        const menuWidth = 288; // w-72 = 18rem = 288px
                        const menuHeight = 220; 
                        
                        if (x + menuWidth > window.innerWidth) {
                            x = window.innerWidth - menuWidth - 20;
                        }
                        if (x < 20) x = 20;
                        
                        if (y + menuHeight > window.innerHeight) {
                            y = window.innerHeight - menuHeight - 20;
                        }
                        if (y < 20) y = 20;

                        menuPos = { x, y };
                        showAiMenu = true;
                        return;
                    }
                }
            }
            
            // Hide if clicked elsewhere and not loading and not clicking inside AI menu
            if (!aiLoading && showAiMenu) {
                const target = e.target as HTMLElement;
                if (!target.closest('.ai-menu-container')) {
                    showAiMenu = false;
                }
            }
        }, 10);
    }


    async function improveWithAi() {
        if (!geminiApiKey) {
            alert($t("ai_generator.api_key_required"));
            return;
        }
        if (!selectedText || !selectionRange) return;

        aiLoading = true;
        showAiMenu = false; // Hide menu

        const originalMarkdown = steps[activeStepIndex].content_markdown;
        const { start, end } = selectionRange;

        let prompt = `Improve the following technical writing/markdown content. Make it clearer, correct grammar, and better formatted. Maintain the original meaning. Only return the improved content, no explanations.\n\nContent:\n${selectedText}`;
        if (aiInstruction.trim()) {
            prompt = `Improve the following technical writing/markdown content based on this instruction: "${aiInstruction}".\nMake it clearer, correct grammar, and better formatted. Maintain the original meaning where possible. Only return the improved content, no explanations.\n\nContent:\n${selectedText}`;
        }
        const systemPrompt = "You are a helpful technical editor.";

        try {
            let fullImprovedContent = "";

            const stream = streamGeminiResponseRobust(prompt, systemPrompt, {
                apiKey: geminiApiKey,
            });

            for await (const chunk of stream) {
                fullImprovedContent += chunk;
                // We don't update the text visually during streaming for "tada" effect
            }

            // All finished, apply "tada"
            const currentFullText = steps[activeStepIndex].content_markdown;
            steps[activeStepIndex].content_markdown =
                currentFullText.substring(0, start) +
                fullImprovedContent +
                currentFullText.substring(end);

            // Update selection to the new content
            const newEnd = start + fullImprovedContent.length;
            selectionRange = {
                start: start,
                end: newEnd,
            };

            if (editorEl) {
                setTimeout(() => {
                    editorEl.focus();
                    editorEl.setSelectionRange(start, newEnd);
                }, 50);
            }
        } catch (e: any) {
            console.error(e);

            // Restore original text if generation failed
            steps[activeStepIndex].content_markdown = originalMarkdown;

            // Provide specific error messages
            let errorMessage = "AI Improvement failed: ";

            if (
                e.message &&
                (e.message.includes("429") ||
                    e.message.includes("API Error: 429"))
            ) {
                errorMessage =
                    "⏱️ Rate limit exceeded.\n\nPlease wait a moment and try again.\nThe free tier has limited requests per minute.";
            } else if (e.message) {
                errorMessage += e.message;
            } else {
                errorMessage += "Unknown error occurred.";
            }

            alert(errorMessage);
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

    async function loadMaterials() {
        try {
            materials = await getMaterials(id);
        } catch (e) {
            console.error("Failed to load materials:", e);
        }
    }

    async function handleAddMaterial() {
        try {
            const material = await addMaterial(id, {
                title: newMaterial.title,
                material_type: newMaterial.material_type,
                link_url: newMaterial.material_type === "link" ? newMaterial.link_url : undefined,
                file_path: newMaterial.material_type === "file" ? newMaterial.file_path : undefined,
            });
            materials = [...materials, material];
            newMaterial = {
                title: "",
                material_type: "link",
                link_url: "",
                file_path: "",
            };
        } catch (e) {
            console.error("Failed to add material:", e);
            alert("Failed to add material");
        }
    }

    async function handleDeleteMaterial(materialId: string) {
        if (!confirm($t("editor.delete_material_confirm"))) return;
        try {
            await deleteMaterial(id, materialId);
            materials = materials.filter((m) => m.id !== materialId);
        } catch (e) {
            console.error("Failed to delete material:", e);
        }
    }

    async function handleMaterialFileSelect(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!input.files || input.files.length === 0) return;
        
        const file = input.files[0];
        try {
            const res = await uploadMaterial(file);
            newMaterial.file_path = res.url;
            // 만약 제목이 비어있다면 파일 이름으로 채워줍니다.
            if (!newMaterial.title) {
                newMaterial.title = res.original_name;
            }
        } catch (e) {
            console.error("Upload failed:", e);
            alert("File upload failed");
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
        if (isFirebaseMode()) {
            return listenToWsReplacement(id, (data) => {
                if (data.type === "chat") {
                    if (messages.find(m => m.text === data.message && m.sender === data.sender_name)) return;
                    messages = [
                        ...messages,
                        {
                            sender: data.sender_name,
                            text: data.message,
                            time: data.created_at?.toDate ? data.created_at.toDate().toLocaleTimeString() : new Date().toLocaleTimeString(),
                            self: false,
                            type: "chat",
                        },
                    ];
                } else if (data.type === "help_request") {
                    refreshLiveData();
                }
            });
        }

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
        if (!chatMessage.trim()) return;
        if (isFirebaseMode()) {
            sendChatMessage(id, {
                sender: "Facilitator",
                message: chatMessage.trim(),
                type: "chat",
            });
            chatMessage = "";
            return;
        }
        if (!ws) return;
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
        if (!dmTarget || !dmMessage.trim()) return;

        if (isFirebaseMode()) {
            sendChatMessage(id, {
                sender: "Facilitator",
                message: dmMessage.trim(),
                type: "dm",
                target_id: dmTarget.id,
            });
            // Also add to local messages for visibility
            messages = [
                ...messages,
                {
                    sender: `To: ${dmTarget.name}`,
                    text: dmMessage.trim(),
                    time: new Date().toLocaleTimeString(),
                    self: true,
                    type: "dm",
                },
            ];
            dmMessage = "";
            dmTarget = null;
            scrollToBottom();
            return;
        }

        if (!ws) return;
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
            alert($t("common.error"));
        }
    }

    function addStep() {
        const newStep: Step = {
            id: "",
            codelab_id: id,
            step_number: steps.length + 1,
            title: $t("editor.untitled_step"),
            content_markdown: `# ${$t("editor.untitled_step")}\n\n${$t("editor.start_writing")}`,
        };
        steps = [...steps, newStep];
        activeStepIndex = steps.length - 1;
    }

    function removeStep(index: number) {
        if (!confirm($t("dashboard.confirm_delete"))) return;
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
        if (isSaving || !codelab) return;
        isSaving = true;
        try {
            await Promise.all([
                saveSteps(
                    id,
                    steps.map((s) => ({
                        title: s.title,
                        content_markdown: s.content_markdown,
                    })),
                ),
                updateCodelab(id, {
                    title: codelab.title,
                    description: codelab.description,
                    author: codelab.author,
                    is_public: codelab.is_public
                })
            ]);
            saveSuccess = true;
            setTimeout(() => (saveSuccess = false), 3000);
        } catch (e) {
            alert("Save failed: " + e);
        } finally {
            isSaving = false;
        }
    }

    async function toggleVisibility() {
        if (!codelab || isSaving) return;

        const newStatus = !codelab.is_public;
        codelab.is_public = newStatus;

        try {
            await updateCodelab(id, {
                title: codelab.title,
                description: codelab.description,
                author: codelab.author,
                is_public: newStatus
            });
        } catch (e) {
            // Revert on failure
            codelab.is_public = !newStatus;
            alert("Failed to update visibility: " + e);
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
            const html = marked.parse(currentStep.content_markdown) as string;
            if (browser) {
                return DOMPurify.sanitize(html);
            }
            return html;
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

<div class="min-h-screen bg-[#F8F9FA] dark:bg-dark-bg flex flex-col font-sans text-[#3C4043] dark:text-dark-text transition-colors">
    <header
        class="bg-white dark:bg-dark-surface border-b border-[#E8EAED] dark:border-dark-border py-3 sm:py-4 px-4 sm:px-8 sticky top-0 z-40 shadow-sm"
    >
        <div class="max-w-screen-2xl mx-auto flex justify-between items-center gap-2 sm:gap-3">
            <div class="flex items-center gap-1 sm:gap-6 flex-1 min-w-0">
                <a
                    href="/admin"
                    class="text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text hover:bg-[#F1F3F4] dark:hover:bg-white/5 p-1.5 sm:p-2 rounded-full transition-all shrink-0"
                    aria-label="Back to dashboard"
                >
                    <ChevronLeft size={24} />
                </a>
                <div class="min-w-0 flex-1">
                    {#if loading}
                        <div
                            class="h-5 sm:h-6 w-32 md:w-48 bg-[#F1F3F4] dark:bg-white/5 animate-pulse rounded"
                        ></div>
                    {:else}
                        <h1
                            class="text-sm sm:text-lg md:text-xl font-bold text-[#202124] dark:text-dark-text flex items-center gap-2 truncate"
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
                            class="text-[10px] sm:text-xs text-[#5F6368] dark:text-dark-text-muted font-medium mt-0.5 hidden xs:block"
                        >
                            ID: {id.split('-')[0]}... &bull; {$t("editor.facilitator_mode")}
                        </p>
                    {/if}
                </div>
            </div>
            <div class="flex items-center gap-1 sm:gap-2 lg:gap-4 shrink-0">
                <div class="hidden md:flex items-center gap-2">
                    <a
                        href="https://github.com/JAICHANGPARK/open-codelabs"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                        title={$t("common.github_repo")}
                    >
                        <Github size={20} />
                    </a>
                    <a
                        href="https://jaichangpark.github.io/open-codelabs/"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                        title={$t("common.documentation")}
                    >
                        <FileText size={20} />
                    </a>
                    <div class="w-px h-6 bg-[#E8EAED] dark:bg-dark-border mx-1"></div>
                </div>
                <button
                    onclick={toggleVisibility}
                    class="relative inline-flex h-7 w-12 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-[#4285F4] focus:ring-offset-2 {codelab?.is_public ? 'bg-[#34A853]' : 'bg-gray-200 dark:bg-dark-border'}"
                    role="switch"
                    aria-checked={codelab?.is_public}
                    title={codelab?.is_public ? $t("common.public") : $t("common.private")}
                >
                    <span
                        class="pointer-events-none flex h-5 w-5 items-center justify-center rounded-full bg-white shadow-sm ring-0 transition-transform duration-200 {codelab?.is_public ? 'translate-x-6' : 'translate-x-1'}"
                    >
                        {#if codelab?.is_public}
                            <Eye size={12} class="text-[#34A853]" />
                        {:else}
                            <X size={12} class="text-[#EA4335]" />
                        {/if}
                    </span>
                </button>
                <div class="h-6 w-px bg-[#E8EAED] dark:bg-dark-border hidden sm:block"></div>
                <button
                    onclick={handleExport}
                    class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-full transition-all"
                    title={$t("editor.export_codelab")}
                >
                    <Download size={20} />
                </button>
                <div
                    class="flex bg-[#F1F3F4] dark:bg-white/5 p-1 rounded-full border border-[#E8EAED] dark:border-dark-border"
                >
                    <button
                        onclick={() => (mode = "edit")}
                        class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                        'edit'
                            ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                    >
                        <Edit3 size={14} />
                        <span class="hidden sm:inline">{$t("editor.edit")}</span>
                    </button>
                    <button
                        onclick={() => (mode = "preview")}
                        class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                        'preview'
                            ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                    >
                        <Eye size={14} />
                        <span class="hidden sm:inline">{$t("editor.preview")}</span>
                    </button>
                    <button
                        onclick={() => (mode = "live")}
                        class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                        'live'
                            ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                    >
                        <Users size={14} />
                        <span class="hidden sm:inline">{$t("editor.live_tab")}</span>
                    </button>
                    <button
                        onclick={() => (mode = "feedback")}
                        class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                        'feedback'
                            ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                    >
                        <MessageSquare size={14} />
                        <span class="hidden sm:inline">{$t("editor.feedback_tab")}</span>
                    </button>
                    <button
                        onclick={() => (mode = "materials")}
                        class="px-2 sm:px-4 py-1.5 rounded-full flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-sm font-bold transition-all {mode ===
                        'materials'
                            ? 'bg-white dark:bg-dark-surface shadow-sm text-[#4285F4]'
                            : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text'}"
                    >
                        <Paperclip size={14} />
                        <span class="hidden sm:inline">{$t("editor.materials_tab")}</span>
                    </button>
                </div>
                <button
                    onclick={handleSave}
                    disabled={isSaving || mode !== "edit"}
                    class="bg-[#4285F4] hover:bg-[#1A73E8] disabled:opacity-50 text-white p-2 sm:px-6 sm:py-2.5 rounded-full flex items-center gap-2 text-sm font-bold transition-all shadow-md active:scale-95 {saveSuccess
                        ? 'bg-[#1E8E3E]'
                        : ''}"
                >
                    {#if isSaving}
                        <Loader2 size={18} class="animate-spin" />
                    {:else if saveSuccess}
                        <CheckCircle2 size={18} />
                    {:else}
                        <Save size={18} />
                    {/if}
                    <span class="hidden sm:inline">{$t("common.save")}</span>
                </button>
            </div>
        </div>
    </header>

    {#if loading}
        <div class="flex-1 flex justify-center items-center">
            <div
                class="animate-spin rounded-full h-12 w-12 border-4 border-[#E8EAED] dark:border-dark-border border-t-[#4285F4] dark:border-t-[#4285F4]"
            ></div>
        </div>
    {:else}
        <main
            class="max-w-screen-2xl mx-auto w-full p-4 sm:p-8 flex-1 grid grid-cols-1 lg:grid-cols-12 gap-2 items-start relative"
        >
            <!-- Mobile Step Navigation Toggle -->
            {#if mode !== "live" && mode !== "feedback" && mode !== "materials"}
                <div
                    class="lg:hidden flex items-center justify-between bg-white dark:bg-dark-surface p-4 rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm mb-2"
                >
                    <span class="font-bold text-sm"
                        >{$t("editor.step_navigation")}</span
                    >
                    <button
                        onclick={() => (isSidebarOpen = !isSidebarOpen)}
                        class="p-2 hover:bg-[#F1F3F4] dark:hover:bg-white/5 rounded-lg transition-colors"
                    >
                        <List size={20} />
                    </button>
                </div>
            {/if}

            <!-- Sidebar Navigation -->
            {#if mode !== "live" && mode !== "feedback" && mode !== "materials"}
                <div
                    class="fixed inset-0 z-50 lg:z-30 lg:relative lg:inset-auto lg:col-span-4 lg:block transition-all duration-300 {isSidebarOpen
                        ? 'translate-x-0 opacity-100'
                        : '-translate-x-full opacity-0 lg:translate-x-0 lg:opacity-100 lg:sticky lg:top-28'}"
                >
                <!-- Overlay for mobile -->
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div 
                    class="absolute inset-0 bg-black/50 backdrop-blur-sm lg:hidden"
                    onclick={() => isSidebarOpen = false}
                ></div>

                <div
                    class="relative bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border overflow-hidden shadow-xl lg:shadow-sm w-4/5 max-w-sm h-[90vh] lg:h-auto m-4 lg:m-0 flex flex-col"
                >
                    <div
                        class="p-5 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA] dark:bg-white/5 flex justify-between items-center"
                    >
                        <span
                            class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-widest"
                            >{$t("editor.step_navigation")}</span
                        >
                        <div class="flex items-center gap-2">
                            <button
                                onclick={addStep}
                                class="text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 p-1.5 rounded-full transition-colors"
                                title={$t("editor.add_step")}
                            >
                                <Plus size={18} />
                            </button>
                            <button 
                                onclick={() => isSidebarOpen = false}
                                class="lg:hidden p-1.5 hover:bg-[#E8EAED] dark:hover:bg-white/5 rounded-full transition-colors"
                            >
                                <X size={18} />
                            </button>
                        </div>
                    </div>
                    <div class="flex-1 overflow-y-auto max-h-[50vh] lg:max-h-[60vh]">
                        {#each steps as step, i}
                            <div
                                role="listitem"
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
                                    onclick={() => {
                                        activeStepIndex = i;
                                        isSidebarOpen = false;
                                    }}
                                    class="w-full text-left px-5 py-4 hover:bg-[#F8F9FA] dark:hover:bg-white/5 transition-all flex items-start gap-4 border-l-4 cursor-pointer {activeStepIndex ===
                                    i
                                        ? 'border-[#4285F4] bg-[#E8F0FE]/30 dark:bg-[#4285F4]/10'
                                        : 'border-transparent'} {draggedStepIndex ===
                                    i
                                        ? 'opacity-50'
                                        : ''}"
                                >
                                    <span
                                        class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold shrink-0 {activeStepIndex ===
                                        i
                                            ? 'bg-[#4285F4] text-white'
                                            : 'bg-[#F1F3F4] dark:bg-white/10 text-[#5F6368] dark:text-dark-text-muted'}"
                                        >{i + 1}</span
                                    >
                                    <span
                                        class="text-sm font-bold {activeStepIndex ===
                                        i
                                            ? 'text-[#1967D2] dark:text-[#4285F4]'
                                            : 'text-[#5F6368] dark:text-dark-text-muted'} line-clamp-1 pt-0.5 pr-6"
                                        >{step.title}</span
                                    >
                                </button>
                                <button
                                    onclick={() => removeStep(i)}
                                    class="absolute right-3 top-1/2 -translate-y-1/2 p-1.5 text-[#BDC1C6] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg lg:opacity-0 lg:group-hover:opacity-100 transition-all"
                                    title={$t("editor.delete_step")}
                                >
                                    <Trash2 size={14} />
                                </button>
                            </div>
                        {/each}
                    </div>

                    <div
                        class="p-6 border-t border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/50 dark:bg-white/5 flex flex-col items-center"
                    >
                        <div
                            class="bg-white p-3 rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm mb-4"
                        >
                            <QRCode value={attendeeUrl} size={140} />
                        </div>
                        <p
                            class="text-[11px] text-[#5F6368] dark:text-dark-text-muted text-center uppercase tracking-widest font-bold mb-3"
                        >
                            {$t("editor.attendee_access")}
                        </p>

                        <div
                            class="w-full flex items-center gap-2 p-2 bg-white dark:bg-dark-bg border border-[#E8EAED] dark:border-dark-border rounded-xl shadow-sm overflow-hidden"
                        >
                            <input
                                type="text"
                                readonly
                                value={attendeeUrl}
                                class="flex-1 text-[10px] text-[#5F6368] dark:text-dark-text-muted font-mono outline-none bg-transparent overflow-hidden text-ellipsis"
                            />
                            <button
                                onclick={copyUrl}
                                class="p-2 hover:bg-[#F1F3F4] dark:hover:bg-white/5 rounded-lg transition-colors {copySuccess
                                    ? 'text-[#1E8E3E]'
                                    : 'text-[#4285F4]'}"
                                title={$t("editor.copy_url")}
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
            {/if}

            <!-- Content Area -->
            <div
                class={mode === "live" ||
                mode === "feedback" ||
                mode === "materials"
                    ? "lg:col-span-12 w-full min-w-0"
                    : "lg:col-span-9 w-full min-w-0"}
                in:fade
            >
                {#if steps.length > 0}
                    <div
                        class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm min-h-[70vh] flex flex-col transition-colors"
                    >
                        {#if mode === "edit" || mode === "preview"}
                            <div
                                class="p-6 sm:p-8 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/30 dark:bg-white/5 sticky top-[73px] z-20 backdrop-blur-md rounded-t-2xl"
                            >
                                <input
                                    type="text"
                                    bind:value={steps[activeStepIndex].title}
                                    readonly={mode === "preview"}
                                    class="text-2xl sm:text-3xl font-bold text-[#202124] dark:text-dark-text w-full bg-transparent outline-none placeholder-[#DADCE0] dark:placeholder-dark-text-muted border-b-2 border-transparent focus:border-[#4285F4] transition-all pb-2"
                                    placeholder={$t("editor.untitled_step")}
                                />
                            </div>
                        {/if}

                        <div class="flex-1 p-4 sm:p-8 flex flex-col">
                            {#if mode === "edit"}
                                <div
                                    class="flex flex-wrap items-center gap-1 sm:gap-2 mb-4 p-2 bg-[#F8F9FA]/90 dark:bg-white/5 backdrop-blur-sm rounded-xl border border-[#E8EAED] dark:border-dark-border sticky top-[166px] z-20"
                                >
                                    <button
                                        onclick={() => insertMarkdown("h1")}
                                        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
                                        title="Heading"
                                        ><Heading1 size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("bold")}
                                        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
                                        title="Bold"><Bold size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("italic")}
                                        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
                                        title="Italic"
                                        ><Italic size={20} /></button
                                    >
                                    <div
                                        class="w-px h-6 bg-[#DADCE0] dark:bg-dark-border mx-1"
                                    ></div>
                                    <button
                                        onclick={() => insertMarkdown("list")}
                                        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
                                        title="List"><List size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("code")}
                                        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
                                        title="Code Block"
                                        ><Code size={20} /></button
                                    >
                                    <button
                                        onclick={() => insertMarkdown("image")}
                                        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]"
                                        title="Image"
                                        ><ImageIcon size={20} /></button
                                    >
                                    <div
                                        class="w-px h-6 bg-[#DADCE0] dark:bg-dark-border mx-1"
                                    ></div>
                                    <button
                                        onclick={() => (isSplitView = !isSplitView)}
                                        class="p-2 hover:bg-white dark:hover:bg-white/10 rounded-lg transition-colors {isSplitView ? 'text-[#4285F4] bg-white dark:bg-white/10 shadow-sm' : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4]'}"
                                        title={$t("editor.split_view")}
                                        ><Columns2 size={20} /></button
                                    >
                                </div>
                                <input
                                    type="file"
                                    accept="image/*"
                                    class="hidden"
                                    bind:this={fileInput}
                                    onchange={handleFileSelect}
                                />
                                <div class="flex-1 flex flex-col min-h-[60vh] relative">
                                    <div class="flex-1 grid {isSplitView ? 'grid-cols-1 lg:grid-cols-2 lg:h-[75vh]' : 'grid-cols-1'} gap-8 relative">
                                        <textarea
                                            bind:this={editorEl}
                                            onscroll={syncEditorScroll}
                                            bind:value={
                                                steps[activeStepIndex].content_markdown
                                            }
                                            onkeydown={handleKeydown}
                                            onpaste={handlePaste}
                                            readonly={aiLoading}
                                            class="w-full h-full outline-none text-[#3C4043] dark:text-dark-text font-mono text-base leading-relaxed resize-none bg-transparent {isSplitView ? 'overflow-y-auto pr-2' : ''}"
                                            style={aiLoading ? "cursor: wait;" : ""}
                                            placeholder={$t("editor.start_writing")}
                                            onmouseup={handleMouseUp}
                                        ></textarea>

                                        {#if isSplitView}
                                            <div 
                                                bind:this={previewEl}
                                                onscroll={syncPreviewScroll}
                                                class="hidden lg:block border-l border-[#F1F3F4] dark:border-dark-border pl-8 overflow-y-auto"
                                            >
                                                <div class="prose dark:prose-invert prose-blue max-w-none markdown-body">
                                                    {@html renderedContent}
                                                </div>
                                            </div>
                                        {/if}
                                    </div>
                                </div>

                                {#if showAiMenu}
                                    <div
                                        class="fixed z-50 animate-in fade-in zoom-in-95 duration-200 ai-menu-container"
                                        style="top: {menuPos.y}px; left: {menuPos.x}px;"
                                    >
                                        <div class="bg-white dark:bg-dark-surface rounded-2xl shadow-2xl border border-[#D2E3FC] dark:border-[#4285F4]/30 p-4 w-72 flex flex-col gap-3">
                                             <div class="flex items-center gap-2 text-[#4285F4] mb-1">
                                                 <Sparkles size={18} />
                                                 <span class="font-bold text-sm">{$t("gemini.improve_with_gemini")}</span>
                                             </div>
                                             
                                             <div class="space-y-2">
                                                 <label for="ai-instruction" class="text-[10px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider">
                                                     {$t("gemini.improvement_instruction")}
                                                 </label>
                                                 <textarea
                                                     id="ai-instruction"
                                                     bind:value={aiInstruction}
                                                     placeholder={$t("gemini.improvement_placeholder")}
                                                     class="w-full h-20 p-2 text-xs bg-[#F8F9FA] dark:bg-white/5 border border-[#DADCE0] dark:border-dark-border rounded-lg outline-none focus:border-[#4285F4] dark:focus:border-[#4285F4] resize-none"
                                                     onkeydown={(e) => {
                                                         if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                                                             improveWithAi();
                                                         }
                                                     }}
                                                 ></textarea>
                                             </div>
                                
                                             <button
                                                 onclick={improveWithAi}
                                                 class="w-full bg-[#4285F4] hover:bg-[#1A73E8] text-white py-2 rounded-xl text-sm font-bold transition-all shadow-md flex items-center justify-center gap-2"
                                             >
                                                 <CheckCircle2 size={16} />
                                                 {$t("gemini.ai_improve_submit")}
                                             </button>
                                        </div>
                                    </div>
                                {/if}

                                {#if aiLoading}
                                    <div
                                        class="fixed z-50 top-4 right-4 bg-white dark:bg-dark-surface px-4 py-3 rounded-xl shadow-lg border border-[#E8EAED] dark:border-dark-border flex items-center gap-3 animate-in slide-in-from-right"
                                    >
                                        <Loader2
                                            class="animate-spin text-[#4285F4]"
                                            size={20}
                                        />
                                        <div>
                                            <p
                                                class="text-sm font-bold text-[#202124] dark:text-dark-text"
                                            >
                                                {$t("gemini.gemini_is_writing")}
                                            </p>
                                            <p class="text-xs text-[#5F6368] dark:text-dark-text-muted">
                                                {$t("gemini.improving_content")}
                                            </p>
                                        </div>
                                    </div>
                                {/if}
                            {:else if mode === "preview"}
                                <div
                                    class="prose dark:prose-invert prose-blue max-w-none flex-1 markdown-body"
                                    in:fade
                                >
                                    {@html renderedContent}
                                </div>
                            {:else if mode === "live"}
                                <div
                                    class="grid grid-cols-1 xl:grid-cols-2 gap-6 sm:gap-8 h-full"
                                    in:fade
                                >
                                    <!-- Left: Activity & Help -->
                                    <div class="space-y-6 flex flex-col h-full min-w-0">
                                        <div
                                            class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col"
                                        >
                                            <div
                                                class="p-4 bg-red-50 dark:bg-red-500/10 border-b border-red-100 dark:border-red-500/20 flex items-center gap-2"
                                            >
                                                <Bell
                                                    size={18}
                                                    class="text-[#EA4335]"
                                                />
                                                <h3
                                                    class="font-bold text-[#EA4335]"
                                                >
                                                    {$t("help.request")} ({helpRequests.length})
                                                </h3>
                                            </div>
                                            <div
                                                class="p-4 space-y-3 max-h-60 overflow-y-auto"
                                            >
                                                {#each helpRequests as hr}
                                                    <div
                                                        class="p-3 bg-red-50/50 dark:bg-red-500/5 rounded-xl border border-red-100 dark:border-red-500/10 flex justify-between items-center"
                                                        in:slide
                                                    >
                                                        <div>
                                                            <p
                                                                class="font-bold text-[#202124] dark:text-dark-text text-sm"
                                                            >
                                                                {hr.attendee_name}
                                                            </p>
                                                            <p
                                                                class="text-xs text-[#EA4335]"
                                                            >
                                                                {$t("editor.stuck_on_step", { values: { step: hr.step_number } })}
                                                            </p>
                                                        </div>
                                                        <button
                                                            onclick={() =>
                                                                handleResolveHelp(
                                                                    hr.id,
                                                                )}
                                                            class="text-xs font-bold text-white bg-[#EA4335] px-3 py-1.5 rounded-full hover:bg-[#D93025] transition-colors shadow-sm"
                                                            >{$t("editor.resolve")}</button
                                                        >
                                                    </div>
                                                {:else}
                                                    <p
                                                        class="text-center py-6 text-[#9AA0A6] dark:text-dark-text-muted text-sm"
                                                    >
                                                        No pending help requests
                                                    </p>
                                                {/each}
                                            </div>
                                        </div>

                                        <div
                                            class="flex-1 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col min-h-[300px] lg:min-h-[400px]"
                                        >
                                            <div
                                                class="p-4 bg-[#F8F9FA] dark:bg-white/5 border-b border-[#E8EAED] dark:border-dark-border flex items-center gap-2"
                                            >
                                                <Users
                                                    size={18}
                                                    class="text-[#4285F4]"
                                                />
                                                <h3
                                                    class="font-bold text-[#3C4043] dark:text-dark-text"
                                                >
                                                    {$t("common.attendee")} ({attendees.length})
                                                </h3>
                                            </div>
                                            <div
                                                class="p-4 space-y-2 overflow-y-auto"
                                            >
                                                {#each attendees as attendee}
                                                    <div
                                                        class="flex items-center justify-between p-2 hover:bg-[#F8F9FA] dark:hover:bg-white/5 rounded-lg transition-colors group"
                                                    >
                                                        <div
                                                            class="flex items-center gap-3"
                                                        >
                                                            <div
                                                                class="w-8 h-8 rounded-full bg-[#E8EAED] dark:bg-white/10 flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted text-xs font-bold uppercase"
                                                            >
                                                                {attendee.name.charAt(
                                                                    0,
                                                                )}
                                                            </div>
                                                            <div>
                                                                <p
                                                                    class="text-sm font-bold text-[#202124] dark:text-dark-text"
                                                                >
                                                                    {attendee.name}
                                                                </p>
                                                                <p
                                                                    class="text-[10px] text-[#9AA0A6] dark:text-dark-text-muted"
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
                                                            class="p-2 text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-lg opacity-0 lg:opacity-0 group-hover:opacity-100 transition-all"
                                                            title={$t("editor.send_dm")}
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
                                        class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col h-full min-h-[500px] lg:min-h-[600px]"
                                    >
                                        <div
                                            class="flex border-b border-[#E8EAED] dark:border-dark-border"
                                        >
                                            <button
                                                onclick={() =>
                                                    (chatTab = "public")}
                                                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab ===
                                                'public'
                                                    ? 'text-[#4285F4] border-b-2 border-[#4285F4] bg-[#F8F9FA] dark:bg-white/5'
                                                    : 'text-[#5F6368] dark:text-dark-text-muted hover:bg-[#F1F3F4] dark:hover:bg-white/5'}"
                                            >
                                                <Users size={16} /> Public Chat
                                            </button>
                                            <button
                                                onclick={() =>
                                                    (chatTab = "direct")}
                                                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab ===
                                                'direct'
                                                    ? 'text-[#4285F4] border-b-2 border-[#4285F4] bg-[#F8F9FA] dark:bg-white/5'
                                                    : 'text-[#5F6368] dark:text-dark-text-muted hover:bg-[#F1F3F4] dark:hover:bg-white/5'}"
                                            >
                                                <MessageSquare size={16} /> Direct
                                                Messages
                                            </button>
                                        </div>

                                        <div
                                            class="flex-1 p-4 space-y-4 overflow-y-auto bg-[#F8F9FA] dark:bg-dark-bg/50"
                                            id="chat-messages"
                                        >
                                            {#each filteredMessages as msg}
                                                <div
                                                    class="flex flex-col {msg.self
                                                        ? 'items-end'
                                                        : 'items-start'}"
                                                >
                                                    <span
                                                        class="text-[10px] text-[#5F6368] dark:text-dark-text-muted font-bold mb-1 mx-1 uppercase"
                                                        >{msg.sender} &bull; {msg.time}</span
                                                    >
                                                    <div
                                                        class="px-4 py-2 rounded-2xl text-sm max-w-[85%] whitespace-pre-wrap break-words {msg.self
                                                            ? 'bg-[#4285F4] text-white rounded-tr-none shadow-md'
                                                            : 'bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border text-[#3C4043] dark:text-dark-text shadow-sm rounded-tl-none'}"
                                                    >
                                                        {msg.text}
                                                    </div>
                                                </div>
                                            {/each}
                                            {#if filteredMessages.length === 0}
                                                <div
                                                    class="h-full flex flex-col items-center justify-center text-[#9AA0A6] dark:text-dark-text-muted"
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
                                            class="p-4 border-t border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface"
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
                                                        class="absolute inset-0 bg-white/80 dark:bg-dark-surface/80 z-10 flex items-center justify-center text-sm text-[#5F6368] dark:text-dark-text-muted font-bold"
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
                                                            class="bg-[#E8F0FE] dark:bg-[#4285F4]/20 text-[#1967D2] dark:text-[#4285F4] px-2 py-1 rounded text-xs font-bold whitespace-nowrap"
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
                                                            class="flex-1 pl-4 pr-12 py-3 bg-[#F8F9FA] dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-xl outline-none focus:border-[#4285F4] text-sm text-[#202124] dark:text-dark-text"
                                                        />
                                                    {:else}
                                                        <input
                                                            type="text"
                                                            bind:value={
                                                                dmMessage
                                                            }
                                                            placeholder="Type a message..."
                                                            class="flex-1 pl-4 pr-12 py-3 bg-[#F8F9FA] dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-xl outline-none focus:border-[#4285F4] text-sm text-[#202124] dark:text-dark-text"
                                                        />
                                                    {/if}
                                                    <button
                                                        type="submit"
                                                        class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-lg transition-all"
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
                                    class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden min-h-[70vh] flex flex-col"
                                    in:fade
                                >
                                    <div
                                        class="p-6 sm:p-8 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/30 dark:bg-white/5 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 sm:gap-8"
                                    >
                                        <div
                                            class="bg-white dark:bg-dark-surface p-4 rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm"
                                        >
                                            <p
                                                class="text-xs text-[#5F6368] dark:text-dark-text-muted font-bold uppercase tracking-wider mb-2"
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
                                                    class="text-base text-[#5F6368] dark:text-dark-text-muted font-normal"
                                                    >/5</span
                                                >
                                            </div>
                                        </div>
                                        <div
                                            class="bg-white dark:bg-dark-surface p-4 rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm"
                                        >
                                            <p
                                                class="text-xs text-[#5F6368] dark:text-dark-text-muted font-bold uppercase tracking-wider mb-2"
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
                                                    class="text-base text-[#5F6368] dark:text-dark-text-muted font-normal"
                                                    >/5</span
                                                >
                                            </div>
                                        </div>
                                        <div
                                            class="bg-white dark:bg-dark-surface p-4 rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm sm:col-span-2 md:col-span-1"
                                        >
                                            <p
                                                class="text-xs text-[#5F6368] dark:text-dark-text-muted font-bold uppercase tracking-wider mb-2"
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
                                        class="flex-1 p-4 sm:p-8 overflow-y-auto space-y-4"
                                    >
                                        {#each feedbacks as f}
                                            <div
                                                class="p-6 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl shadow-sm hover:shadow-md transition-shadow"
                                            >
                                                <div
                                                    class="flex flex-col sm:flex-row justify-between items-start gap-4 mb-4"
                                                >
                                                    <div class="flex flex-wrap gap-3">
                                                        <div
                                                            class="bg-[#E6F4EA] dark:bg-green-500/10 text-[#137333] dark:text-green-400 px-3 py-1 rounded-full text-xs font-bold"
                                                        >
                                                            Satisfaction: {f.satisfaction}/5
                                                        </div>
                                                        <div
                                                            class="bg-[#FEF7E0] dark:bg-yellow-500/10 text-[#B06000] dark:text-yellow-400 px-3 py-1 rounded-full text-xs font-bold"
                                                        >
                                                            Difficulty: {f.difficulty}/5
                                                        </div>
                                                    </div>
                                                    <span
                                                        class="text-xs text-[#5F6368] dark:text-dark-text-muted"
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
                                                        class="text-[#3C4043] dark:text-dark-text text-sm leading-relaxed bg-[#F8F9FA] dark:bg-white/5 p-4 rounded-lg border border-transparent dark:border-dark-border"
                                                    >
                                                        "{f.comment}"
                                                    </p>
                                                {:else}
                                                    <p
                                                        class="text-[#9AA0A6] dark:text-dark-text-muted text-sm italic"
                                                    >
                                                        No comments provided
                                                    </p>
                                                {/if}
                                            </div>
                                        {:else}
                                            <div
                                                class="text-center py-20 text-[#5F6368] dark:text-dark-text-muted"
                                            >
                                                <MessageSquare
                                                    size={48}
                                                    class="mx-auto mb-4 opacity-20"
                                                />
                                                <p class="text-lg font-medium dark:text-dark-text">
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

                            {#if mode === "materials"}
                                <div
                                    class="flex-1 flex flex-col p-6 sm:p-8 space-y-8 overflow-y-auto max-h-[75vh]"
                                >
                                    <div
                                        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4"
                                    >
                                        <div>
                                            <h2
                                                class="text-2xl font-bold text-[#202124] dark:text-dark-text mb-1"
                                            >
                                                {$t("editor.materials_title")}
                                            </h2>
                                        </div>
                                    </div>

                                    <!-- Material Form -->
                                    <div
                                        class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-2xl border border-[#E8EAED] dark:border-dark-border space-y-4 shadow-sm"
                                    >
                                        <div
                                            class="grid grid-cols-1 md:grid-cols-2 gap-4"
                                        >
                                            <div class="space-y-2">
                                                <label
                                                    for="mat-name"
                                                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                                                    >{$t(
                                                        "editor.material_name",
                                                    )}</label
                                                >
                                                <input
                                                    id="mat-name"
                                                    type="text"
                                                    bind:value={newMaterial.title}
                                                    placeholder={$t(
                                                        "editor.material_placeholder_name",
                                                    )}
                                                    class="w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-3 outline-none focus:ring-2 focus:ring-[#4285F4] transition-all dark:text-dark-text shadow-sm"
                                                />
                                            </div>
                                            <div class="space-y-2">
                                                <label
                                                    for="mat-type"
                                                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                                                    >{$t(
                                                        "editor.material_type",
                                                    )}</label
                                                >
                                                <select
                                                    id="mat-type"
                                                    bind:value={newMaterial.material_type}
                                                    class="w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-3 outline-none focus:ring-2 focus:ring-[#4285F4] transition-all dark:text-dark-text shadow-sm"
                                                >
                                                    <option value="link"
                                                        >Link</option
                                                    >
                                                    <option value="file"
                                                        >File</option
                                                    >
                                                </select>
                                            </div>
                                        </div>

                                        {#if newMaterial.material_type === "link"}
                                            <div class="space-y-2">
                                                <label
                                                    for="mat-link"
                                                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                                                    >{$t(
                                                        "editor.material_link",
                                                    )}</label
                                                >
                                                <input
                                                    id="mat-link"
                                                    type="text"
                                                    bind:value={newMaterial.link_url}
                                                    placeholder={$t(
                                                        "editor.material_placeholder_link",
                                                    )}
                                                    class="w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-3 outline-none focus:ring-2 focus:ring-[#4285F4] transition-all dark:text-dark-text shadow-sm"
                                                />
                                            </div>
                                        {:else}
                                            <div class="space-y-2">
                                                <label
                                                    for="mat-file-upload"
                                                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                                                    >{$t(
                                                        "editor.material_file",
                                                    )}</label
                                                >
                                                <div
                                                    class="flex items-center gap-4"
                                                >
                                                    <button
                                                        id="mat-file-upload"
                                                        onclick={() =>
                                                            materialFileInput?.click()}
                                                        class="flex items-center gap-2 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border hover:bg-[#F1F3F4] dark:hover:bg-white/10 px-4 py-2.5 rounded-xl transition-all shadow-sm text-sm"
                                                    >
                                                        <Plus size={18} />
                                                        <span
                                                            >{$t(
                                                                "editor.upload_file",
                                                            )}</span
                                                        >
                                                    </button>
                                                    {#if newMaterial.file_path}
                                                        <span
                                                            class="text-sm text-[#1E8E3E] flex items-center gap-1"
                                                            ><CheckCircle2
                                                                size={14}
                                                            /> Uploaded</span
                                                        >
                                                    {/if}
                                                </div>
                                                <input
                                                    type="file"
                                                    bind:this={materialFileInput}
                                                    onchange={handleMaterialFileSelect}
                                                    class="hidden"
                                                />
                                            </div>
                                        {/if}

                                        <div class="flex justify-end pt-2">
                                            <button
                                                onclick={handleAddMaterial}
                                                disabled={!newMaterial.title ||
                                                    (newMaterial.material_type ===
                                                    "link"
                                                        ? !newMaterial.link_url
                                                        : !newMaterial.file_path)}
                                                class="bg-[#4285F4] hover:bg-[#1A73E8] disabled:opacity-50 text-white px-8 py-3 rounded-xl font-bold transition-all shadow-md active:scale-95 flex items-center gap-2"
                                            >
                                                <Plus size={18} />
                                                {$t("editor.add_material")}
                                            </button>
                                        </div>
                                    </div>

                                    <!-- Material List -->
                                    <div class="space-y-4">
                                        {#if materials.length > 0}
                                            <div
                                                class="grid grid-cols-1 md:grid-cols-2 gap-4"
                                            >
                                                {#each materials as mat}
                                                    <div
                                                        class="flex items-center justify-between p-4 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-sm hover:shadow-md transition-all group"
                                                    >
                                                        <div
                                                            class="flex items-center gap-3 min-w-0"
                                                        >
                                                            <div
                                                                class="p-2.5 bg-[#F1F3F4] dark:bg-white/10 rounded-xl text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4] transition-colors shrink-0"
                                                            >
                                                                {#if mat.material_type === "link"}
                                                                    <ExternalLink
                                                                        size={20}
                                                                    />
                                                                {:else}
                                                                    <Download
                                                                        size={20}
                                                                    />
                                                                {/if}
                                                            </div>
                                                            <div class="min-w-0">
                                                                <h4
                                                                    class="font-bold text-[#202124] dark:text-dark-text truncate"
                                                                >
                                                                    {mat.title}
                                                                </h4>
                                                                <p
                                                                    class="text-xs text-[#5F6368] dark:text-dark-text-muted truncate"
                                                                >
                                                                    {mat.material_type ===
                                                                    "link"
                                                                        ? mat.link_url
                                                                        : mat.file_path
                                                                              ?.split("/")
                                                                              .pop()}
                                                                </p>
                                                            </div>
                                                        </div>
                                                        <button
                                                            onclick={() =>
                                                                handleDeleteMaterial(
                                                                    mat.id,
                                                                )}
                                                            class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#EA4335] hover:bg-[#FCE8E6] dark:hover:bg-[#EA4335]/10 rounded-lg transition-all opacity-0 group-hover:opacity-100 shrink-0"
                                                        >
                                                            <Trash2 size={18} />
                                                        </button>
                                                    </div>
                                                {/each}
                                            </div>
                                        {:else}
                                            <div
                                                class="text-center py-12 border-2 border-dashed border-[#DADCE0] dark:border-dark-border rounded-3xl opacity-50"
                                            >
                                                <Paperclip
                                                    size={40}
                                                    class="mx-auto mb-3 text-[#DADCE0] dark:text-dark-border"
                                                />
                                                <p>
                                                    {$t("editor.no_materials")}
                                                </p>
                                            </div>
                                        {/if}
                                    </div>
                                </div>
                            {/if}
                        </div>
                    </div>
                {:else}
                    <div
                        class="bg-white dark:bg-dark-surface rounded-3xl border-2 border-dashed border-[#DADCE0] dark:border-dark-border p-12 sm:p-24 text-center shadow-sm"
                        in:fly={{ y: 20 }}
                    >
                        <div
                            class="w-20 h-20 bg-[#F1F3F4] dark:bg-white/5 rounded-full flex items-center justify-center mx-auto mb-8"
                        >
                            <Plus size={40} class="text-[#BDC1C6] dark:text-dark-text-muted" />
                        </div>
                        <h3 class="text-2xl font-bold text-[#202124] dark:text-dark-text mb-3">
                            {$t("editor.empty_codelab")}
                        </h3>
                        <p
                            class="text-[#5F6368] dark:text-dark-text-muted text-lg mb-10 max-w-sm mx-auto"
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
    /* Syntax Highlighting - Handled globally in app.css */
    :global(.markdown-body code:not(pre code)) {
        font-family: inherit;
        color: #c5221f;
        background-color: #fce8e6;
        padding: 2px 5px;
        border-radius: 4px;
        font-size: 0.9em;
    }
    :global(html.dark .markdown-body code:not(pre code)) {
        color: #ff8077;
        background-color: rgba(234, 67, 53, 0.15);
    }
    :global(.markdown-body pre code) {
        font-family: "JetBrains Mono", "Google Sans Mono", monospace;
        background-color: transparent;
        padding: 0;
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
    :global(html.dark .markdown-body h2) {
        color: #e8eaed;
        border-bottom-color: #3c4043;
    }
</style>
