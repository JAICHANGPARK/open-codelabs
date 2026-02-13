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
        getInlineComments,
        ASSET_URL,
        uploadImage,
        getFeedback,
        getMaterials,
        addMaterial,
        deleteMaterial,
        uploadMaterial,
        isServerlessMode,
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
    import {
        streamGeminiResponseRobust,
        streamGeminiStructuredOutput,
        type GeminiStructuredConfig,
    } from "$lib/gemini";
    import { adminMarked as marked } from "$lib/markdown";
    import DOMPurify from "dompurify";
    import { decrypt } from "$lib/crypto";
    import { getQuizzes, updateQuizzes, getQuizSubmissions } from "$lib/api";
    // ... icons imports ...
    import { Plus } from "lucide-svelte";
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
    import ImageGalleryMode from "$lib/components/admin/ImageGalleryMode.svelte";
    import RaffleMode from "$lib/components/admin/RaffleMode.svelte";
    import CertificateMode from "$lib/components/admin/CertificateMode.svelte";
    import AiConversationsMode from "$lib/components/admin/AiConversationsMode.svelte";
    import WorkspaceBrowser from "$lib/components/admin/WorkspaceBrowser.svelte";
    import WorkspaceMode from "$lib/components/admin/WorkspaceMode.svelte";
    import MonitoringDashboard from "$lib/components/admin/MonitoringDashboard.svelte";

    import {
        getSubmissions,
        deleteSubmission as apiDeleteSubmission,
    } from "$lib/api";

    let id = page.params.id as string;
    let activeStepIndex = $state(0);

    // Initialize mode from URL or default to 'edit'
    let initialMode = page.url.searchParams.get("mode");
    let mode = $state<
        | "edit"
        | "preview"
        | "guide"
        | "live"
        | "feedback"
        | "materials"
        | "quiz"
        | "submissions"
        | "gallery"
        | "settings"
        | "workspace"
        | "raffle"
        | "certificate"
        | "ai"
        | "monitoring"
    >(
        initialMode === "preview" ||
            initialMode === "guide" ||
            initialMode === "live" ||
            initialMode === "feedback" ||
            initialMode === "materials" ||
            initialMode === "quiz" ||
            initialMode === "submissions" ||
            initialMode === "gallery" ||
            initialMode === "settings" ||
            initialMode === "workspace" ||
            initialMode === "raffle" ||
            initialMode === "certificate" ||
            initialMode === "ai" ||
            initialMode === "monitoring"
            ? (initialMode as any)
            : "edit",
    );

    let isSaving = $state(false);
    type AdminCodelab = Codelab & { guide_markdown: string };
    let codelab = $state<AdminCodelab | null>(null);
    let steps = $state<Step[]>([]);
    let saveSuccess = $state(false);
    let loading = $state(true);
    let copySuccess = $state(false);
    let savedStepContentById = $state<Record<string, string>>({});
    let savedGuideMarkdown = $state("");
    let showInlineStaleWarningModal = $state(false);
    let inlineStaleImpactItems = $state<
        { label: string; thread_count: number; message_count: number }[]
    >([]);
    let inlineStaleImpactThreads = $state(0);
    let inlineStaleImpactMessages = $state(0);

    let attendees = $state<Attendee[]>([]);
    let helpRequests = $state<HelpRequest[]>([]);
    let feedbacks = $state<Feedback[]>([]); // Feedback
    let materials = $state<Material[]>([]);
    let submissions = $state<any[]>([]);
    let quizzes = $state<Quiz[]>([]);
    let quizSubmissions = $state<any[]>([]);
    let isQuizGenerating = $state(false);
    let isGuideGenerating = $state(false);
    let isGuideProGenerating = $state(false);
    let guideProStage = $state<"plan" | "draft" | "review" | "revise" | null>(
        null,
    );
    let guideProPlanOutput = $state("");
    let guideProDraftOutput = $state("");
    let guideProReviewOutput = $state("");
    let guideProRevisedOutput = $state("");
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
            senderId?: string;
        }[]
    >([]);
    let chatTab = $state<"public" | "direct">("public");
    let dmTarget = $state<Attendee | null>(null);
    let dmMessage = $state("");
    let fileInput = $state<HTMLInputElement>(); // File input ref
    let chatUploadLoading = $state(false);

    // AI State
    let geminiApiKey = $state("");
    let showAiMenu = $state(false);
    let suppressAiMenuClose = $state(false);
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

    function hashText(content: string): string {
        let hash = 2166136261;
        for (let i = 0; i < content.length; i++) {
            hash ^= content.charCodeAt(i);
            hash = Math.imul(hash, 16777619);
        }
        return (hash >>> 0).toString(16);
    }

    function captureSavedContentSnapshot() {
        const nextMap: Record<string, string> = {};
        for (const step of steps) {
            if (!step.id) continue;
            nextMap[step.id] = step.content_markdown || "";
        }
        savedStepContentById = nextMap;
        savedGuideMarkdown = codelab?.guide_markdown || "";
    }

    function closeInlineStaleWarningModal() {
        showInlineStaleWarningModal = false;
        inlineStaleImpactItems = [];
        inlineStaleImpactThreads = 0;
        inlineStaleImpactMessages = 0;
    }

    async function collectInlineStaleImpact() {
        if (!codelab) {
            return {
                items: [] as {
                    label: string;
                    thread_count: number;
                    message_count: number;
                }[],
                total_threads: 0,
                total_messages: 0,
            };
        }

        const changedSteps = steps.filter((step) => {
            const previous = savedStepContentById[step.id] ?? "";
            const current = step.content_markdown || "";
            return previous !== current;
        });
        const guideChanged =
            (savedGuideMarkdown || "") !== (codelab.guide_markdown || "");

        if (changedSteps.length === 0 && !guideChanged) {
            return {
                items: [] as {
                    label: string;
                    thread_count: number;
                    message_count: number;
                }[],
                total_threads: 0,
                total_messages: 0,
            };
        }

        let threads = [] as Awaited<ReturnType<typeof getInlineComments>>;
        try {
            threads = await getInlineComments(id);
        } catch (e) {
            console.error("Failed to load inline comments before save:", e);
            return {
                items: [] as {
                    label: string;
                    thread_count: number;
                    message_count: number;
                }[],
                total_threads: 0,
                total_messages: 0,
            };
        }

        const items: {
            label: string;
            thread_count: number;
            message_count: number;
        }[] = [];
        let totalThreads = 0;
        let totalMessages = 0;

        for (const step of changedSteps) {
            const previousHash = hashText(savedStepContentById[step.id] ?? "");
            const nextHash = hashText(step.content_markdown || "");
            if (previousHash === nextHash) continue;

            const affected = threads.filter(
                (thread) =>
                    thread.target_type === "step" &&
                    thread.target_step_id === step.id &&
                    thread.content_hash === previousHash &&
                    thread.content_hash !== nextHash,
            );
            if (affected.length === 0) continue;

            const messageCount = affected.reduce(
                (sum, thread) => sum + thread.messages.length,
                0,
            );
            items.push({
                label: `Step ${step.step_number}: ${step.title}`,
                thread_count: affected.length,
                message_count: messageCount,
            });
            totalThreads += affected.length;
            totalMessages += messageCount;
        }

        if (guideChanged) {
            const previousHash = hashText(savedGuideMarkdown || "");
            const nextHash = hashText(codelab.guide_markdown || "");
            if (previousHash !== nextHash) {
                const affected = threads.filter(
                    (thread) =>
                        thread.target_type === "guide" &&
                        thread.content_hash === previousHash &&
                        thread.content_hash !== nextHash,
                );
                if (affected.length > 0) {
                    const messageCount = affected.reduce(
                        (sum, thread) => sum + thread.messages.length,
                        0,
                    );
                    items.push({
                        label: $t("editor.guide_tab") || "Guide",
                        thread_count: affected.length,
                        message_count: messageCount,
                    });
                    totalThreads += affected.length;
                    totalMessages += messageCount;
                }
            }
        }

        return {
            items,
            total_threads: totalThreads,
            total_messages: totalMessages,
        };
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
        } else if (mode === "gallery") {
            loadSubmissions();
        } else if (mode === "quiz") {
            loadQuizzes();
            loadQuizSubmissions();
        } else if (mode === "live") {
            refreshLiveData();
            scrollToBottom();
        } else if (mode === "raffle") {
            refreshLiveData();
        }
    });

    async function loadSubmissions() {
        try {
            console.log("Loading submissions for codelab:", id);
            submissions = await getSubmissions(id);
            console.log("Loaded submissions:", submissions);
        } catch (e) {
            console.error("Failed to load submissions:", e);
        }
    }

    async function handleDeleteSubmission(
        attendeeId: string,
        submissionId: string,
    ) {
        if (!confirm($t("common.confirm_delete"))) return;
        try {
            await apiDeleteSubmission(id, attendeeId, submissionId);
            submissions = submissions.filter((s) => s.id !== submissionId);
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
            codelab = {
                ...data[0],
                guide_markdown: data[0].guide_markdown ?? "",
            };
            steps = data[1];
            captureSavedContentSnapshot();

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
            if (wsCleanup && typeof wsCleanup === "function") wsCleanup();
            if (ws) ws.close();
        };
    });

    function handleSelectionChange() {
        if (mode !== "edit" || aiLoading) return;

        const activeElement = document.activeElement as HTMLTextAreaElement;
        if (
            activeElement &&
            activeElement.tagName === "TEXTAREA" &&
            !activeElement.closest(".ai-menu-container")
        ) {
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

    function openAiMenuFromSelection(e: MouseEvent, offsetY = -40) {
        const activeElement = document.activeElement as HTMLTextAreaElement;
        const isTextArea =
            activeElement &&
            activeElement.tagName === "TEXTAREA" &&
            !activeElement.closest(".ai-menu-container");

        if (!isTextArea) return false;

        const start = activeElement.selectionStart;
        const end = activeElement.selectionEnd;

        if (start === end) return false;

        const text = activeElement.value.substring(start, end);
        if (!text.trim()) return false;

        selectedText = text;
        selectionRange = { start, end };
        aiInstruction = "";

        let x = e.clientX;
        let y = e.clientY + offsetY;

        const menuWidth = 320;
        const menuHeight = Math.min(520, Math.floor(window.innerHeight * 0.7));

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
        // Keep selection highlighted for user clarity
        setTimeout(() => {
            try {
                activeElement.focus();
                activeElement.setSelectionRange(start, end);
            } catch (err) {
                // ignore selection restore errors
            }
        }, 0);
        return true;
    }

    function handleMouseUp(e: MouseEvent) {
        if (mode !== "edit") return;
        setTimeout(() => {
            if (suppressAiMenuClose) return;
            if (!aiLoading && showAiMenu) {
                const target = e.target as HTMLElement;
                if (!target.closest(".ai-menu-container")) {
                    showAiMenu = false;
                }
            }
        }, 10);
    }

    function handleContextMenu(e: MouseEvent) {
        if (mode !== "edit") return;
        const opened = openAiMenuFromSelection(e, 8);
        if (opened) {
            e.preventDefault();
            suppressAiMenuClose = true;
            setTimeout(() => {
                suppressAiMenuClose = false;
            }, 200);
        }
    }

    function openAiMenuForFullDoc(pos: { x: number; y: number }) {
        if (mode !== "edit") return;
        selectedText = "";
        selectionRange = null;
        menuPos = pos;
        showAiMenu = true;
    }

    async function improveWithAi(instructionOverride?: string) {
        if (!geminiApiKey) {
            alert($t("ai_generator.api_key_required"));
            return;
        }
        const currentMarkdown = steps[activeStepIndex].content_markdown;
        let targetText = selectedText;
        let targetRange = selectionRange;

        if (!targetText || !targetRange) {
            targetText = currentMarkdown;
            targetRange = { start: 0, end: currentMarkdown.length };
        }

        aiLoading = true;
        showAiMenu = false; // Hide menu

        const originalMarkdown = currentMarkdown;
        const { start, end } = targetRange;

        const instruction = instructionOverride || aiInstruction;
        let prompt = `Improve the following technical writing/markdown content. Make it clearer, correct grammar, and better formatted. Maintain the original meaning. Only return the improved content, no explanations.\n\nContent:\n${targetText}`;
        if (instruction.trim()) {
            prompt = `Improve the following technical writing/markdown content based on this instruction: "${instruction}".\nMake it clearer, correct grammar, and better formatted. Maintain the original meaning where possible. Only return the improved content, no explanations.\n\nContent:\n${targetText}`;
        }
        const systemPrompt = "You are a helpful technical editor.";

        try {
            let fullImprovedContent = "";

            const stream = streamGeminiResponseRobust(prompt, systemPrompt, {
                apiKey: geminiApiKey,
            });

            for await (const chunk of stream) {
                if (chunk.text) {
                    fullImprovedContent += chunk.text;
                }
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
            quizzes = rawQuizzes.map((q) => ({
                ...q,
                options:
                    typeof q.options === "string"
                        ? JSON.parse(q.options)
                        : q.options,
                correct_answers:
                    typeof q.correct_answers === "string"
                        ? JSON.parse(q.correct_answers)
                        : q.correct_answers || [q.correct_answer],
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
        quizzes = [
            ...quizzes,
            {
                id: "",
                codelab_id: id,
                question: "",
                quiz_type: "multiple_choice",
                options: ["", "", "", ""],
                correct_answer: 0,
                correct_answers: [0],
            } as any,
        ];
    }

    function removeQuiz(index: number) {
        quizzes = quizzes.filter((_, i) => i !== index);
    }

    async function handleQuizSave() {
        if (!codelab) return;
        isSaving = true;
        try {
            await updateQuizzes(
                id,
                quizzes.map((q) => ({
                    question: q.question,
                    quiz_type: q.quiz_type || "multiple_choice",
                    options: Array.isArray(q.options)
                        ? q.options
                        : typeof q.options === "string"
                          ? JSON.parse(q.options)
                          : [],
                    correct_answer: q.correct_answer,
                    correct_answers: Array.isArray(q.correct_answers)
                        ? q.correct_answers
                        : typeof q.correct_answers === "string"
                          ? JSON.parse(q.correct_answers)
                          : [q.correct_answer],
                })),
            );
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
            const targetLanguage = resolveTargetLanguage();
            const context = steps
                .map(
                    (s) =>
                        `Step ${s.step_number}: ${s.title}\n${s.content_markdown}`,
                )
                .join("\n\n");
            const prompt = `Based on the following codelab content, generate ${numQuizToGenerate} multiple-choice questions. 
            Each question must have exactly 5 options. 
            Write ALL quiz questions and options in ${targetLanguage}.
            Return ONLY a valid JSON array of objects with this structure:
            [{"question": "string", "options": ["string", "string", "string", "string", "string"], "correct_answer": number (0-4)}]
            
            Codelab Content:
            ${context}`;

            const stream = streamGeminiResponseRobust(
                prompt,
                `You are a helpful education assistant that generates quizzes. You MUST write everything in ${targetLanguage}.`,
                {
                    apiKey: geminiApiKey,
                },
            );

            let responseText = "";
            for await (const chunk of stream) {
                if (chunk.text) {
                    responseText += chunk.text;
                }
            }

            // Extract JSON from response (sometimes Gemini adds markdown code blocks)
            const jsonMatch = responseText.match(/\[.*\]/s);
            if (jsonMatch) {
                const newQuizzes = JSON.parse(jsonMatch[0]);
                quizzes = [
                    ...quizzes,
                    ...newQuizzes.map((q: any) => ({
                        ...q,
                        id: "",
                        codelab_id: id,
                    })),
                ];
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

    const MAX_GUIDE_PROMPT_CHARS = 30_000;
    const MAX_GUIDE_STEP_CHARS = 2000;
    const GUIDE_CONTEXT_TRUNCATION_NOTE =
        "Note: The codelab content was truncated to fit the model's input limit.";
    const GUIDE_SECTION_TRUNCATION_NOTE =
        "Note: The following section was truncated to fit the model's input limit.";
    const PROMPT_CONTEXT_PREFIX_LEN = "Context:\n".length;
    const PROMPT_QUESTION_PREFIX_LEN = "\n\nQuestion:\n".length;

    const normalizeGuideContent = (content: string, maxChars: number) => {
        const compact = content
            .replace(/\r\n/g, "\n")
            .replace(/\n{3,}/g, "\n\n")
            .trim();
        if (compact.length <= maxChars) return compact;
        return `${compact.slice(0, maxChars)}\n...`;
    };

    const truncateForPrompt = (text: string, maxChars: number) => {
        const trimmed = text.trim();
        if (maxChars <= 0) {
            return { text: "...", truncated: true };
        }
        if (trimmed.length <= maxChars) {
            return { text: trimmed, truncated: false };
        }
        const sliceLen = Math.max(0, maxChars - 4);
        return { text: `${trimmed.slice(0, sliceLen)}\n...`, truncated: true };
    };

    const clampPrompt = (text: string) => {
        if (text.length <= MAX_GUIDE_PROMPT_CHARS) return text;
        return truncateForPrompt(text, MAX_GUIDE_PROMPT_CHARS).text;
    };

    const formatPromptSection = (
        label: string,
        content: string,
        maxChars: number,
    ) => {
        const { text, truncated } = truncateForPrompt(content, maxChars);
        return `${label}\n${text}${
            truncated ? `\n${GUIDE_SECTION_TRUNCATION_NOTE}` : ""
        }`;
    };

    const buildGuideContext = (maxChars: number) => {
        let contextText = "";
        let truncated = false;

        steps.forEach((step, index) => {
            if (truncated) return;
            const stepNumber = step.step_number || index + 1;
            const content = normalizeGuideContent(
                step.content_markdown || "",
                MAX_GUIDE_STEP_CHARS,
            );
            const block = `Step ${stepNumber}: ${step.title}\n${content}`;
            const nextLength = contextText.length + block.length + 2;

            if (nextLength > maxChars) {
                const remaining = maxChars - contextText.length;
                if (remaining > 0) {
                    const clipped = block.slice(0, Math.max(0, remaining - 4));
                    contextText += `${clipped}\n...`;
                }
                truncated = true;
                return;
            }

            contextText += `${block}\n\n`;
        });

        return {
            context: contextText.trim(),
            truncated,
        };
    };

    const buildGuidePrompt = (promptHeader: string, maxChars: number) => {
        const footerReserve = GUIDE_CONTEXT_TRUNCATION_NOTE.length + 2;
        const maxContextChars = Math.max(
            0,
            maxChars - promptHeader.length - footerReserve,
        );
        const { context, truncated } = buildGuideContext(maxContextChars);
        return {
            prompt: `${promptHeader}${context}${
                truncated ? `\n\n${GUIDE_CONTEXT_TRUNCATION_NOTE}` : ""
            }`,
            truncated,
        };
    };

    function resolveTargetLanguage() {
        const userLanguage = $locale || "en";
        const languageNames: Record<string, string> = {
            ko: "Korean",
            en: "English",
            zh: "Chinese",
            ja: "Japanese",
        };
        return languageNames[userLanguage] || "English";
    }

    function parseStructuredJson<T>(raw: string): T | null {
        const trimmed = raw.trim();
        if (!trimmed) return null;
        const firstBrace = trimmed.indexOf("{");
        const lastBrace = trimmed.lastIndexOf("}");
        if (firstBrace === -1 || lastBrace === -1) return null;
        const jsonText = trimmed.substring(firstBrace, lastBrace + 1);
        try {
            return JSON.parse(jsonText) as T;
        } catch {
            return null;
        }
    }

    type GuidePlanSection = {
        title: string;
        items: string[];
    };

    type GuidePlan = {
        summary: string;
        audience: string;
        prerequisites: string[];
        sections: GuidePlanSection[];
        checklist: string[];
        search_terms: string[];
    };

    type GuideDraft = {
        markdown: string;
    };

    type GuideReviewIssue = {
        severity: string;
        issue: string;
        recommendation: string;
    };

    type GuideReview = {
        expert: {
            summary: string;
            issues: GuideReviewIssue[];
            missing: string[];
            improvements: string[];
        };
        novice: {
            summary: string;
            confusion_points: string[];
            missing: string[];
            improvements: string[];
        };
    };

    const buildGuidePlanSchema = (targetLanguage: string) => ({
        type: "object",
        properties: {
            summary: {
                type: "string",
                description: `Plan summary in ${targetLanguage}`,
            },
            audience: {
                type: "string",
                description: `Target audience description in ${targetLanguage}`,
            },
            prerequisites: {
                type: "array",
                items: {
                    type: "string",
                    description: `Prerequisite in ${targetLanguage}`,
                },
            },
            sections: {
                type: "array",
                items: {
                    type: "object",
                    properties: {
                        title: {
                            type: "string",
                            description: `Section title in ${targetLanguage}`,
                        },
                        items: {
                            type: "array",
                            items: {
                                type: "string",
                                description: `Section bullet in ${targetLanguage}`,
                            },
                        },
                    },
                    required: ["title", "items"],
                },
            },
            checklist: {
                type: "array",
                items: {
                    type: "string",
                    description: `Checklist item in ${targetLanguage}`,
                },
            },
            search_terms: {
                type: "array",
                items: {
                    type: "string",
                    description:
                        "Short English search query for latest versions or commands",
                },
            },
        },
        required: [
            "summary",
            "audience",
            "prerequisites",
            "sections",
            "checklist",
            "search_terms",
        ],
    });

    const buildGuideDraftSchema = (targetLanguage: string) => ({
        type: "object",
        properties: {
            markdown: {
                type: "string",
                description: `Full preparation guide in markdown written in ${targetLanguage}`,
            },
        },
        required: ["markdown"],
    });

    const buildGuideReviewSchema = (targetLanguage: string) => ({
        type: "object",
        properties: {
            expert: {
                type: "object",
                properties: {
                    summary: {
                        type: "string",
                        description: `Expert review summary in ${targetLanguage}`,
                    },
                    issues: {
                        type: "array",
                        items: {
                            type: "object",
                            properties: {
                                severity: {
                                    type: "string",
                                    description: `Severity in ${targetLanguage}`,
                                },
                                issue: {
                                    type: "string",
                                    description: `Issue in ${targetLanguage}`,
                                },
                                recommendation: {
                                    type: "string",
                                    description: `Recommendation in ${targetLanguage}`,
                                },
                            },
                            required: ["severity", "issue", "recommendation"],
                        },
                    },
                    missing: {
                        type: "array",
                        items: {
                            type: "string",
                            description: `Missing item in ${targetLanguage}`,
                        },
                    },
                    improvements: {
                        type: "array",
                        items: {
                            type: "string",
                            description: `Improvement in ${targetLanguage}`,
                        },
                    },
                },
                required: ["summary", "issues", "missing", "improvements"],
            },
            novice: {
                type: "object",
                properties: {
                    summary: {
                        type: "string",
                        description: `Novice review summary in ${targetLanguage}`,
                    },
                    confusion_points: {
                        type: "array",
                        items: {
                            type: "string",
                            description: `Confusion point in ${targetLanguage}`,
                        },
                    },
                    missing: {
                        type: "array",
                        items: {
                            type: "string",
                            description: `Missing item in ${targetLanguage}`,
                        },
                    },
                    improvements: {
                        type: "array",
                        items: {
                            type: "string",
                            description: `Improvement in ${targetLanguage}`,
                        },
                    },
                },
                required: [
                    "summary",
                    "confusion_points",
                    "missing",
                    "improvements",
                ],
            },
        },
        required: ["expert", "novice"],
    });

    async function streamStructuredJson<T>(
        prompt: string,
        systemPrompt: string,
        schema: object,
        config: GeminiStructuredConfig,
    ): Promise<T> {
        let responseText = "";
        const stream = streamGeminiStructuredOutput(
            prompt,
            systemPrompt,
            schema,
            config,
        );
        for await (const chunk of stream) {
            if (chunk.content) {
                responseText += chunk.content;
            }
        }
        const parsed = parseStructuredJson<T>(responseText);
        if (!parsed) {
            throw new Error($t("ai_generator.error_parse"));
        }
        return parsed;
    }

    async function generateGuideWithAi() {
        if (!geminiApiKey || !codelab) {
            alert($t("ai_generator.api_key_required"));
            return;
        }
        if (isGuideProGenerating) return;
        isGuideGenerating = true;

        try {
            const targetLanguage = resolveTargetLanguage();

            const systemPrompt =
                `You are a professional developer advocate writing a preparation guide for a workshop. ` +
                `You MUST write everything in ${targetLanguage}.`;
            const promptHeader = `Based on the following codelab content, create a comprehensive "Preparation & Setup Guide" for attendees. 
            Include:
            1. System requirements (Prerequisites).
            2. Required software/tools to install.
            3. Language/framework installation guidance (where to download, how to install, and version verification).
            4. Environment variable and PATH setup steps (what to set and how to verify).
            5. Environment setup instructions.
            6. Initial project boilerplate setup if necessary.
            7. A local environment smoke test with minimal runnable code and commands for the primary language in the codelab. The test must pass to confirm readiness.
            8. A "Glossary" section that lists programming languages, tools, and key terms from the codelab with brief definitions.
            
            Write ALL content in ${targetLanguage}.
            Write it in professional markdown. 
            
            Codelab Title: ${codelab.title}
            Description: ${codelab.description}
            
            Codelab Content:
            `;
            const promptBudget = Math.max(
                0,
                MAX_GUIDE_PROMPT_CHARS -
                    systemPrompt.length -
                    PROMPT_CONTEXT_PREFIX_LEN -
                    PROMPT_QUESTION_PREFIX_LEN,
            );
            const { prompt } = buildGuidePrompt(promptHeader, promptBudget);

            const stream = streamGeminiResponseRobust(prompt, systemPrompt, {
                apiKey: geminiApiKey,
                model: "gemini-3-flash-preview",
            });

            let responseText = "";
            codelab.guide_markdown = "";
            for await (const chunk of stream) {
                if (chunk.text) {
                    responseText += chunk.text;
                    codelab.guide_markdown = responseText;
                }
            }
            await saveGuideMarkdown();
        } catch (e) {
            console.error("Guide generation failed:", e);
            const errorMessage = (e as any)?.message || String(e);
            alert($t("ai_generator.error_generate") + ": " + errorMessage);
        } finally {
            isGuideGenerating = false;
        }
    }

    async function saveGuideMarkdown() {
        await handleSave();
    }

    async function generateGuideWithAiPro() {
        if (!geminiApiKey || !codelab) {
            alert($t("ai_generator.api_key_required"));
            return;
        }
        if (isGuideGenerating || isGuideProGenerating) return;
        isGuideProGenerating = true;
        guideProStage = "plan";
        guideProPlanOutput = "";
        guideProDraftOutput = "";
        guideProReviewOutput = "";
        guideProRevisedOutput = "";

        try {
            const targetLanguage = resolveTargetLanguage();
            const systemPrompt =
                `You are a senior developer advocate and technical writer. ` +
                `You MUST write everything in ${targetLanguage}.`;

            const planHeader =
                `Create a professional plan for a "Preparation & Setup Guide" for attendees. ` +
                `Include prerequisites, setup flow, environment checks, and common pitfalls. ` +
                `Add a dedicated section for language/framework installation (download location, install steps, version check). ` +
                `Add a dedicated section for environment variables/PATH setup and verification. ` +
                `Provide a clear outline, checklist, and search_terms for latest info. ` +
                `Include a "Local Environment Smoke Test" section with minimal runnable code and commands for the primary language in the codelab. ` +
                `Include a "Glossary" section that lists programming languages, tools, and key terms from the codelab with brief definitions. ` +
                `For search_terms, use short English queries. ` +
                `Write ALL content in ${targetLanguage}.\n\n` +
                `Codelab Title: ${codelab.title}\n` +
                `Description: ${codelab.description}\n\n` +
                `Codelab Content:\n`;
            const { prompt: planPrompt } = buildGuidePrompt(
                planHeader,
                MAX_GUIDE_PROMPT_CHARS,
            );

            const planData = await streamStructuredJson<GuidePlan>(
                planPrompt,
                systemPrompt,
                buildGuidePlanSchema(targetLanguage),
                {
                    apiKey: geminiApiKey,
                    model: "gemini-3-flash-preview",
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );
            guideProPlanOutput = JSON.stringify(planData, null, 2);

            guideProStage = "draft";

            const searchTerms = planData.search_terms || [];
            const searchHint = searchTerms.length
                ? `Use the Google Search tool to verify the latest info for these queries: ${searchTerms.join(
                      ", ",
                  )}.`
                : "Use the Google Search tool if any versions, commands, or APIs need verification.";

            const planJson = JSON.stringify(planData, null, 2);
            const planBudget = Math.min(
                8000,
                Math.floor(MAX_GUIDE_PROMPT_CHARS * 0.3),
            );
            const planBlock = formatPromptSection(
                "Plan JSON:",
                planJson,
                planBudget,
            );

            const draftHeader =
                `Write the full preparation guide using the plan. ` +
                `${searchHint} Write ALL content in ${targetLanguage}. ` +
                `Use clear headings, checklists, and code blocks when needed. ` +
                `Include explicit language/framework installation steps (download link location, installation commands/steps, version verification). ` +
                `Include environment variable/PATH setup steps and verification commands. ` +
                `Include a "Local Environment Smoke Test" section with minimal runnable code and commands for the primary language in the codelab, and explain that the test must pass to confirm readiness. ` +
                `Include a "Glossary" section that lists programming languages, tools, and key terms from the codelab with brief definitions.\n\n` +
                `${planBlock}\n\n` +
                `Codelab Title: ${codelab.title}\n` +
                `Description: ${codelab.description}\n\n` +
                `Codelab Content:\n`;
            const { prompt: draftPrompt } = buildGuidePrompt(
                draftHeader,
                MAX_GUIDE_PROMPT_CHARS,
            );

            const draftData = await streamStructuredJson<GuideDraft>(
                draftPrompt,
                systemPrompt,
                buildGuideDraftSchema(targetLanguage),
                {
                    apiKey: geminiApiKey,
                    model: "gemini-3-flash-preview",
                    tools: [{ googleSearch: {} }],
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );
            guideProDraftOutput = draftData?.markdown || "";

            if (draftData?.markdown) {
                codelab.guide_markdown = draftData.markdown;
            }

            guideProStage = "review";

            const reviewPlanBudget = Math.min(
                6000,
                Math.floor(MAX_GUIDE_PROMPT_CHARS * 0.2),
            );
            const reviewDraftBudget = Math.min(
                16000,
                Math.floor(MAX_GUIDE_PROMPT_CHARS * 0.6),
            );
            const reviewPrompt = clampPrompt(
                `Review the preparation guide from two perspectives: expert and novice. ` +
                    `Be critical, practical, and specific. Write ALL content in ${targetLanguage}.\n\n` +
                    `${formatPromptSection(
                        "Plan JSON:",
                        planJson,
                        reviewPlanBudget,
                    )}\n\n` +
                    `${formatPromptSection(
                        "Draft Guide Markdown:",
                        draftData?.markdown || "",
                        reviewDraftBudget,
                    )}`,
            );

            const reviewData = await streamStructuredJson<GuideReview>(
                reviewPrompt,
                systemPrompt,
                buildGuideReviewSchema(targetLanguage),
                {
                    apiKey: geminiApiKey,
                    model: "gemini-3-flash-preview",
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );
            guideProReviewOutput = JSON.stringify(reviewData, null, 2);

            guideProStage = "revise";

            const revisePlanBudget = Math.min(
                6000,
                Math.floor(MAX_GUIDE_PROMPT_CHARS * 0.2),
            );
            const reviseDraftBudget = Math.min(
                15000,
                Math.floor(MAX_GUIDE_PROMPT_CHARS * 0.55),
            );
            const reviseReviewBudget = Math.min(
                6000,
                Math.floor(MAX_GUIDE_PROMPT_CHARS * 0.2),
            );
            const reviewJson = JSON.stringify(reviewData, null, 2);
            const revisePrompt = clampPrompt(
                `Revise the preparation guide based on the expert and novice reviews. ` +
                    `${searchHint} Write ALL content in ${targetLanguage}. ` +
                    `Ensure the guide includes language/framework installation steps (download location, install steps, version verification). ` +
                    `Ensure the guide includes environment variable/PATH setup steps and verification. ` +
                    `Ensure the guide includes a "Local Environment Smoke Test" section with minimal runnable code and commands for the primary language in the codelab. ` +
                    `Ensure the guide includes a "Glossary" section that lists programming languages, tools, and key terms from the codelab with brief definitions.\n\n` +
                    `${formatPromptSection(
                        "Plan JSON:",
                        planJson,
                        revisePlanBudget,
                    )}\n\n` +
                    `${formatPromptSection(
                        "Draft Guide Markdown:",
                        draftData?.markdown || "",
                        reviseDraftBudget,
                    )}\n\n` +
                    `${formatPromptSection(
                        "Review JSON:",
                        reviewJson,
                        reviseReviewBudget,
                    )}`,
            );

            const revisedData = await streamStructuredJson<GuideDraft>(
                revisePrompt,
                systemPrompt,
                buildGuideDraftSchema(targetLanguage),
                {
                    apiKey: geminiApiKey,
                    model: "gemini-3-flash-preview",
                    tools: [{ googleSearch: {} }],
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );
            guideProRevisedOutput = revisedData?.markdown || "";

            if (revisedData?.markdown) {
                codelab.guide_markdown = revisedData.markdown;
            }
            await saveGuideMarkdown();
        } catch (e: any) {
            console.error("Guide pro generation failed:", e);
            const errorMessage = e?.message || String(e);
            alert($t("ai_generator.error_generate") + ": " + errorMessage);
        } finally {
            isGuideProGenerating = false;
            guideProStage = null;
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
                link_url:
                    newMaterial.material_type === "link"
                        ? newMaterial.link_url
                        : undefined,
                file_path:
                    newMaterial.material_type === "file"
                        ? newMaterial.file_path
                        : undefined,
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
                            senderId: msg.target_id,
                        };
                    } else {
                        return {
                            sender: `[DM] ${msg.sender_name}`,
                            text: msg.message,
                            time: timeStr,
                            self: false,
                            type: "dm",
                            senderId: msg.sender_id || msg.sender_name,
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
        if (isServerlessMode()) {
            return listenToWsReplacement(id, (data) => {
                if (data.type === "chat") {
                    if (
                        messages.find(
                            (m) =>
                                m.text === data.message &&
                                m.sender === data.sender_name,
                        )
                    )
                        return;
                    messages = [
                        ...messages,
                        {
                            sender: data.sender_name,
                            text: data.message,
                            time: data.created_at?.toDate
                                ? data.created_at.toDate().toLocaleTimeString()
                                : new Date().toLocaleTimeString(),
                            self: false,
                            type: "chat",
                        },
                    ];
                } else if (data.type === "help_request") {
                    refreshLiveData();
                }
            });
        }

        const adminToken = localStorage.getItem("adminToken");
        const wsUrl = getWsUrl(id, "admin", adminToken || undefined);
        const newWs = new WebSocket(wsUrl);

        newWs.onopen = () => {
            // Identify as facilitator
            newWs.send(JSON.stringify({ type: "facilitator" }));
        };

        newWs.addEventListener("message", (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === "chat") {
                    // Prevent duplicate messages from own broadcasts
                    if (
                        messages.find(
                            (m) =>
                                m.text === data.message &&
                                m.sender === data.sender &&
                                m.time ===
                                    (data.timestamp ||
                                        new Date().toLocaleTimeString()),
                        )
                    ) {
                        return;
                    }
                    messages = [
                        ...messages,
                        {
                            sender: data.sender,
                            text: data.message,
                            time:
                                data.timestamp ||
                                new Date().toLocaleTimeString(),
                            self: data.sender === "Facilitator",
                            type: "chat",
                        },
                    ];
                    if (chatTab === "public") scrollToBottom();
                } else if (data.type === "dm") {
                    // Prevent duplicate DM messages
                    if (
                        messages.find(
                            (m) =>
                                m.text === data.message &&
                                m.sender === `[DM] ${data.sender}`,
                        )
                    ) {
                        return;
                    }
                    messages = [
                        ...messages,
                        {
                            sender: `[DM] ${data.sender}`,
                            text: data.message,
                            time:
                                data.timestamp ||
                                new Date().toLocaleTimeString(),
                            self: false,
                            type: "dm",
                            senderId: data.sender_id,
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
                } else if (data.type === "attendee_screen_status") {
                    attendees = attendees.map((a) =>
                        a.id === data.attendee_id
                            ? {
                                  ...a,
                                  is_sharing_screen: data.status === "started",
                              }
                            : a,
                    );
                }
            } catch (e) {
                console.error("WS error:", e);
            }
        });

        newWs.onclose = () => {
            setTimeout(initWebSocket, 3000);
        };
        ws = newWs;
    }

    function sendBroadcast() {
        if (!chatMessage.trim()) return;
        if (isServerlessMode()) {
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

        if (isServerlessMode()) {
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
                    senderId: dmTarget.id,
                },
            ];
            dmMessage = "";
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
                senderId: dmTarget.id,
            },
        ];

        dmMessage = "";
        scrollToBottom();
    }

    async function sendChatImage(file: File) {
        if (chatTab === "direct" && !dmTarget) return;
        if (chatUploadLoading) return;
        chatUploadLoading = true;
        try {
            const { url } = await uploadImage(file);
            const message = `![image](${url})`;
            if (isServerlessMode()) {
                sendChatMessage(id, {
                    sender: "Facilitator",
                    message,
                    type: chatTab === "public" ? "chat" : "dm",
                    target_id: chatTab === "direct" ? dmTarget?.id : undefined,
                });
                if (chatTab === "direct" && dmTarget) {
                    messages = [
                        ...messages,
                        {
                            sender: `To: ${dmTarget.name}`,
                            text: message,
                            time: new Date().toLocaleTimeString(),
                            self: true,
                            type: "dm",
                            senderId: dmTarget.id,
                        },
                    ];
                }
                scrollToBottom();
                return;
            }

            if (!ws) return;
            const msg = {
                type: chatTab === "public" ? "chat" : "dm",
                target_id: chatTab === "direct" ? dmTarget?.id : undefined,
                sender: "Facilitator",
                message,
                timestamp: new Date().toLocaleTimeString([], {
                    hour: "2-digit",
                    minute: "2-digit",
                }),
            };
            ws.send(JSON.stringify(msg));
            if (chatTab === "direct" && dmTarget) {
                messages = [
                    ...messages,
                    {
                        sender: `To: ${dmTarget.name}`,
                        text: message,
                        time: msg.timestamp,
                        self: true,
                        type: "dm",
                        senderId: dmTarget.id,
                    },
                ];
            }
            scrollToBottom();
        } catch (e) {
            alert($t("common.error"));
        } finally {
            chatUploadLoading = false;
        }
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

    function createStepId() {
        if (
            typeof crypto !== "undefined" &&
            typeof crypto.randomUUID === "function"
        ) {
            return crypto.randomUUID();
        }
        return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(
            /[xy]/g,
            (char) => {
                const random = Math.floor(Math.random() * 16);
                const value = char === "x" ? random : (random & 0x3) | 0x8;
                return value.toString(16);
            },
        );
    }

    function addStep() {
        const newStep: Step = {
            id: createStepId(),
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

    async function handleSave(skipStaleWarning = false) {
        if (isSaving || !codelab) return;

        if (!skipStaleWarning) {
            const impact = await collectInlineStaleImpact();
            if (impact.total_threads > 0) {
                inlineStaleImpactItems = impact.items;
                inlineStaleImpactThreads = impact.total_threads;
                inlineStaleImpactMessages = impact.total_messages;
                showInlineStaleWarningModal = true;
                return;
            }
        }

        isSaving = true;
        try {
            await Promise.all([
                saveSteps(
                    id,
                    steps.map((s) => ({
                        id: s.id,
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
                    guide_markdown: codelab.guide_markdown,
                }),
            ]);
            const latest = await getCodelab(id);
            codelab = {
                ...latest[0],
                guide_markdown: latest[0].guide_markdown ?? "",
            };
            steps = latest[1];
            if (activeStepIndex >= steps.length) {
                activeStepIndex = Math.max(0, steps.length - 1);
            }
            captureSavedContentSnapshot();
            closeInlineStaleWarningModal();
            saveSuccess = true;
            setTimeout(() => (saveSuccess = false), 3000);
        } catch (e) {
            alert("Save failed: " + e);
        } finally {
            isSaving = false;
        }
    }

    async function handleConfirmInlineStaleSave() {
        await handleSave(true);
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
                guide_markdown: codelab.guide_markdown,
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

    let showWorkspaceBrowser = $state(false);

    async function handleDownloadWorkspace() {
        try {
            const { downloadCodeServerWorkspace } = await import("$lib/api");
            await downloadCodeServerWorkspace(id);
        } catch (e) {
            if (e instanceof Error) {
                if (e.message.includes("Not supported")) {
                    alert($t("workspace.errors.firebase_unavailable"));
                } else if (e.message.includes("not found")) {
                    alert($t("workspace.errors.not_found"));
                } else {
                    alert(
                        $t("workspace.errors.download_failed", {
                            values: { error: e.message },
                        }),
                    );
                }
            } else {
                alert(
                    $t("workspace.errors.download_failed", {
                        values: { error: String(e) },
                    }),
                );
            }
        }
    }

    function handleBrowseWorkspace() {
        showWorkspaceBrowser = true;
    }

    type InsertOptions = {
        language?: string;
        snippet?: string;
        url?: string;
    };

    function applyEditorReplacement(
        replacement: string,
        start: number,
        end: number,
        selectionStart: number,
        selectionEnd: number,
    ) {
        const textarea = editorEl ?? document.querySelector("textarea");
        if (!textarea) return;

        // Use execCommand to preserve undo history
        textarea.focus();
        textarea.setSelectionRange(start, end);
        try {
            document.execCommand("insertText", false, replacement);
        } catch (e) {
            console.error(
                "execCommand failed, falling back to setRangeText",
                e,
            );
            textarea.setRangeText(replacement, start, end, "preserve");
        }

        textarea.dispatchEvent(new Event("input", { bubbles: true }));

        setTimeout(() => {
            textarea.focus();
            textarea.setSelectionRange(selectionStart, selectionEnd);
        }, 0);
    }

    function insertMarkdown(type: string, options: InsertOptions = {}) {
        if (mode !== "edit" || !steps[activeStepIndex]) return;

        // Handle image special case
        if (type === "image") {
            fileInput?.click();
            return;
        }

        const textarea = editorEl ?? document.querySelector("textarea");
        if (!textarea) return;

        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value || "";
        const selected = text.substring(start, end);
        const language = options.language ?? "";

        let replacement = "";
        let selectionStart = start;
        let selectionEnd = start;

        const setCursorToEnd = () => {
            selectionStart = start + replacement.length;
            selectionEnd = selectionStart;
        };

        const setSelection = (offset: number, length: number) => {
            selectionStart = start + offset;
            selectionEnd = selectionStart + length;
        };

        switch (type) {
            case "bold":
                if (selected) {
                    replacement = `**${selected}**`;
                    setCursorToEnd();
                } else {
                    const placeholder = "bold text";
                    replacement = `**${placeholder}**`;
                    setSelection(2, placeholder.length);
                }
                break;
            case "italic":
                if (selected) {
                    replacement = `*${selected}*`;
                    setCursorToEnd();
                } else {
                    const placeholder = "italic text";
                    replacement = `*${placeholder}*`;
                    setSelection(1, placeholder.length);
                }
                break;
            case "inline_code":
                if (selected) {
                    replacement = `\`${selected}\``;
                    setCursorToEnd();
                } else {
                    const placeholder = "code";
                    replacement = `\`${placeholder}\``;
                    setSelection(1, placeholder.length);
                }
                break;
            case "code":
            case "code_block": {
                const placeholder = selected || "// code here";
                const prefix = language ? `\n\`\`\`${language}\n` : "\n```\n";
                replacement = `${prefix}${placeholder}\n\`\`\`\n`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(prefix.length, placeholder.length);
                }
                break;
            }
            case "h1":
            case "h2":
            case "h3": {
                const level =
                    type === "h1" ? "#" : type === "h2" ? "##" : "###";
                const placeholder = "Heading";
                const content = selected || placeholder;
                replacement = `${level} ${content}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(level.length + 1, placeholder.length);
                }
                break;
            }
            case "list": {
                const placeholder = "list item";
                const lines = selected ? selected.split("\n") : [""];
                const listText = lines
                    .map((line) => `- ${line || placeholder}`)
                    .join("\n");
                replacement = selected ? listText : `\n${listText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "ordered_list": {
                const placeholder = "list item";
                const lines = selected ? selected.split("\n") : [""];
                const listText = lines
                    .map(
                        (line, index) => `${index + 1}. ${line || placeholder}`,
                    )
                    .join("\n");
                replacement = selected ? listText : `\n${listText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "task_list": {
                const placeholder = "task";
                const lines = selected ? selected.split("\n") : [""];
                const listText = lines
                    .map((line) => `- [ ] ${line || placeholder}`)
                    .join("\n");
                replacement = selected ? listText : `\n${listText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "quote": {
                const placeholder = "Quote";
                const lines = selected ? selected.split("\n") : [""];
                const quoteText = lines
                    .map((line) => `> ${line || placeholder}`)
                    .join("\n");
                replacement = selected ? quoteText : `\n${quoteText}`;
                if (selected) {
                    setCursorToEnd();
                } else {
                    setSelection(
                        replacement.length - placeholder.length,
                        placeholder.length,
                    );
                }
                break;
            }
            case "link": {
                const linkText = selected || "link text";
                const url = options.url || "https://";
                const prefix = `[${linkText}](`;
                replacement = `${prefix}${url})`;
                setSelection(prefix.length, url.length);
                break;
            }
            case "table": {
                const header = "| Column | Column | Column |";
                const divider = "| --- | --- | --- |";
                const row = "| Cell | Cell | Cell |";
                const prefix = "\n| ";
                replacement = `\n${header}\n${divider}\n${row}\n`;
                setSelection(prefix.length, "Column".length);
                break;
            }
            case "snippet": {
                const snippet = options.snippet?.trimEnd();
                if (!snippet) return;
                const needsPrefix = start > 0 && text[start - 1] !== "\n";
                const needsSuffix = end < text.length && text[end] !== "\n";
                replacement = `${needsPrefix ? "\n" : ""}${snippet}${needsSuffix ? "\n" : ""}`;
                setCursorToEnd();
                break;
            }
            default:
                return;
        }

        applyEditorReplacement(
            replacement,
            start,
            end,
            selectionStart,
            selectionEnd,
        );
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
            const textarea = editorEl ?? document.querySelector("textarea");
            if (!textarea) return;

            const start = textarea.selectionStart;
            const end = textarea.selectionEnd;
            const text = textarea.value || "";
            const fullUrl = url.startsWith("http") ? url : `${ASSET_URL}${url}`;
            const replacement = `![image](${fullUrl})`;

            const newCursorPos = start + replacement.length;
            applyEditorReplacement(
                replacement,
                start,
                end,
                newCursorPos,
                newCursorPos,
            );
        } catch (e) {
            console.error(e);
            alert("Image upload failed");
        }
    }

    async function handlePaste(event: ClipboardEvent) {
        const items = event.clipboardData?.items;
        if (!items) return;

        // Find the first image item
        let imageItem: DataTransferItem | null = null;
        for (const item of items) {
            if (item.type.indexOf("image") !== -1) {
                imageItem = item;
                break;
            }
        }

        // If an image was found, prevent default paste and upload
        if (imageItem) {
            event.preventDefault();
            const file = imageItem.getAsFile();
            if (file) {
                await uploadAndInsertImage(file);
            }
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (mode !== "edit") return;

        if (e.metaKey || e.ctrlKey) {
            if (e.shiftKey) {
                switch (e.code) {
                    case "Digit7":
                        e.preventDefault();
                        insertMarkdown("ordered_list");
                        return;
                    case "Digit8":
                        e.preventDefault();
                        insertMarkdown("list");
                        return;
                    case "Digit9":
                        e.preventDefault();
                        insertMarkdown("quote");
                        return;
                }
            }

            switch (e.key.toLowerCase()) {
                case "b":
                    e.preventDefault();
                    insertMarkdown("bold");
                    break;
                case "i":
                    e.preventDefault();
                    insertMarkdown("italic");
                    break;
                case "k":
                    e.preventDefault();
                    insertMarkdown("link");
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
    function extractYoutubeId(rawUrl: string): string | null {
        try {
            const url = new URL(rawUrl);
            const host = url.hostname.replace(/^www\./, "");
            if (host === "youtu.be") {
                return url.pathname.replace("/", "").split("/")[0] || null;
            }
            if (host === "youtube.com") {
                if (url.pathname === "/watch") {
                    return url.searchParams.get("v");
                }
                if (url.pathname.startsWith("/embed/")) {
                    return url.pathname.split("/")[2] || null;
                }
                if (url.pathname.startsWith("/shorts/")) {
                    return url.pathname.split("/")[2] || null;
                }
            }
        } catch (e) {
            // ignore invalid urls
        }
        return null;
    }

    function injectYoutubeEmbeds(html: string): string {
        const anchorRegex = /<a[^>]*href="([^"]+)"[^>]*>.*?<\/a>/gi;
        return html.replace(anchorRegex, (full, href) => {
            const id = extractYoutubeId(href);
            if (!id) return full;
            const embedUrl = `https://www.youtube-nocookie.com/embed/${id}`;
            return `
<div class="video-embed" style="width:100%;max-width:100%;display:block;margin:1.25rem 0;">
  <iframe
    src="${embedUrl}"
    title="YouTube video"
    loading="lazy"
    style="width:100%;height:auto;aspect-ratio:16/9;display:block;border:0;border-radius:16px;background:#000;"
    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
    referrerpolicy="strict-origin-when-cross-origin"
    allowfullscreen
  ></iframe>
</div>`;
        });
    }

    let renderedContent = $derived.by(() => {
        if (!currentStep) return "";
        try {
            const html = marked.parse(currentStep.content_markdown) as string;
            if (browser) {
                const sanitized = DOMPurify.sanitize(html);
                return injectYoutubeEmbeds(sanitized);
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

<div
    class="min-h-screen bg-background dark:bg-dark-bg flex flex-col font-sans text-foreground dark:text-dark-text transition-colors"
>
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
        {handleDownloadWorkspace}
        {handleBrowseWorkspace}
        {ws}
    />

    {#if loading}
        <div class="flex-1 flex justify-center items-center">
            <div
                class="animate-spin rounded-full h-12 w-12 border-4 border-border dark:border-dark-border border-t-primary dark:border-t-primary"
            ></div>
        </div>
    {:else}
        <main
            class="max-w-screen-2xl mx-auto w-full p-4 sm:p-8 flex-1 grid grid-cols-1 lg:grid-cols-12 gap-4 items-start relative"
        >
            <!-- Sidebar Navigation -->
            {#if mode !== "live" && mode !== "feedback" && mode !== "materials" && mode !== "quiz" && mode !== "settings" && mode !== "guide" && mode !== "submissions" && mode !== "gallery" && mode !== "workspace" && mode !== "raffle"}
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
                mode === "guide" ||
                mode === "submissions" ||
                mode === "gallery" ||
                mode === "workspace" ||
                mode === "raffle"
                    ? "lg:col-span-12 w-full min-w-0"
                    : "lg:col-span-9 w-full min-w-0"}
                in:fade
            >
                {#if steps.length > 0}
                    <div
                        class="bg-white dark:bg-dark-surface rounded-2xl border border-border dark:border-dark-border shadow-sm min-h-[70vh] flex flex-col transition-colors"
                    >
                        {#if mode === "edit" || mode === "preview"}
                            <div
                                class="p-6 sm:p-8 border-b border-border dark:border-dark-border bg-accent/40 dark:bg-white/5 sticky top-[73px] z-20 backdrop-blur-md rounded-t-2xl"
                            >
                                <input
                                    type="text"
                                    bind:value={steps[activeStepIndex].title}
                                    readonly={mode === "preview"}
                                    class="text-2xl sm:text-3xl font-bold text-foreground dark:text-dark-text w-full bg-transparent outline-none placeholder-muted-foreground/60 dark:placeholder-dark-text-muted border-b-2 border-transparent focus:border-primary transition-all pb-2"
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
                                    {handleContextMenu}
                                    {improveWithAi}
                                    {openAiMenuForFullDoc}
                                    {syncEditorScroll}
                                    {syncPreviewScroll}
                                />
                            {:else if mode === "preview"}
                                <PreviewMode {renderedContent} />
                            {:else if mode === "live"}
                                <LiveMode
                                    {attendees}
                                    {helpRequests}
                                    totalSteps={steps.length}
                                    bind:chatTab
                                    bind:dmTarget
                                    bind:dmMessage
                                    bind:chatMessage
                                    {messages}
                                    {handleResolveHelp}
                                    sendChat={sendBroadcast}
                                    {sendDM}
                                    attachImage={sendChatImage}
                                />
                            {:else if mode === "feedback"}
                                <FeedbackMode {feedbacks} />
                            {:else if mode === "settings"}
                                <SettingsMode
                                    bind:codelab
                                    {isSaving}
                                    {saveSuccess}
                                    handleSave={handleUniversalSave}
                                />
                            {:else if mode === "workspace"}
                                <WorkspaceMode
                                    codelabId={id}
                                    {steps}
                                    {geminiApiKey}
                                />
                            {:else if mode === "raffle"}
                                <RaffleMode
                                    {attendees}
                                    onRefresh={refreshLiveData}
                                />
                            {:else if mode === "certificate"}
                                <CertificateMode {attendees} />
                            {/if}

                            {#if mode === "guide" && codelab}
                                <GuideMode
                                    bind:guide_markdown={codelab.guide_markdown}
                                    codelab_title={codelab.title}
                                    {isSaving}
                                    {handleSave}
                                    {generateGuideWithAi}
                                    {generateGuideWithAiPro}
                                    isGenerating={isGuideGenerating}
                                    {isGuideProGenerating}
                                    {guideProStage}
                                    {guideProPlanOutput}
                                    {guideProDraftOutput}
                                    {guideProReviewOutput}
                                    {guideProRevisedOutput}
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
                            {:else if mode === "gallery"}
                                <ImageGalleryMode {submissions} />
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
                            {:else if mode === "ai"}
                                <AiConversationsMode codelabId={id} />
                            {:else if mode === "monitoring"}
                                <MonitoringDashboard {ws} {attendees} />
                            {/if}
                        </div>
                    </div>
                {:else}
                    <div
                        class="bg-white dark:bg-dark-surface rounded-3xl border-2 border-dashed border-border dark:border-dark-border p-12 sm:p-24 text-center shadow-sm"
                        in:fly={{ y: 20 }}
                    >
                        <div
                            class="w-20 h-20 bg-accent/60 dark:bg-white/5 rounded-full flex items-center justify-center mx-auto mb-8"
                        >
                            <Plus
                                size={40}
                                class="text-muted-foreground/70 dark:text-dark-text-muted"
                            />
                        </div>
                        <h3
                            class="text-2xl font-bold text-foreground dark:text-dark-text mb-3"
                        >
                            {$t("editor.empty_codelab")}
                        </h3>
                        <p
                            class="text-muted-foreground dark:text-dark-text-muted text-lg mb-10 max-w-sm mx-auto"
                        >
                            {$t("editor.empty_desc")}
                        </p>
                        <button
                            onclick={addStep}
                            class="bg-primary text-primary-foreground px-10 py-3 rounded-full font-bold flex items-center gap-2 mx-auto shadow-md hover:shadow-lg transition-all active:scale-95"
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

{#if showWorkspaceBrowser}
    <div
        class="modal-overlay"
        onclick={() => (showWorkspaceBrowser = false)}
        onkeydown={(e) => e.key === "Escape" && (showWorkspaceBrowser = false)}
        role="button"
        tabindex="-1"
    >
        <div
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="button"
            tabindex="-1"
        >
            <WorkspaceBrowser
                codelabId={id}
                onClose={() => (showWorkspaceBrowser = false)}
            />
        </div>
    </div>
{/if}

{#if showInlineStaleWarningModal}
    <div
        class="modal-overlay"
        onclick={closeInlineStaleWarningModal}
        onkeydown={(e) =>
            e.key === "Escape" && closeInlineStaleWarningModal()}
        role="button"
        tabindex="-1"
    >
        <div
            class="w-[min(680px,calc(100vw-24px))] rounded-2xl border border-border dark:border-dark-border bg-white dark:bg-dark-surface shadow-2xl p-6"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="button"
            tabindex="-1"
        >
            <h3 class="text-lg font-bold text-foreground dark:text-dark-text">
                {$t("inline_comment.admin_stale_modal_title") ||
                    "Inline comments will move to previous-version section"}
            </h3>
            <p
                class="mt-2 text-sm text-muted-foreground dark:text-dark-text-muted"
            >
                {$t("inline_comment.admin_stale_modal_desc") ||
                    "Saving these edits changes the content hash. Matching inline comment threads will be marked stale (read/delete only, no reply)."}
            </p>
            <p class="mt-3 text-sm font-semibold text-foreground dark:text-dark-text">
                {$t("inline_comment.admin_stale_summary", {
                    values: {
                        threads: inlineStaleImpactThreads,
                        messages: inlineStaleImpactMessages,
                    },
                }) ||
                    `${inlineStaleImpactThreads} thread(s), ${inlineStaleImpactMessages} message(s) affected`}
            </p>

            <div class="mt-4 max-h-56 overflow-y-auto space-y-2">
                {#each inlineStaleImpactItems as item}
                    <div
                        class="rounded-lg border border-border dark:border-dark-border bg-accent/50 dark:bg-white/5 px-3 py-2 text-sm"
                    >
                        <p class="font-semibold text-foreground dark:text-dark-text">
                            {item.label}
                        </p>
                        <p class="text-xs text-muted-foreground dark:text-dark-text-muted">
                            {item.thread_count} {$t(
                                "inline_comment.admin_stale_threads",
                            ) || "threads"}
                            ·
                            {item.message_count} {$t(
                                "inline_comment.admin_stale_messages",
                            ) || "messages"}
                        </p>
                    </div>
                {/each}
            </div>

            <div class="mt-6 flex justify-end gap-2">
                <button
                    type="button"
                    class="px-4 py-2 rounded-lg border border-border dark:border-dark-border text-sm font-semibold"
                    onclick={closeInlineStaleWarningModal}
                >
                    {$t("common.cancel")}
                </button>
                <button
                    type="button"
                    class="px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-semibold disabled:opacity-50"
                    onclick={handleConfirmInlineStaleSave}
                    disabled={isSaving}
                >
                    {isSaving
                        ? $t("common.loading")
                        : $t("inline_comment.admin_stale_continue") ||
                          "Save anyway"}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    :global(.markdown-body) {
        font-size: 1.1rem;
        line-height: 1.6;
    }
    /* Syntax Highlighting - Handled globally in app.css */
    :global(.markdown-body code:not(pre code)) {
        font-family: inherit;
        color: var(--color-destructive);
        background-color: var(--color-accent);
        padding: 2px 5px;
        border-radius: 4px;
        font-size: 0.9em;
    }
    :global(html.dark .markdown-body code:not(pre code)) {
        color: var(--color-destructive);
        background-color: var(--color-dark-hover);
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
        color: var(--color-foreground);
        margin-top: 2rem;
        border-bottom: 1px solid var(--color-border);
        padding-bottom: 0.5rem;
    }
    :global(.video-embed) {
        position: relative;
        width: 100%;
        padding-top: 56.25%;
        margin: 1.25rem 0;
        border-radius: 16px;
        overflow: hidden;
        background: #000;
    }
    :global(.video-embed iframe) {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        border: 0;
    }
    :global(html.dark .markdown-body h2) {
        color: var(--color-dark-text);
        border-bottom-color: var(--color-dark-border);
    }
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        z-index: 999;
        display: flex;
        align-items: center;
        justify-content: center;
    }
</style>
