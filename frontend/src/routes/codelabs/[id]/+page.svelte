<script lang="ts">
    import { onMount } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { browser } from "$app/environment";
    import {
        getCodelab,
        getMaterials,
        requestHelp,
        getWsUrl,
        getChatHistory,
        getSession,
        ASSET_URL,
        submitFeedback,
        completeCodelab,
        getQuizzes,
        submitQuiz,
        isFirebaseMode,
        isServerlessMode,
        listenToWsReplacement,
        sendChatMessage,
        updateAttendeeProgress,
        type Codelab,
        type Step,
        type Attendee,
        type ChatMessage,
        type Material,
    } from "$lib/api";
    import { loadProgress, saveProgress } from "$lib/Progress";
    import { attendeeMarked as marked } from "$lib/markdown";
    import { extractPlaygrounds, type PlaygroundBlock } from "$lib/playground";
    import DOMPurify from "dompurify";
    import {
        ChevronLeft,
        ChevronRight,
        Menu,
        X,
        Clock,
        Code,
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
        Github,
        FileText,
        Volume2,
        Square,
        Paperclip,
        ExternalLink,
        Download,
        Info,
        Upload,
        Trash2,
        FileUp,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";
    import AskGemini from "$lib/components/codelabs/AskGemini.svelte";
    import PlaygroundPanel from "$lib/components/codelabs/PlaygroundPanel.svelte";
    import SubmissionPanel from "$lib/components/codelabs/SubmissionPanel.svelte";
    import { createTtsPlayer } from "$lib/tts";
    import { themeState } from "$lib/theme.svelte";
    import { submitFile, getSubmissions, deleteSubmission as apiDeleteSubmission } from "$lib/api";


    let id = page.params.id as string;
    let codelab = $state<Codelab | null>(null);
    let steps = $state<Step[]>([]);
    let loading = $state(true);
    let currentStepIndex = $state(0);
    let showSidebar = $state(true);
    let showChat = $state(false);
    let showProfile = $state(false);
    let showGuide = $state(false);
    let showPlayground = $state(false);
    let isFinished = $state(false);
    let materials = $state<Material[]>([]);

    let attendee = $state<Attendee | null>(null);

    // Submission State
    let mySubmissions = $state<any[]>([]);
    let submittingFile = $state(false);
    let totalSubmissionSize = $derived(mySubmissions.reduce((acc, s) => acc + s.file_size, 0));
    let submissionProgress = $state(0);

    // Quiz State
    let quizzes = $state<any[]>([]);
    let quizAnswers = $state<any[]>([]); // number for MC, string for Descriptive
    let quizSubmitted = $state(false);
    let quizCorrectCount = $state(0);
    let isQuizPassed = $derived(quizSubmitted && quizCorrectCount === quizzes.length);

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
    let lastStepId = $state<string | null>(null);
    let profileRef = $state<HTMLDivElement | null>(null);

    const defaultPlaygrounds: PlaygroundBlock[] = [
        { language: "dart", code: "" },
        { language: "go", code: "" },
        { language: "python", code: "" },
        { language: "jupyter", code: "" },
    ];

    // Gemini State
    let showGeminiButton = $state(false);
    let showGeminiModal = $state(false);
    let selectedContext = $state("");
    let geminiButtonPos = $state({ x: 0, y: 0 });

    // TTS State
    const tts = createTtsPlayer();
    let isSpeaking = $state(false);

    let filteredMessages = $derived(
        chatTab === "public"
            ? messages.filter((m) => m.type === "chat")
            : messages.filter((m) => m.type === "dm"),
    );

    let canGetCertificate = $derived(
        (!codelab?.require_quiz || isQuizPassed) &&
        (!codelab?.require_feedback || feedbackSubmitted)
    );

    async function handleCertificateClick(e: MouseEvent) {
        if (!canGetCertificate) {
            e.preventDefault();
            let missing = [];
            if (codelab?.require_quiz && !isQuizPassed) missing.push($t("certificate.quiz_required"));
            if (codelab?.require_feedback && !feedbackSubmitted) missing.push($t("certificate.feedback_required"));
            
            alert(`${$t("certificate.not_earned")}\n\n${$t("certificate.requirements_guide")}\n- ${missing.join("\n- ")}`);
            return;
        }

        e.preventDefault();
        if (!attendee) return;

        try {
            await completeCodelab(id);
        } catch (e) {
            console.error("Complete codelab error", e);
        }

        window.open(`/certificate/${attendee.id}`, "_blank", "noopener,noreferrer");
    }

    $effect(() => {
        if (attendee) {
            if (isServerlessMode()) {
                updateAttendeeProgress(id, attendee.id, currentStepIndex + 1);
            } else if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(
                    JSON.stringify({
                        type: "step_progress",
                        step_number: currentStepIndex + 1,
                    }),
                );
            }
        }
    });

    $effect(() => {
        if (currentStepIndex >= 0) {
            tts.stop();
            isSpeaking = false;
        }
    });

    function handleTtsToggle() {
        if (isSpeaking) {
            tts.stop();
            isSpeaking = false;
        } else if (currentStep) {
            const textToRead = `${currentStep.title}. ${currentStep.content_markdown}`;
            tts.speak(textToRead, $locale || "en");
            isSpeaking = true;
        }
    }

    // Selection listener
    $effect(() => {
        if (!browser) return;
        document.addEventListener("mouseup", handleSelection);
        return () => {
            document.removeEventListener("mouseup", handleSelection);
        };
    });

    $effect(() => {
        if (!browser || !showProfile) return;
        const handler = (event: MouseEvent) => {
            if (!profileRef) return;
            if (!profileRef.contains(event.target as Node)) {
                showProfile = false;
            }
        };
        document.addEventListener("click", handler);
        return () => document.removeEventListener("click", handler);
    });

    function formatTimestamp(value?: string | null) {
        if (!value) return "";
        const date = new Date(value);
        if (Number.isNaN(date.getTime())) return value;
        return date.toLocaleString($locale || "en");
    }

    let wsCleanup: any;
    onMount(async () => {
        // Check for registration
        const savedAttendee = localStorage.getItem(`attendee_${id}`);
        if (!savedAttendee) {
            goto(`/codelabs/${id}/entry`);
            return;
        }
        attendee = JSON.parse(savedAttendee);
        if (!isServerlessMode()) {
            try {
                const session = await getSession();
                const sessionMatches =
                    session?.role === "attendee" && session.codelab_id === id;
                if (!sessionMatches) {
                    localStorage.removeItem(`attendee_${id}`);
                    goto(`/codelabs/${id}/entry`);
                    return;
                }
            } catch (e) {
                localStorage.removeItem(`attendee_${id}`);
                goto(`/codelabs/${id}/entry`);
                return;
            }
        }

        try {
            const data = await getCodelab(id);
            codelab = data[0];
            steps = data[1];
            currentStepIndex = loadProgress(id);
            if (currentStepIndex >= steps.length) currentStepIndex = 0;

            // Show guide automatically on first visit if guide exists
            if (currentStepIndex === 0 && codelab.guide_markdown) {
                const hasSeenGuide = localStorage.getItem(`seen_guide_${id}`);
                if (!hasSeenGuide) {
                    showGuide = true;
                    localStorage.setItem(`seen_guide_${id}`, "true");
                }
            }

            // Load materials
            getMaterials(id).then((m) => {
                materials = m;
            }).catch(e => console.error("Failed to load materials", e));

            // Load Quizzes
            getQuizzes(id).then(q => {
                quizzes = q.map(i => ({
                    ...i, 
                    quiz_type: i.quiz_type || 'multiple_choice',
                    options: typeof i.options === 'string' ? JSON.parse(i.options) : i.options
                }));
                quizAnswers = quizzes.map(i => i.quiz_type === 'descriptive' ? "" : -1);
            }).catch(e => console.error("Failed to load quizzes", e));

            // Load My Submissions
            if (!isFirebaseMode()) {
                console.log("Fetching submissions for attendee:", attendee?.id);
                const allSubmissions = await getSubmissions(id);
                console.log("All submissions for this codelab:", allSubmissions);
                mySubmissions = allSubmissions.filter(s => s.attendee_id === attendee?.id);
                console.log("My filtered submissions:", mySubmissions);
            }

            await loadChatHistory();
            wsCleanup = initWebSocket();
        } catch (e: any) {
            console.error(e);
            if (e.message === 'PRIVATE_CODELAB') {
                goto(`/codelabs/${id}/entry`); // Let entry page handle private error
            }
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
                                ? `To: ${$t("common.facilitator")}`
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
        if (isServerlessMode()) {
            return listenToWsReplacement(id, (data) => {
                if (data.type === "chat") {
                    // Check if it's already in messages to avoid duplicates from history load
                    if (messages.find(m => m.text === data.message && m.sender === data.sender_name)) return;

                    messages = [
                        ...messages,
                        {
                            sender: data.sender_name,
                            text: data.message,
                            time: data.created_at?.toDate ? data.created_at.toDate().toLocaleTimeString() : new Date().toLocaleTimeString(),
                            self: data.sender_name === attendee?.name,
                            type: "chat",
                        },
                    ];
                } else if (data.type === "dm") {
                    if (data.target_id !== attendee?.id) return;
                    messages = [
                        ...messages,
                        {
                            sender: `[DM] ${data.sender_name}`,
                            text: data.message,
                            time: new Date().toLocaleTimeString(),
                            self: false,
                            type: "dm",
                        },
                    ];
                    if (chatTab !== "direct") hasNewDm = true;
                    showChat = true;
                }
            });
        }

        const wsUrl = getWsUrl(id);
        ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            if (attendee) {
                // Send initial progress
                ws?.send(
                    JSON.stringify({
                        type: "step_progress",
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
                            time: data.timestamp || new Date().toLocaleTimeString(),
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
                            time: data.timestamp || new Date().toLocaleTimeString(),
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
        if (!chatMessage.trim() || !attendee) return;

        if (isServerlessMode()) {
            sendChatMessage(id, {
                sender: attendee.name,
                message: chatMessage.trim(),
                type: "chat",
            });
            chatMessage = "";
            return;
        }

        if (!ws) return;
        const msg = {
            type: "chat",
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
            await requestHelp(id, currentStepIndex + 1);
            helpSent = true;
            setTimeout(() => (helpSent = false), 30000); // Prevent spamming
            alert($t("help.sent"));
        } catch (e) {
            console.error("Help request failed:", e);
            const errorMsg = e instanceof Error ? e.message : String(e);
            alert($t("help.failed") + "\n" + errorMsg);
        }
    }

    async function handleFileUpload(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!input.files || !input.files[0] || !attendee) return;

        const file = input.files[0];
        if (totalSubmissionSize + file.size > 10 * 1024 * 1024) {
            alert($t("submission.size_limit_exceeded"));
            return;
        }

        submittingFile = true;
        try {
            console.log("Submitting file for attendee:", attendee.id);
            const submission = await submitFile(id, attendee.id, file);
            console.log("File submitted successfully:", submission);
            mySubmissions = [...mySubmissions, submission];
        } catch (err: any) {
            alert(err.message || $t("submission_panel.upload_failed"));
        } finally {
            submittingFile = false;
            input.value = "";
        }
    }

    async function handleDeleteSubmission(submissionId: string) {
        if (!attendee || !confirm($t("common.confirm_delete"))) return;
        try {
            await apiDeleteSubmission(id, attendee.id, submissionId);
            mySubmissions = mySubmissions.filter(s => s.id !== submissionId);
        } catch (err: any) {
            alert(err.message || $t("submission_panel.delete_failed"));
        }
    }

    async function handleQuizSubmit() {
        let correct = 0;
        const submissions = quizzes.map((q, i) => {
            let is_correct = false;
            let answer = "";
            if (q.quiz_type === 'descriptive') {
                answer = quizAnswers[i] || "";
                if (answer.trim().length > 0) {
                    is_correct = true;
                    correct++;
                }
            } else {
                answer = quizAnswers[i].toString();
                if (quizAnswers[i] === q.correct_answer) {
                    is_correct = true;
                    correct++;
                }
            }
            return {
                quiz_id: q.id,
                answer: answer,
                is_correct: is_correct
            };
        });

        quizCorrectCount = correct;
        quizSubmitted = true;
        
        // Send to backend
        if (attendee) {
            try {
                await submitQuiz(id, {
                    submissions: submissions
                });
            } catch (e) {
                console.error("Failed to submit quiz results", e);
            }
        }
        
        if (correct === quizzes.length) {
            window.scrollTo({ top: 0, behavior: "smooth" });
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
            });
            
            // Mark as completed in backend
            try {
                await completeCodelab(id);
            } catch (ce) {
                console.error("Complete codelab error", ce);
            }

            feedbackSubmitted = true;
        } catch (e: any) {
            console.error("Feedback error", e);
            if (e.message === "ALREADY_SUBMITTED") {
                alert($t("feedback.already_submitted"));
                feedbackSubmitted = true; // Show submitted state
            } else {
                alert($t("feedback.failed_submit"));
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

    let renderedContent = $derived.by(() => {
        if (!currentStep) return "";
        const html = marked.parse(currentStep.content_markdown) as string;
        if (browser) {
            return DOMPurify.sanitize(html);
        }
        return html;
    });
    let playgrounds = $derived.by(() => {
        if (!currentStep) return [];
        return extractPlaygrounds(currentStep.content_markdown);
    });
    let playgroundsForPanel = $derived.by(() => {
        if (playgrounds.length > 0) return playgrounds;
        return defaultPlaygrounds;
    });
    $effect(() => {
        const stepId = currentStep?.id ?? null;
        if (!stepId || stepId === lastStepId) return;
        showPlayground = playgrounds.length > 0;
        lastStepId = stepId;
    });
    let progressPercent = $derived(
        steps.length > 0 ? ((currentStepIndex + 1) / steps.length) * 100 : 0,
    );
</script>

<a href="#main-content" class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:bg-white dark:focus:bg-dark-surface focus:text-foreground dark:focus:text-dark-text focus:p-3 focus:rounded-lg focus:shadow-lg">
    {$t("common.skip_to_main") || "Skip to main content"}
</a>

<div
    class="min-h-screen bg-background dark:bg-dark-bg flex flex-col font-sans text-foreground dark:text-dark-text selection:bg-primary/20 selection:text-primary"
>
    <!-- Header -->
    <header
        class="h-16 border-b border-border dark:border-dark-border flex items-center justify-between px-4 lg:px-8 sticky top-0 bg-white dark:bg-dark-surface z-30 transition-colors"
    >
        <div class="flex items-center gap-4">
            <button
                onclick={() => (showSidebar = !showSidebar)}
                class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full lg:hidden transition-colors"
                aria-label="Toggle sidebar"
            >
                {#if showSidebar}<X size={20} class="dark:text-dark-text" />{:else}<Menu size={20} class="dark:text-dark-text" />{/if}
            </button>
            <div class="flex items-center gap-3">
                <div
                    class="w-8 h-8 bg-primary rounded flex items-center justify-center text-primary-foreground font-bold"
                >
                    OC
                </div>
                <h1 class="font-bold text-lg hidden sm:block text-muted-foreground dark:text-dark-text-muted">
                    Open-Codelabs
                </h1>
            </div>
        </div>

        <div class="flex-1 max-w-2xl px-8 text-center hidden md:block">
            <h2 class="font-medium text-foreground dark:text-dark-text truncate text-base">
                {codelab?.title || "Loading..."}
            </h2>
        </div>

        <div class="flex items-center gap-2 sm:gap-4">
            {#if codelab?.guide_markdown}
                <button
                    onclick={() => (showGuide = !showGuide)}
                    class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full transition-all {showGuide ? 'text-primary bg-accent/80 dark:bg-primary/20' : 'text-muted-foreground dark:text-dark-text-muted'}"
                    title={$t("editor.guide_tab")}
                    aria-label={$t("editor.guide_tab")}
                >
                    <Info size={20} />
                </button>
            {/if}
            <button
                onclick={() => (showPlayground = !showPlayground)}
                class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full transition-all relative {showPlayground ? 'text-primary bg-accent/80 dark:bg-primary/20' : 'text-muted-foreground dark:text-dark-text-muted'}"
                title={showPlayground ? $t("playground.toggle_close") : $t("playground.toggle_open")}
                aria-label={showPlayground ? $t("playground.toggle_close") : $t("playground.toggle_open")}
            >
                <Code size={20} />
                {#if !showPlayground && playgrounds.length > 0}
                    <span
                        class="absolute top-2 right-2 w-2 h-2 bg-red-500 rounded-full border-2 border-white dark:border-dark-surface"
                        aria-hidden="true"
                    ></span>
                {/if}
            </button>
            <div
                class="hidden sm:flex items-center gap-2 text-muted-foreground dark:text-dark-text-muted text-[11px] font-bold uppercase tracking-wider"
            >
                <Clock size={14} />
                <span>{$t("editor.mins_remaining", { values: { mins: steps.length * 5 } })}</span>
            </div>

            <div class="h-6 w-px bg-border dark:bg-dark-border hidden md:block mx-1"></div>

            <div class="hidden md:flex items-center gap-1">
                <a
                    href="https://github.com/JAICHANGPARK/open-codelabs"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 text-muted-foreground dark:text-dark-text-muted hover:text-primary dark:hover:text-primary transition-all"
                    title="GitHub Repository"
                    aria-label="GitHub Repository"
                >
                    <Github size={18} />
                </a>
                <a
                    href="https://jaichangpark.github.io/open-codelabs/"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 text-muted-foreground dark:text-dark-text-muted hover:text-primary dark:hover:text-primary transition-all"
                    title="Documentation"
                    aria-label="Documentation"
                >
                    <FileText size={18} />
                </a>
            </div>

            <button
                onclick={() => (showChat = !showChat)}
                class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full relative transition-colors"
                title={$t("editor.public_chat")}
                aria-label={$t("editor.public_chat")}
                aria-expanded={showChat}
            >
                <MessageSquare
                    size={20}
                    class={showChat ? "text-primary" : "text-muted-foreground dark:text-dark-text-muted"}
                />
                {#if !showChat && messages.length > 0}
                    <span
                        class="absolute top-1 right-1 w-2 h-2 bg-red-500 rounded-full border-2 border-white dark:border-dark-surface"
                    ></span>
                {/if}
            </button>

            <div class="relative" bind:this={profileRef}>
                <button
                    class="w-8 h-8 rounded-full bg-accent/70 dark:bg-white/10 flex items-center justify-center text-muted-foreground dark:text-dark-text-muted border-2 border-border dark:border-dark-surface shadow-sm hover:bg-white dark:hover:bg-white/20 transition-colors"
                    title={attendee?.name || $t("profile.title")}
                    aria-label={$t("profile.title")}
                    aria-expanded={showProfile}
                    onclick={() => (showProfile = !showProfile)}
                >
                    <User size={18} />
                </button>

                {#if showProfile}
                    <div
                        class="absolute right-0 mt-3 w-72 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-2xl z-40 overflow-hidden"
                    >
                        <div class="p-4 border-b border-border dark:border-dark-border">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-full bg-primary/10 text-primary flex items-center justify-center font-bold">
                                    {attendee?.name
                                        ? attendee.name.slice(0, 2).toUpperCase()
                                        : "?"}
                                </div>
                                <div class="min-w-0">
                                    <div class="font-bold text-foreground dark:text-dark-text truncate">
                                        {attendee?.name || $t("attendee.anonymous_user")}
                                    </div>
                                    <div class="text-xs text-muted-foreground dark:text-dark-text-muted truncate">
                                        {codelab?.title || $t("common.title")}
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="p-4 space-y-3 text-sm">
                            <div class="flex items-center justify-between gap-4">
                                <span class="text-muted-foreground dark:text-dark-text-muted">
                                    {$t("profile.email")}
                                </span>
                                <span class="text-foreground dark:text-dark-text text-right truncate max-w-[160px]">
                                    {attendee?.email || $t("profile.not_available")}
                                </span>
                            </div>
                            <div class="flex items-center justify-between gap-4">
                                <span class="text-muted-foreground dark:text-dark-text-muted">
                                    {$t("profile.attendee_id")}
                                </span>
                                <span class="text-foreground dark:text-dark-text text-right truncate max-w-[160px]">
                                    {attendee?.id || "-"}
                                </span>
                            </div>
                            <div class="flex items-center justify-between gap-4">
                                <span class="text-muted-foreground dark:text-dark-text-muted">
                                    {$t("profile.registered_at")}
                                </span>
                                <span class="text-foreground dark:text-dark-text text-right truncate max-w-[160px]">
                                    {attendee?.created_at
                                        ? formatTimestamp(attendee.created_at)
                                        : $t("profile.not_available")}
                                </span>
                            </div>
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    </header>

    <!-- Progress Bar -->
    <div class="h-1 bg-accent/70 dark:bg-dark-border transition-all sticky top-16 z-30">
        <div
            class="h-full bg-primary transition-all duration-700 ease-out {themeState.isColorblind ? 'opacity-80' : ''}"
            style="width: {isFinished ? 100 : progressPercent}%; {themeState.isColorblind ? 'background-image: repeating-linear-gradient(45deg, transparent, transparent 10px, rgba(255,255,255,0.3) 10px, rgba(255,255,255,0.3) 20px); border-bottom: 2px solid #000;' : ''}"
        ></div>
    </div>

    <div class="flex flex-1 relative overflow-hidden">
        <!-- Sidebar -->
        <aside
            class="fixed inset-y-0 left-0 transform {showSidebar
                ? 'translate-x-0'
                : '-translate-x-full'} lg:relative lg:translate-x-0 transition-transform duration-300 ease-in-out z-20 w-72 bg-background dark:bg-dark-surface border-r border-border dark:border-dark-border overflow-y-auto pt-16 lg:pt-0"
        >
            <nav class="p-4 space-y-1">
                {#if codelab?.guide_markdown}
                    <button
                        onclick={() => {
                            isFinished = false;
                            showGuide = true;
                            if (window.innerWidth < 1024) showSidebar = false;
                            window.scrollTo({ top: 0, behavior: "smooth" });
                        }}
                        class="w-full text-left p-3 rounded-lg flex items-start gap-4 transition-all duration-200 {showGuide && !isFinished
                            ? 'bg-primary/10 dark:bg-primary/20 text-primary'
                            : 'hover:bg-accent/60 dark:hover:bg-white/5 text-muted-foreground dark:text-dark-text-muted'}"
                    >
                        <span
                            class="text-xs font-bold mt-1 w-5 h-5 rounded-full flex items-center justify-center shrink-0 {showGuide && !isFinished
                                ? 'bg-primary text-primary-foreground'
                                : 'bg-accent/80 dark:bg-white/10 text-muted-foreground dark:text-dark-text-muted'}"
                        >
                            <Info size={12} />
                        </span>
                        <span class="text-sm font-bold leading-tight pt-1"
                            >{$t("editor.guide_tab")}</span
                        >
                    </button>

                    <div class="h-px bg-border dark:bg-dark-border my-4 mx-2"></div>
                {/if}

                {#each steps as step, i}
                    <button
                        onclick={() => {
                            isFinished = false;
                            showGuide = false;
                            jumpToStep(i);
                        }}
                        class="w-full text-left p-3 rounded-lg flex items-start gap-4 transition-all duration-200 {currentStepIndex ===
                            i && !isFinished && !showGuide
                            ? 'bg-accent/80 dark:bg-primary/20 text-primary'
                            : 'hover:bg-accent/60 dark:hover:bg-white/5 text-muted-foreground dark:text-dark-text-muted'}"
                    >
                        <span
                            class="text-xs font-bold mt-1 w-5 h-5 rounded-full flex items-center justify-center shrink-0 {currentStepIndex ===
                                i && !isFinished
                                ? 'bg-primary text-primary-foreground'
                                : 'bg-accent/80 dark:bg-white/10 text-muted-foreground dark:text-dark-text-muted'}">{i + 1}</span
                        >
                        <span class="text-sm font-medium leading-tight pt-1"
                            >{step.title}</span
                        >
                    </button>
                {/each}

                {#if materials.length > 0}
                    <!-- ... (keep existing materials UI) -->
                    <div
                        class="mt-8 pt-8 border-t border-border dark:border-dark-border px-2"
                    >
                        <h3
                            class="text-[11px] font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-widest mb-4 px-2 flex items-center gap-2"
                        >
                            <Paperclip size={14} />
                            {$t("editor.materials_tab")}
                        </h3>
                        <!-- ... -->
                    </div>
                {/if}

                <SubmissionPanel
                    submissions={mySubmissions}
                    submitting={submittingFile}
                    totalSize={totalSubmissionSize}
                    onUpload={handleFileUpload}
                    onDelete={handleDeleteSubmission}
                />
            </nav>
        </aside>

        {#if showSidebar}
            <button
                onclick={() => (showSidebar = false)}
                aria-label="Close sidebar"
                class="fixed inset-0 bg-black/40 dark:bg-black/60 backdrop-blur-[2px] z-10 lg:hidden transition-opacity"
                transition:fade={{ duration: 200 }}
            ></button>
        {/if}

        <!-- Content Area -->
        <main id="main-content" class="flex-1 overflow-y-auto p-4 sm:p-6 lg:p-12 bg-white dark:bg-dark-bg relative transition-colors" aria-live="polite">
            <div class="max-w-3xl mx-auto min-h-full">
                {#if showGuide && codelab?.guide_markdown}
                    <div
                        class="mb-12 bg-accent/60 dark:bg-dark-surface border border-border dark:border-dark-border rounded-3xl overflow-hidden shadow-sm"
                        in:slide
                    >
                        <div class="bg-primary px-8 py-4 flex items-center justify-between text-primary-foreground">
                            <div class="flex items-center gap-3">
                                <Info size={20} />
                                <h3 class="font-bold">{$t("editor.guide_tab")}</h3>
                            </div>
                            <button
                                type="button"
                                onclick={() => (showGuide = false)}
                                class="hover:bg-white/20 p-1 rounded-full transition-colors"
                                aria-label={$t("common.close")}
                            >
                                <X size={18} />
                            </button>
                        </div>
                        <div class="p-8 prose dark:prose-invert max-w-none">
                            <div class="markdown-body">
                                {@html DOMPurify.sanitize(marked.parse(codelab.guide_markdown))}
                            </div>
                        </div>
                        <div class="bg-accent/60 dark:bg-dark-bg/50 px-8 py-4 flex justify-end">
                            <button
                                onclick={() => (showGuide = false)}
                                class="bg-primary text-primary-foreground px-6 py-2 rounded-full font-bold text-sm shadow-md hover:bg-primary/90 transition-all"
                            >
                                {$t("common.close")}
                            </button>
                        </div>
                    </div>
                {/if}

                {#if loading}
                    <div class="space-y-6" in:fade>
                        <div
                            class="h-12 bg-accent/70 dark:bg-dark-surface rounded-md w-3/4 animate-pulse"
                        ></div>
                        <div class="space-y-3">
                            <div
                                class="h-4 bg-accent/70 dark:bg-dark-surface rounded w-full animate-pulse"
                            ></div>
                            <div
                                class="h-4 bg-accent/70 dark:bg-dark-surface rounded w-5/6 animate-pulse"
                            ></div>
                            <div
                                class="h-4 bg-accent/70 dark:bg-dark-surface rounded w-4/5 animate-pulse"
                            ></div>
                        </div>
                        <div
                            class="h-80 bg-accent/60 dark:bg-dark-surface rounded-xl w-full mt-10 animate-pulse"
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
                        class="w-24 h-24 bg-emerald-50 dark:bg-green-500/10 text-emerald-600 dark:text-green-400 rounded-full flex items-center justify-center mb-8"
                        >
                            <CheckCircle2 size={48} />
                        </div>
                        <h1 class="text-4xl font-extrabold text-foreground dark:text-dark-text mb-4">
                            {$t("feedback.done_title")}
                        </h1>
                        <p
                            class="text-muted-foreground dark:text-dark-text-muted text-xl max-w-lg mb-12 leading-relaxed"
                        >
                            {$t("feedback.done_desc", { values: { title: codelab?.title } })}
                        </p>

                        {#if quizzes.length > 0 && !isQuizPassed}
                            <div class="max-w-2xl w-full bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-3xl p-8 mb-12 text-left shadow-lg transition-all">
                                <div class="flex items-center gap-3 mb-8">
                                    <div class="p-2 bg-primary/10 rounded-xl text-primary">
                                        <Sparkles size={24} />
                                    </div>
                                    <h2 class="text-2xl font-bold text-foreground dark:text-dark-text">{$t("editor.quiz_tab")}</h2>
                                </div>
                                
                                <div class="space-y-10">
                                    {#each quizzes as q, i}
                                        <div class="space-y-4">
                                            <p class="font-bold text-lg text-foreground dark:text-dark-text flex gap-3">
                                                <span class="text-primary">Q{i+1}.</span>
                                                {q.question}
                                            </p>
                                            
                                            {#if q.quiz_type === 'descriptive'}
                                                <div class="pl-8">
                                                    <textarea
                                                        bind:value={quizAnswers[i]}
                                                        disabled={quizSubmitted}
                                                        placeholder="Type your answer here..."
                                                        aria-label={q.question}
                                                        class="w-full p-4 rounded-2xl border-2 border-border dark:border-dark-border bg-white dark:bg-dark-surface focus:border-primary outline-none transition-all min-h-[100px] text-sm"
                                                    ></textarea>
                                                </div>
                                            {:else}
                                                <div class="grid grid-cols-1 gap-3 pl-8">
                                                    {#each q.options as opt, oi}
                                                        <button 
                                                            onclick={() => { if(!quizSubmitted) quizAnswers[i] = oi }}
                                                            class="w-full text-left p-4 rounded-2xl border-2 transition-all flex items-center gap-4 {quizAnswers[i] === oi ? 'border-primary bg-accent/60 dark:bg-primary/10 text-primary' : 'border-border dark:border-dark-border hover:border-border/80 dark:hover:border-dark-border'}"
                                                            disabled={quizSubmitted}
                                                        >
                                                            <div class="w-6 h-6 rounded-full border-2 flex items-center justify-center shrink-0 {quizAnswers[i] === oi ? 'border-primary bg-primary text-primary-foreground' : 'border-border dark:border-dark-border text-transparent'}">
                                                                <Check size={14} />
                                                            </div>
                                                            <span class="font-medium">{opt}</span>
                                                        </button>
                                                    {/each}
                                                </div>
                                            {/if}

                                            {#if quizSubmitted && q.quiz_type !== 'descriptive' && quizAnswers[i] !== q.correct_answer}
                                                <p class="text-red-500 text-sm font-bold pl-8 flex items-center gap-2">
                                                    <AlertCircle size={16} />
                                                    Correct answer: {q.options[q.correct_answer]}
                                                </p>
                                            {/if}
                                        </div>
                                    {/each}
                                </div>

                                <div class="mt-12 flex flex-col items-center gap-4">
                                    <button 
                                        onclick={handleQuizSubmit}
                                        disabled={quizAnswers.includes(-1) || quizSubmitted}
                                        class="bg-primary text-primary-foreground px-12 py-4 rounded-full font-bold text-lg shadow-md hover:bg-primary/90 disabled:opacity-50 transition-all active:scale-95 flex items-center gap-2"
                                    >
                                        <CheckCircle2 size={24} />
                                        Submit Answers
                                    </button>
                                    {#if quizSubmitted && !isQuizPassed}
                                        <p class="text-red-500 font-bold">You got {quizCorrectCount} / {quizzes.length} correct. Please try again!</p>
                                        <button 
                                            onclick={() => { 
                                                quizSubmitted = false; 
                                                quizAnswers = quizzes.map(i => i.quiz_type === 'descriptive' ? "" : -1);
                                            }}
                                            class="text-primary font-bold hover:underline"
                                        >
                                            Retry Quiz
                                        </button>
                                    {/if}
                                </div>
                            </div>
                        {/if}

                        {#if !feedbackSubmitted}
                            {#if isQuizPassed || quizzes.length === 0 || !codelab?.require_quiz}
                                <!-- Submission Card -->
                                <div class="max-w-md w-full bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-3xl p-8 mb-8 text-left shadow-lg transition-all">
                                    <div class="flex items-center gap-3 mb-6">
                                        <div class="p-2 bg-emerald-100/80 rounded-xl text-emerald-600">
                                            <FileUp size={24} />
                                        </div>
                                        <h3 class="text-xl font-bold text-foreground dark:text-dark-text">{$t("submission.title")}</h3>
                                    </div>
                                    
                                    <p class="text-sm text-muted-foreground dark:text-dark-text-muted mb-6">
                                        {$t("submission.description")}
                                    </p>

                                    <div class="space-y-3 mb-6">
                                        {#each mySubmissions as sub}
                                            <div class="flex items-center justify-between p-4 rounded-2xl bg-accent/60 dark:bg-white/5 border border-border dark:border-dark-border group">
                                                <div class="flex items-center gap-3 min-w-0">
                                                    <div class="p-2 bg-white dark:bg-white/10 rounded-lg shadow-sm">
                                                        <FileText size={18} class="text-primary" />
                                                    </div>
                                                    <div class="flex flex-col min-w-0">
                                                        <span class="text-sm font-medium truncate text-foreground dark:text-dark-text">{sub.file_name}</span>
                                                        <span class="text-[10px] text-muted-foreground">{(sub.file_size / 1024).toFixed(1)} KB</span>
                                                    </div>
                                                </div>
                                                <button 
                                                    type="button"
                                                    onclick={() => handleDeleteSubmission(sub.id)}
                                                    class="p-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-full transition-all"
                                                    aria-label={$t("common.delete")}
                                                >
                                                    <Trash2 size={18} />
                                                </button>
                                            </div>
                                        {/each}
                                    </div>

                                    <label class="flex flex-col items-center justify-center w-full p-8 border-2 border-dashed border-border dark:border-dark-border rounded-3xl hover:bg-accent/60 dark:hover:bg-white/5 transition-all cursor-pointer group relative overflow-hidden">
                                        {#if submittingFile}
                                            <div class="absolute inset-0 bg-white/80 dark:bg-dark-surface/80 flex flex-col items-center justify-center z-10">
                                                <div class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin mb-4"></div>
                                                <p class="text-sm font-bold text-primary">{$t("submission.uploading")}</p>
                                            </div>
                                        {/if}
                                        <div class="flex flex-col items-center justify-center">
                                            <div class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                                                <Upload size={32} class="text-primary" />
                                            </div>
                                            <p class="text-base font-bold text-foreground dark:text-dark-text mb-1">{$t("submission.upload_btn")}</p>
                                            <p class="text-xs text-muted-foreground dark:text-dark-text-muted">
                                                {(totalSubmissionSize / 1024 / 1024).toFixed(1)}MB / 10MB
                                            </p>
                                        </div>
                                        <input type="file" class="hidden" onchange={handleFileUpload} disabled={submittingFile} />
                                    </label>
                                </div>

                                <div
                                    class="max-w-md w-full bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-xl p-6 mb-8 text-left shadow-sm transition-colors"
                                >
                                    <h3
                                        class="font-bold text-lg mb-4 text-foreground dark:text-dark-text"
                                    >
                                        {$t("feedback.experience_title")}
                                    </h3>

                                    <div class="mb-4">
                                        <span
                                            class="block text-sm font-bold text-muted-foreground dark:text-dark-text-muted mb-2"
                                            >{$t("feedback.satisfaction")}</span
                                        >
                                        <div class="flex gap-2">
                                            {#each [1, 2, 3, 4, 5] as s}
                                                <button
                                                    onclick={() =>
                                                        (feedbackSatisfaction = s)}
                                                    class="p-1 rounded-lg transition-all hover:bg-yellow-50 dark:hover:bg-yellow-500/10 focus:outline-none focus:ring-2 focus:ring-yellow-400"
                                                    aria-label={$t("feedback.satisfaction") + " " + s + "/5"}
                                                    aria-pressed={feedbackSatisfaction >= s}
                                                >
                                                    <Star
                                                        size={28}
                                                        fill={feedbackSatisfaction >=
                                                        s
                                                            ? "currentColor"
                                                            : "none"}
                                                        class={feedbackSatisfaction >=
                                                        s
                                                            ? "text-amber-500"
                                                            : "text-muted-foreground/50 dark:text-dark-text-muted/30"}
                                                        aria-hidden="true"
                                                    />
                                                </button>
                                            {/each}
                                        </div>
                                    </div>

                                    <div class="mb-4">
                                        <label
                                            for="difficulty-slider"
                                            class="block text-sm font-bold text-muted-foreground dark:text-dark-text-muted mb-2"
                                            >{$t("feedback.difficulty")}</label
                                        >
                                        <input
                                            id="difficulty-slider"
                                            type="range"
                                            min="1"
                                            max="5"
                                            step="1"
                                            bind:value={feedbackDifficulty}
                                            class="w-full accent-[var(--primary)] h-2 bg-gray-200 dark:bg-dark-border rounded-lg appearance-none cursor-pointer"
                                        />
                                        <div
                                            class="flex justify-between text-xs text-muted-foreground dark:text-dark-text-muted mt-2 font-medium"
                                        >
                                            <span>{$t("feedback.too_easy")}</span>
                                            <span>{$t("feedback.just_right")}</span>
                                            <span>{$t("feedback.too_hard")}</span>
                                        </div>
                                    </div>

                                    <div class="mb-6">
                                        <label
                                            for="feedback-comments"
                                            class="block text-sm font-bold text-muted-foreground dark:text-dark-text-muted mb-2"
                                            >{$t("feedback.comments_optional")}</label
                                        >
                                        <textarea
                                            id="feedback-comments"
                                            bind:value={feedbackComment}
                                            class="w-full bg-transparent border border-border dark:border-dark-border rounded-lg p-3 text-sm text-foreground dark:text-dark-text focus:border-primary outline-none transition-colors"
                                            rows="3"
                                            placeholder={$t("feedback.comments_placeholder")}
                                        ></textarea>
                                    </div>

                                    <button
                                        onclick={handleFeedbackSubmit}
                                        disabled={feedbackSubmitting}
                                        class="w-full bg-primary text-primary-foreground py-3 rounded-full font-bold hover:bg-primary/90 disabled:opacity-50 transition-all shadow-md active:scale-95"
                                    >
                                        {feedbackSubmitting
                                            ? $t("feedback.submitting")
                                            : $t("feedback.submit")}
                                    </button>
                                </div>
                            {/if}
                        {:else}
                            <div
                                class="bg-emerald-50 dark:bg-green-500/10 text-emerald-700 dark:text-green-400 px-8 py-6 rounded-2xl mb-12 flex flex-col items-center gap-2 border border-emerald-200 dark:border-green-500/20"
                            >
                                <CheckCircle2 size={32} />
                                <span class="font-bold text-lg"
                                    >{$t("feedback.thanks")}</span
                                >
                            </div>
                        {/if}

                        <div class="flex flex-wrap justify-center gap-4">
                            <a
                                href="/certificate/{attendee?.id}"
                                target="_blank"
                                rel="noopener noreferrer"
                                onclick={handleCertificateClick}
                                class="bg-primary text-primary-foreground hover:bg-primary/90 px-8 py-3 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2 {!canGetCertificate ? 'opacity-70' : ''}"
                            >
                                <FileText size={20} />
                                {$t("feedback.get_certificate")}
                            </a>
                            <a
                                href="/codelabs/{id}/live"
                                class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border text-primary hover:bg-accent/60 dark:hover:bg-white/5 px-8 py-3 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                            >
                                <Users size={20} />
                                {$t("feedback.view_live_status")}
                            </a>
                        </div>
                    </div>
                {:else if currentStep}
                    <div
                        class="prose dark:prose-invert max-w-none text-foreground dark:text-dark-text"
                        in:fade={{ duration: 300 }}
                    >
                        <h1
                            class="text-[32px] leading-tight font-bold text-foreground dark:text-dark-text border-b border-border dark:border-dark-border pb-6 mb-10 transition-colors flex items-center justify-between"
                        >
                            <span>{currentStepIndex + 1}. {currentStep.title}</span>
                            <button
                                onclick={handleTtsToggle}
                                class="p-2 rounded-full hover:bg-accent/60 dark:hover:bg-white/10 transition-all {isSpeaking ? 'text-primary bg-accent/80 dark:bg-primary/20' : 'text-muted-foreground dark:text-dark-text-muted'}"
                                title={isSpeaking ? $t("common.tts_stop") : $t("common.tts_read")}
                                aria-label={isSpeaking ? $t("common.tts_stop") : $t("common.tts_read")}
                            >
                                {#if isSpeaking}
                                    <Square size={20} fill="currentColor" />
                                {:else}
                                    <Volume2 size={20} />
                                {/if}
                            </button>
                        </h1>
                        <div class="markdown-body">
                            {@html renderedContent}
                        </div>
                        {#if showPlayground}
                            <PlaygroundPanel playgrounds={playgroundsForPanel} />
                        {/if}
                    </div>
                {/if}
            </div>

            <!-- Floating Help Button -->
            {#if !isFinished && !loading}
                <button
                    onclick={handleRequestHelp}
                    disabled={helpSent}
                    class="fixed bottom-24 right-20 sm:right-24 p-3 sm:p-4 border rounded-full shadow-lg hover:shadow-xl transition-all active:scale-95 group z-20 flex items-center gap-2 {helpSent
                        ? 'bg-emerald-500 border-emerald-500 text-white cursor-not-allowed'
                        : 'bg-white dark:bg-dark-surface border-border dark:border-dark-border text-red-500 hover:border-red-500'}"
                >
                    <div
                        class="p-2 rounded-full transition-colors {helpSent
                            ? 'bg-white/20'
                            : 'bg-red-500/10 dark:bg-red-500/20 group-hover:bg-red-500 group-hover:text-white'}"
                    >
                        {#if helpSent}
                            <Check size={20} class="sm:w-6 sm:h-6" />
                        {:else}
                            <HelpCircle size={20} class="sm:w-6 sm:h-6" />
                        {/if}
                    </div>
                    {#if helpSent}
                        <span class="pr-1 sm:pr-2 text-xs sm:text-sm font-bold animate-pulse"
                            >{$t("help.requested")} ✓</span
                        >
                    {:else}
                        <span class="pr-1 sm:pr-2 text-xs sm:text-sm font-bold">{$t("help.request")}</span>
                    {/if}
                </button>
            {/if}

            <!-- Gemini Context Menu -->
            {#if showGeminiButton}
                <button
                    style="top: {geminiButtonPos.y}px; left: {geminiButtonPos.x}px;"
                    class="fixed z-50 bg-white dark:bg-dark-surface text-primary px-4 py-2 rounded-lg shadow-xl border border-border dark:border-dark-border flex items-center gap-2 font-bold text-sm animate-in fade-in zoom-in-95 duration-200 hover:bg-accent/60 dark:hover:bg-white/10 active:scale-95 cursor-pointer"
                    onmousedown={(e) => {
                        e.preventDefault();
                        showGeminiModal = true;
                        showGeminiButton = false;
                    }}
                >
                    <span class="hidden sm:inline"><Sparkles size={16} /></span>
                    Ask Gemini
                </button>
            {/if}

            {#if showGeminiModal}
                <AskGemini
                    context={selectedContext}
                    codelabId={id}
                    stepNumber={currentStepIndex + 1}
                    onClose={() => (showGeminiModal = false)}
                />
            {/if}
        </main>

        <!-- Chat Sidebar -->
        {#if showChat}
            <aside
                transition:fly={{ x: 320, duration: 300 }}
                class="fixed inset-y-0 right-0 z-40 w-full sm:w-80 bg-white dark:bg-dark-surface border-l border-border dark:border-dark-border flex flex-col pt-16 lg:pt-0"
            >
                <div class="border-b border-border dark:border-dark-border bg-accent/60 dark:bg-white/5">
                    <div class="p-4 flex items-center justify-between pb-2">
                        <h3
                            class="font-bold text-foreground dark:text-dark-text flex items-center gap-2"
                        >
                            <MessageSquare size={18} />
                            {chatTab === "public"
                                ? $t("editor.public_chat")
                                : $t("editor.direct_messages")}
                        </h3>
                        <button
                            onclick={() => (showChat = false)}
                            class="p-1 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full dark:text-dark-text"
                            aria-label="Close chat"
                        >
                            <X size={18} />
                        </button>
                    </div>

                    <div class="flex px-4 pb-2 gap-4" role="tablist">
                        <button
                            onclick={() => (chatTab = "public")}
                            role="tab"
                            aria-selected={chatTab === 'public'}
                            class="pb-2 text-sm font-bold transition-all relative {chatTab ===
                            'public'
                                ? 'text-primary border-b-2 border-primary'
                                : 'text-muted-foreground dark:text-dark-text-muted hover:text-foreground dark:hover:text-dark-text'}"
                        >
                            {$t("editor.public_chat")}
                        </button>
                        <button
                            onclick={() => {
                                chatTab = "direct";
                                hasNewDm = false;
                            }}
                            role="tab"
                            aria-selected={chatTab === 'direct'}
                            class="pb-2 text-sm font-bold transition-all relative {chatTab ===
                            'direct'
                                ? 'text-primary border-b-2 border-primary'
                                : 'text-muted-foreground dark:text-dark-text-muted hover:text-foreground dark:hover:text-dark-text'}"
                        >
                            {$t("editor.direct_messages")}
                            {#if hasNewDm}
                                <span
                                    class="absolute -top-1 -right-2 w-2 h-2 bg-red-500 rounded-full border border-white dark:border-dark-surface"
                                    aria-label="New message"
                                ></span>
                            {/if}
                        </button>
                    </div>
                </div>

                <div
                    id="chat-messages"
                    class="flex-1 overflow-y-auto p-4 space-y-4 bg-white dark:bg-dark-bg/50"
                    aria-live="polite"
                >
                    {#each filteredMessages as msg}
                        <div
                            class="flex flex-col {msg.self
                                ? 'items-end'
                                : 'items-start'}"
                        >
                            {#if chatTab === "public"}
                                <span
                                    class="text-[10px] text-muted-foreground dark:text-dark-text-muted font-bold mb-1 ml-1 mr-1 uppercase tracking-tight"
                                >
                                    {msg.sender} &bull; {msg.time}
                                </span>
                            {:else}
                                <span
                                    class="text-[10px] text-muted-foreground dark:text-dark-text-muted font-bold mb-1 ml-1 mr-1 uppercase tracking-tight {msg.self
                                        ? ''
                                        : 'text-red-600'}"
                                >
                                    {msg.sender} &bull; {msg.time}
                                </span>
                            {/if}
                            <div
                                class="max-w-[90%] px-4 py-2.5 rounded-2xl text-sm leading-relaxed shadow-sm {msg.self
                                    ? 'bg-primary text-primary-foreground rounded-tr-none'
                                    : msg.type === 'dm'
                                      ? 'bg-amber-100/80 dark:bg-yellow-500/20 text-foreground dark:text-yellow-200 rounded-tl-none border border-amber-200 dark:border-yellow-500/30'
                                      : 'bg-accent/70 dark:bg-dark-surface text-foreground dark:text-dark-text rounded-tl-none border border-transparent dark:border-dark-border'}"
                            >
                                {msg.text}
                            </div>
                        </div>
                    {/each}
                    {#if filteredMessages.length === 0}
                        <div
                            class="flex flex-col items-center justify-center h-full text-center text-muted-foreground dark:text-dark-text-muted px-6"
                        >
                            <div
                                class="w-12 h-12 bg-accent/60 dark:bg-white/5 rounded-full flex items-center justify-center mb-4"
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

                <div class="p-4 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface">
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
                            aria-label="Chat message"
                        class="w-full pl-4 pr-12 py-3 bg-background dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary transition-all text-sm text-foreground dark:text-dark-text"
                    />
                    <button
                        type="submit"
                        class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-primary hover:bg-primary hover:text-primary-foreground rounded-lg transition-all"
                        aria-label="Send message"
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
        class="h-20 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface flex items-center justify-center px-4 sm:px-8 sticky bottom-0 z-30 transition-colors"
    >
        <div class="max-w-3xl w-full flex justify-between items-center">
            <button
                onclick={prevStep}
                disabled={currentStepIndex === 0 || isFinished}
                class="flex items-center gap-1 sm:gap-2 px-3 sm:px-6 py-2 rounded-full font-bold transition-all {currentStepIndex ===
                    0 || isFinished
                    ? 'text-muted-foreground/40 dark:text-dark-text-muted/30 cursor-not-allowed'
                    : 'text-muted-foreground dark:text-dark-text-muted hover:bg-accent/60 dark:hover:bg-white/10'}"
            >
                <ChevronLeft size={20} />
                <span class="hidden xs:inline">{$t("editor.back")}</span>
            </button>
            <div
                class="flex items-center text-xs sm:text-sm font-bold text-muted-foreground dark:text-dark-text-muted bg-accent/60 dark:bg-white/10 px-3 sm:px-4 py-1.5 rounded-full"
            >
                {isFinished ? steps.length : currentStepIndex + 1} / {steps.length}
            </div>
            {#if currentStepIndex === steps.length - 1 && !isFinished}
                <button
                    onclick={finishCodelab}
                    class="bg-emerald-600 hover:bg-emerald-700 text-white px-6 sm:px-10 py-2.5 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                >
                    <span class="hidden xs:inline">{$t("editor.finish")}</span>
                    <span class="xs:hidden">{$t("editor.finish")}</span>
                </button>
            {:else if isFinished}
                <div class="w-[60px] sm:w-[100px]"></div>
            {:else}
                <button
                    onclick={nextStep}
                    class="bg-primary hover:bg-primary/90 text-primary-foreground px-6 sm:px-10 py-2.5 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                >
                    <span class="hidden xs:inline">{$t("editor.next")}</span>
                    <span class="xs:hidden">{$t("editor.next")}</span>
                    <ChevronRight size={20} />
                </button>
            {/if}
        </div>
    </footer>
</div>
