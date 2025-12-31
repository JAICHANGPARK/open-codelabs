<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { page } from "$app/state";
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
        type Quiz,
    } from "$lib/api";
    import { streamGeminiResponseRobust } from "$lib/gemini";
    import { adminMarked as marked } from "$lib/markdown";
    import DOMPurify from "dompurify";
    import { decrypt } from "$lib/crypto";
    import { 
        getQuizzes, 
        updateQuizzes, 
        getQuizSubmissions,
    } from "$lib/api";
    // ... icons imports ...
    import {
        Plus,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";

    // Components
    import AdminHeader from "$lib/components/admin/AdminHeader.svelte";
    import AdminSidebar from "$lib/components/admin/AdminSidebar.svelte";
    import EditMode from "$lib/components/admin/EditMode.svelte";
    import PreviewMode from "$lib/components/admin/PreviewMode.svelte";
    import LiveMode from "$lib/components/admin/LiveMode.svelte";
    import FeedbackMode from "$lib/components/admin/FeedbackMode.svelte";
    import MaterialsMode from "$lib/components/admin/MaterialsMode.svelte";
    import QuizMode from "$lib/components/admin/QuizMode.svelte";
    import SettingsMode from "$lib/components/admin/SettingsMode.svelte";
    import GuideMode from "$lib/components/admin/GuideMode.svelte";
    import SubmissionsMode from "$lib/components/admin/SubmissionsMode.svelte";

    import { 
        getSubmissions,
        deleteSubmission as apiDeleteSubmission
    } from "$lib/api";

    let id = page.params.id as string;
    let activeStepIndex = $state(0);

    // Initialize mode from URL or default to 'edit'
    let initialMode = page.url.searchParams.get("mode");
    let mode = $state<"edit" | "preview" | "guide" | "live" | "feedback" | "materials" | "quiz" | "submissions" | "settings">(
        initialMode === "preview" ||
            initialMode === "guide" ||
            initialMode === "live" ||
            initialMode === "feedback" ||
            initialMode === "materials" ||
            initialMode === "quiz" ||
            initialMode === "submissions" ||
            initialMode === "settings"
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
    let submissions = $state<any[]>([]);
    let quizzes = $state<Quiz[]>([]);
    let quizSubmissions = $state<any[]>([]);
    let isQuizGenerating = $state(false);
    let isGuideGenerating = $state(false);
    let numQuizToGenerate = $state(5);
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
        } else if (mode === "submissions") {
            loadSubmissions();
        } else if (mode === "quiz") {
            loadQuizzes();
            loadQuizSubmissions();
        } else if (mode === "live") {
            refreshLiveData();
            scrollToBottom();
        }
    });

    async function loadSubmissions() {
        try {
            submissions = await getSubmissions(id);
        } catch (e) {
            console.error("Failed to load submissions:", e);
        }
    }

    async function handleDeleteSubmission(attendeeId: string, submissionId: string) {
        if (!confirm($t("common.confirm_delete"))) return;
        try {
            await apiDeleteSubmission(id, attendeeId, submissionId);
            submissions = submissions.filter(s => s.id !== submissionId);
        } catch (e) {
            console.error("Failed to delete submission:", e);
        }
    }

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
                    if (editorEl) {
                        editorEl.focus();
                        editorEl.setSelectionRange(start, newEnd);
                    }
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

    async function loadQuizzes() {
        try {
            const rawQuizzes = await getQuizzes(id);
            quizzes = rawQuizzes.map(q => ({
                ...q,
                options: JSON.parse(q.options)
            }));
        } catch (e) {
            console.error("Failed to load quizzes:", e);
        }
    }

    async function loadQuizSubmissions() {
        try {
            quizSubmissions = await getQuizSubmissions(id);
        } catch (e) {
            console.error("Failed to load quiz submissions:", e);
        }
    }

    function addEmptyQuiz() {
        quizzes = [...quizzes, {
            id: "",
            codelab_id: id,
            question: "",
            quiz_type: "multiple_choice",
            options: ["", "", "", ""],
            correct_answer: 0
        } as any];
    }

    function removeQuiz(index: number) {
        quizzes = quizzes.filter((_, i) => i !== index);
    }

    async function handleQuizSave() {
        if (!codelab) return;
        isSaving = true;
        try {
            await updateQuizzes(id, quizzes.map(q => ({
                question: q.question,
                quiz_type: q.quiz_type || 'multiple_choice',
                options: Array.isArray(q.options) ? q.options : JSON.parse(q.options as any),
                correct_answer: q.correct_answer
            })));
            saveSuccess = true;
            setTimeout(() => (saveSuccess = false), 3000);
        } catch (e) {
            console.error("Failed to save quizzes:", e);
            alert("Failed to save quizzes");
        } finally {
            isSaving = false;
        }
    }

    async function generateQuizWithAi() {
        if (!geminiApiKey) {
            alert($t("ai_generator.api_key_required"));
            return;
        }
        isQuizGenerating = true;
        
        try {
            const context = steps.map(s => `Step ${s.step_number}: ${s.title}\n${s.content_markdown}`).join("\n\n");
            const prompt = `Based on the following codelab content, generate ${numQuizToGenerate} multiple-choice questions. 
            Each question must have exactly 5 options. 
            Return ONLY a valid JSON array of objects with this structure:
            [{"question": "string", "options": ["string", "string", "string", "string", "string"], "correct_answer": number (0-4)}]
            
            Codelab Content:
            ${context}`;

            const stream = streamGeminiResponseRobust(prompt, "You are a helpful education assistant that generates quizzes.", {
                apiKey: geminiApiKey
            });

            let responseText = "";
            for await (const chunk of stream) {
                responseText += chunk;
            }

            // Extract JSON from response (sometimes Gemini adds markdown code blocks)
            const jsonMatch = responseText.match(/\[.*\]/s);
            if (jsonMatch) {
                const newQuizzes = JSON.parse(jsonMatch[0]);
                quizzes = [...quizzes, ...newQuizzes.map((q: any) => ({
                    ...q,
                    id: "",
                    codelab_id: id
                }))];
            } else {
                throw new Error("Failed to parse AI response as JSON");
            }
        } catch (e) {
            console.error("Quiz generation failed:", e);
            alert("Quiz generation failed: " + e);
        } finally {
            isQuizGenerating = false;
        }
    }

    async function generateGuideWithAi() {
        if (!geminiApiKey || !codelab) {
            alert($t("ai_generator.api_key_required"));
            return;
        }
        isGuideGenerating = true;
        
        try {
            // Detect user language
            const userLanguage = $locale || "en";
            const languageNames: Record<string, string> = {
                ko: "Korean",
                en: "English",
                zh: "Chinese",
                ja: "Japanese",
            };
            const targetLanguage = languageNames[userLanguage] || "English";

            const context = steps.map(s => `Step ${s.step_number}: ${s.title}\n${s.content_markdown}`).join("\n\n");
            const prompt = `Based on the following codelab content, create a comprehensive "Preparation & Setup Guide" for attendees. 
            Include:
            1. System requirements (Prerequisites).
            2. Required software/tools to install.
            3. Environment setup instructions.
            4. Initial project boilerplate setup if necessary.
            
            Write ALL content in ${targetLanguage}.
            Write it in professional markdown. 
            
            Codelab Title: ${codelab.title}
            Description: ${codelab.description}
            
            Codelab Content:
            ${context}`;

            const stream = streamGeminiResponseRobust(prompt, `You are a professional developer advocate writing a preparation guide for a workshop. You MUST write everything in ${targetLanguage}.`, {
                apiKey: geminiApiKey
            });

            let responseText = "";
            codelab.guide_markdown = "";
            for await (const chunk of stream) {
                responseText += chunk;
                codelab.guide_markdown = responseText;
            }
        } catch (e) {
            console.error("Guide generation failed:", e);
            alert("Guide generation failed: " + e);
        } finally {
            isGuideGenerating = false;
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

    async function handleUniversalSave() {
        if (mode === "quiz") {
            await handleQuizSave();
        } else {
            // handleSave handles both "edit" and "settings" modes
            await handleSave();
        }
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
                    is_public: codelab.is_public,
                    require_quiz: codelab.require_quiz,
                    require_feedback: codelab.require_feedback,
                    guide_markdown: codelab.guide_markdown
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
                is_public: newStatus,
                require_quiz: codelab.require_quiz,
                require_feedback: codelab.require_feedback,
                guide_markdown: codelab.guide_markdown
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
            handleUniversalSave();
        }
    }}
/>

<div class="min-h-screen bg-[#F8F9FA] dark:bg-dark-bg flex flex-col font-sans text-[#3C4043] dark:text-dark-text transition-colors">
    <AdminHeader
        {id}
        {codelab}
        {loading}
        bind:mode
        {isSaving}
        {saveSuccess}
        {toggleVisibility}
        {handleExport}
        handleSave={handleUniversalSave}
    />

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
            <!-- Sidebar Navigation -->
            {#if mode !== "live" && mode !== "feedback" && mode !== "materials" && mode !== "quiz" && mode !== "settings" && mode !== "guide"}
                <AdminSidebar
                    bind:steps
                    bind:activeStepIndex
                    bind:isSidebarOpen
                    {attendeeUrl}
                    bind:copySuccess
                    {addStep}
                    {removeStep}
                    handleCopyUrl={copyUrl}
                />
            {/if}

            <!-- Content Area -->
            <div
                class={mode === "live" ||
                mode === "feedback" ||
                mode === "materials" ||
                mode === "quiz" ||
                mode === "settings" ||
                mode === "guide"
                    ? "lg:col-span-12 w-full min-w-0"
                    : "lg:col-span-8 w-full min-w-0"}
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
                                <EditMode
                                    bind:step={steps[activeStepIndex]}
                                    bind:isSplitView
                                    {aiLoading}
                                    bind:editorEl
                                    bind:previewEl
                                    bind:fileInput
                                    bind:aiInstruction
                                    bind:showAiMenu
                                    {menuPos}
                                    {selectedText}
                                    {renderedContent}
                                    {geminiApiKey}
                                    {insertMarkdown}
                                    {handleFileSelect}
                                    {handleKeydown}
                                    {handlePaste}
                                    {handleMouseUp}
                                    {improveWithAi}
                                    {syncEditorScroll}
                                    {syncPreviewScroll}
                                />
                            {:else if mode === "preview"}
                                <PreviewMode {renderedContent} />
                            {:else if mode === "live"}
                                <LiveMode
                                    {attendees}
                                    {helpRequests}
                                    bind:chatTab
                                    bind:dmTarget
                                    bind:dmMessage
                                    bind:chatMessage
                                    {filteredMessages}
                                    {handleResolveHelp}
                                    sendChat={sendBroadcast}
                                    sendDM={sendDM}
                                />
                            {:else if mode === "feedback"}
                                <FeedbackMode {feedbacks} />
                            {:else if mode === "settings"}
                                <SettingsMode
                                    bind:codelab
                                    {isSaving}
                                    {saveSuccess}
                                    handleSave={handleSave}
                                />
                            {/if}

                            {#if mode === "guide" && codelab}
                                <GuideMode
                                    bind:guide_markdown={codelab.guide_markdown}
                                    codelab_title={codelab.title}
                                    {isSaving}
                                    {handleSave}
                                    {generateGuideWithAi}
                                    isGenerating={isGuideGenerating}
                                />
                            {/if}

                            {#if mode === "materials"}
                                <MaterialsMode
                                    {materials}
                                    bind:newMaterial
                                    bind:materialFileInput
                                    {handleMaterialFileSelect}
                                    {handleAddMaterial}
                                    {handleDeleteMaterial}
                                />
                            {:else if mode === "submissions"}
                                <SubmissionsMode
                                    {submissions}
                                    onDelete={handleDeleteSubmission}
                                />
                            {:else if mode === "quiz"}
                                <QuizMode
                                    bind:quizzes
                                    {quizSubmissions}
                                    bind:numQuizToGenerate
                                    {isQuizGenerating}
                                    {isSaving}
                                    {saveSuccess}
                                    {generateQuizWithAi}
                                    {addEmptyQuiz}
                                    {removeQuiz}
                                    {handleQuizSave}
                                />
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
