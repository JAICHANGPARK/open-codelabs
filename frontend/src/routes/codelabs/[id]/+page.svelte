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
        ASSET_URL,
        submitFeedback,
        completeCodelab,
        getQuizzes,
        submitQuiz,
        isFirebaseMode,
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
        Github,
        FileText,
        Volume2,
        Square,
        Paperclip,
        ExternalLink,
        Download,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";
    import AskGemini from "$lib/components/AskGemini.svelte";
    import { createTtsPlayer } from "$lib/tts";
    import { themeState } from "$lib/theme.svelte";

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
    let materials = $state<Material[]>([]);

    let attendee = $state<Attendee | null>(null);

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

    function handleCertificateClick(e: MouseEvent) {
        if (!canGetCertificate) {
            e.preventDefault();
            let missing = [];
            if (codelab?.require_quiz && !isQuizPassed) missing.push($t("certificate.quiz_required"));
            if (codelab?.require_feedback && !feedbackSubmitted) missing.push($t("certificate.feedback_required"));
            
            alert(`${$t("certificate.not_earned")}\n\n${$t("certificate.requirements_guide")}\n- ${missing.join("\n- ")}`);
        }
    }

    $effect(() => {
        if (attendee) {
            if (isFirebaseMode()) {
                updateAttendeeProgress(id, attendee.id, currentStepIndex + 1);
            } else if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(
                    JSON.stringify({
                        type: "step_progress",
                        attendee_id: attendee.id,
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

    // Run Prism highlighting whenever content changes
    $effect(() => {
        if (currentStepIndex >= 0 && browser) {
            // Need to wait for DOM to update
            setTimeout(() => {
                Prism.highlightAll();
            }, 0);
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

    // Selection listener
    $effect(() => {
        if (!browser) return;
        document.addEventListener("mouseup", handleSelection);
        return () => {
            document.removeEventListener("mouseup", handleSelection);
        };
    });

    let wsCleanup: any;
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
        if (isFirebaseMode()) {
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
        if (!chatMessage.trim() || !attendee) return;

        if (isFirebaseMode()) {
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
            alert($t("help.sent"));
        } catch (e) {
            alert($t("help.failed"));
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
                    attendee_id: attendee.id,
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
                attendee_id: attendee.id,
            });
            
            // Mark as completed in backend
            try {
                await completeCodelab(id, attendee.id);
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
    let progressPercent = $derived(
        steps.length > 0 ? ((currentStepIndex + 1) / steps.length) * 100 : 0,
    );
</script>

<div
    class="min-h-screen bg-white dark:bg-dark-bg flex flex-col font-sans text-[#3C4043] dark:text-dark-text selection:bg-[#4285F4]/20 selection:text-[#4285F4]"
>
    <!-- Header -->
    <header
        class="h-16 border-b border-[#E8EAED] dark:border-dark-border flex items-center justify-between px-4 lg:px-8 sticky top-0 bg-white dark:bg-dark-surface z-30 transition-colors"
    >
        <div class="flex items-center gap-4">
            <button
                onclick={() => (showSidebar = !showSidebar)}
                class="p-2 hover:bg-[#F1F3F4] dark:hover:bg-white/10 rounded-full lg:hidden transition-colors"
                aria-label="Toggle sidebar"
            >
                {#if showSidebar}<X size={20} class="dark:text-dark-text" />{:else}<Menu size={20} class="dark:text-dark-text" />{/if}
            </button>
            <div class="flex items-center gap-3">
                <div
                    class="w-8 h-8 bg-[#4285F4] rounded flex items-center justify-center text-white font-bold"
                >
                    OC
                </div>
                <h1 class="font-bold text-lg hidden sm:block text-[#5F6368] dark:text-dark-text-muted">
                    Open-Codelabs
                </h1>
            </div>
        </div>

        <div class="flex-1 max-w-2xl px-8 text-center hidden md:block">
            <h2 class="font-medium text-[#3C4043] dark:text-dark-text truncate text-base">
                {codelab?.title || "Loading..."}
            </h2>
        </div>

        <div class="flex items-center gap-2 sm:gap-4">
            <div
                class="hidden sm:flex items-center gap-2 text-[#5F6368] dark:text-dark-text-muted text-[11px] font-bold uppercase tracking-wider"
            >
                <Clock size={14} />
                <span>{$t("editor.mins_remaining", { values: { mins: steps.length * 5 } })}</span>
            </div>

            <div class="h-6 w-px bg-[#E8EAED] dark:bg-dark-border hidden md:block mx-1"></div>

            <div class="hidden md:flex items-center gap-1">
                <a
                    href="https://github.com/JAICHANGPARK/open-codelabs"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
                    title="GitHub Repository"
                    aria-label="GitHub Repository"
                >
                    <Github size={18} />
                </a>
                <a
                    href="https://jaichangpark.github.io/open-codelabs/"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
                    title="Documentation"
                    aria-label="Documentation"
                >
                    <FileText size={18} />
                </a>
            </div>

            <button
                onclick={() => (showChat = !showChat)}
                class="p-2 hover:bg-[#F1F3F4] dark:hover:bg-white/10 rounded-full relative transition-colors"
                title={$t("editor.public_chat")}
                aria-label={$t("editor.public_chat")}
                aria-expanded={showChat}
            >
                <MessageSquare
                    size={20}
                    class={showChat ? "text-[#4285F4]" : "text-[#5F6368] dark:text-dark-text-muted"}
                />
                {#if !showChat && messages.length > 0}
                    <span
                        class="absolute top-1 right-1 w-2 h-2 bg-[#EA4335] rounded-full border-2 border-white dark:border-dark-surface"
                    ></span>
                {/if}
            </button>

            <div
                class="w-8 h-8 rounded-full bg-[#E8EAED] dark:bg-white/10 flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted border-2 border-white dark:border-dark-surface shadow-sm"
                title={attendee?.name}
            >
                <User size={18} />
            </div>
        </div>
    </header>

    <!-- Progress Bar -->
    <div class="h-1 bg-[#F1F3F4] dark:bg-dark-border transition-all sticky top-16 z-30">
        <div
            class="h-full bg-[#4285F4] transition-all duration-700 ease-out {themeState.colorblindMode ? 'opacity-80' : ''}"
            style="width: {isFinished ? 100 : progressPercent}%; {themeState.colorblindMode ? 'background-image: repeating-linear-gradient(45deg, transparent, transparent 10px, rgba(255,255,255,0.3) 10px, rgba(255,255,255,0.3) 20px); border-bottom: 2px solid #000;' : ''}"
        ></div>
    </div>

    <div class="flex flex-1 relative overflow-hidden">
        <!-- Sidebar -->
        <aside
            class="fixed inset-y-0 left-0 transform {showSidebar
                ? 'translate-x-0'
                : '-translate-x-full'} lg:relative lg:translate-x-0 transition-transform duration-300 ease-in-out z-20 w-72 bg-[#F8F9FA] dark:bg-dark-surface border-r border-[#E8EAED] dark:border-dark-border overflow-y-auto pt-16 lg:pt-0"
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
                            ? 'bg-[#E8F0FE] dark:bg-[#4285F4]/20 text-[#1967D2] dark:text-[#4285F4]'
                            : 'hover:bg-[#F1F3F4] dark:hover:bg-white/5 text-[#5F6368] dark:text-dark-text-muted'}"
                    >
                        <span
                            class="text-xs font-bold mt-1 w-5 h-5 rounded-full flex items-center justify-center shrink-0 {currentStepIndex ===
                                i && !isFinished
                                ? 'bg-[#4285F4] text-white'
                                : 'bg-[#E8EAED] dark:bg-white/10 text-[#5F6368] dark:text-dark-text-muted'}">{i + 1}</span
                        >
                        <span class="text-sm font-medium leading-tight pt-1"
                            >{step.title}</span
                        >
                    </button>
                {/each}

                {#if materials.length > 0}
                    <div
                        class="mt-8 pt-8 border-t border-[#E8EAED] dark:border-dark-border px-2 pb-8"
                    >
                        <h3
                            class="text-[11px] font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-widest mb-4 px-2 flex items-center gap-2"
                        >
                            <Paperclip size={14} />
                            {$t("editor.materials_tab")}
                        </h3>
                        <div class="space-y-1">
                            {#each materials as mat}
                                <a
                                    href={mat.material_type === "link"
                                        ? mat.link_url
                                        : `${ASSET_URL}${mat.file_path}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="flex items-center gap-3 p-2.5 rounded-lg hover:bg-[#F1F3F4] dark:hover:bg-white/5 transition-all group"
                                >
                                    <div
                                        class="p-2 bg-white dark:bg-white/10 rounded-lg text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4] transition-colors shadow-sm shrink-0"
                                    >
                                        {#if mat.material_type === "link"}
                                            <ExternalLink size={16} />
                                        {:else}
                                            <Download size={16} />
                                        {/if}
                                    </div>
                                    <span
                                        class="text-sm text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#202124] dark:group-hover:text-dark-text transition-colors truncate font-medium"
                                    >
                                        {mat.title}
                                    </span>
                                </a>
                            {/each}
                        </div>
                    </div>
                {/if}
            </nav>
        </aside>

        {#if showSidebar}
            <button
                onclick={() => (showSidebar = false)}
                aria-label="Close sidebar"
                class="fixed inset-0 bg-[#3C4043]/40 dark:bg-black/60 backdrop-blur-[2px] z-10 lg:hidden transition-opacity"
                transition:fade={{ duration: 200 }}
            ></button>
        {/if}

        <!-- Content Area -->
        <main class="flex-1 overflow-y-auto p-4 sm:p-6 lg:p-12 bg-white dark:bg-dark-bg relative transition-colors">
            <div class="max-w-3xl mx-auto min-h-full">
                {#if loading}
                    <div class="space-y-6" in:fade>
                        <div
                            class="h-12 bg-[#F1F3F4] dark:bg-dark-surface rounded-md w-3/4 animate-pulse"
                        ></div>
                        <div class="space-y-3">
                            <div
                                class="h-4 bg-[#F1F3F4] dark:bg-dark-surface rounded w-full animate-pulse"
                            ></div>
                            <div
                                class="h-4 bg-[#F1F3F4] dark:bg-dark-surface rounded w-5/6 animate-pulse"
                            ></div>
                            <div
                                class="h-4 bg-[#F1F3F4] dark:bg-dark-surface rounded w-4/5 animate-pulse"
                            ></div>
                        </div>
                        <div
                            class="h-80 bg-[#F8F9FA] dark:bg-dark-surface rounded-xl w-full mt-10 animate-pulse"
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
                            class="w-24 h-24 bg-[#E6F4EA] dark:bg-green-500/10 text-[#1E8E3E] dark:text-green-400 rounded-full flex items-center justify-center mb-8"
                        >
                            <CheckCircle2 size={48} />
                        </div>
                        <h1 class="text-4xl font-extrabold text-[#202124] dark:text-dark-text mb-4">
                            {$t("feedback.done_title")}
                        </h1>
                        <p
                            class="text-[#5F6368] dark:text-dark-text-muted text-xl max-w-lg mb-12 leading-relaxed"
                        >
                            {$t("feedback.done_desc", { values: { title: codelab?.title } })}
                        </p>

                        {#if quizzes.length > 0 && !isQuizPassed}
                            <div class="max-w-2xl w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-3xl p-8 mb-12 text-left shadow-lg transition-all">
                                <div class="flex items-center gap-3 mb-8">
                                    <div class="p-2 bg-[#4285F4]/10 rounded-xl text-[#4285F4]">
                                        <Sparkles size={24} />
                                    </div>
                                    <h2 class="text-2xl font-bold text-[#202124] dark:text-dark-text">{$t("editor.quiz_tab")}</h2>
                                </div>
                                
                                <div class="space-y-10">
                                    {#each quizzes as q, i}
                                        <div class="space-y-4">
                                            <p class="font-bold text-lg text-[#3C4043] dark:text-dark-text flex gap-3">
                                                <span class="text-[#4285F4]">Q{i+1}.</span>
                                                {q.question}
                                            </p>
                                            
                                            {#if q.quiz_type === 'descriptive'}
                                                <div class="pl-8">
                                                    <textarea
                                                        bind:value={quizAnswers[i]}
                                                        disabled={quizSubmitted}
                                                        placeholder="Type your answer here..."
                                                        class="w-full p-4 rounded-2xl border-2 border-[#F1F3F4] dark:border-dark-border bg-white dark:bg-dark-surface focus:border-[#4285F4] outline-none transition-all min-h-[100px] text-sm"
                                                    ></textarea>
                                                </div>
                                            {:else}
                                                <div class="grid grid-cols-1 gap-3 pl-8">
                                                    {#each q.options as opt, oi}
                                                        <button 
                                                            onclick={() => { if(!quizSubmitted) quizAnswers[i] = oi }}
                                                            class="w-full text-left p-4 rounded-2xl border-2 transition-all flex items-center gap-4 {quizAnswers[i] === oi ? 'border-[#4285F4] bg-[#E8F0FE]/50 dark:bg-[#4285F4]/10 text-[#1967D2] dark:text-[#4285F4]' : 'border-[#F1F3F4] dark:border-dark-border hover:border-[#DADCE0] dark:hover:border-dark-border'}"
                                                            disabled={quizSubmitted}
                                                        >
                                                            <div class="w-6 h-6 rounded-full border-2 flex items-center justify-center shrink-0 {quizAnswers[i] === oi ? 'border-[#4285F4] bg-[#4285F4] text-white' : 'border-[#DADCE0] dark:border-dark-border text-transparent'}">
                                                                <Check size={14} />
                                                            </div>
                                                            <span class="font-medium">{opt}</span>
                                                        </button>
                                                    {/each}
                                                </div>
                                            {/if}

                                            {#if quizSubmitted && q.quiz_type !== 'descriptive' && quizAnswers[i] !== q.correct_answer}
                                                <p class="text-[#EA4335] text-sm font-bold pl-8 flex items-center gap-2">
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
                                        class="bg-[#4285F4] text-white px-12 py-4 rounded-full font-bold text-lg shadow-md hover:bg-[#1A73E8] disabled:opacity-50 transition-all active:scale-95 flex items-center gap-2"
                                    >
                                        <CheckCircle2 size={24} />
                                        Submit Answers
                                    </button>
                                    {#if quizSubmitted && !isQuizPassed}
                                        <p class="text-[#EA4335] font-bold">You got {quizCorrectCount} / {quizzes.length} correct. Please try again!</p>
                                        <button 
                                            onclick={() => { 
                                                quizSubmitted = false; 
                                                quizAnswers = quizzes.map(i => i.quiz_type === 'descriptive' ? "" : -1);
                                            }}
                                            class="text-[#4285F4] font-bold hover:underline"
                                        >
                                            Retry Quiz
                                        </button>
                                    {/if}
                                </div>
                            </div>
                        {/if}

                        {#if !feedbackSubmitted}
                            {#if isQuizPassed || quizzes.length === 0 || !codelab?.require_quiz}
                                <div
                                    class="max-w-md w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-6 mb-8 text-left shadow-sm transition-colors"
                                >
                                    <h3
                                        class="font-bold text-lg mb-4 text-[#202124] dark:text-dark-text"
                                    >
                                        {$t("feedback.experience_title")}
                                    </h3>

                                    <div class="mb-4">
                                        <span
                                            class="block text-sm font-bold text-[#5F6368] dark:text-dark-text-muted mb-2"
                                            >{$t("feedback.satisfaction")}</span
                                        >
                                        <div class="flex gap-2">
                                            {#each [1, 2, 3, 4, 5] as s}
                                                <button
                                                    onclick={() =>
                                                        (feedbackSatisfaction = s)}
                                                    class="p-1 rounded-lg transition-all hover:bg-yellow-50 dark:hover:bg-yellow-500/10 focus:outline-none focus:ring-2 focus:ring-yellow-400"
                                                    aria-label="Rate {s} out of 5 stars"
                                                    aria-pressed={feedbackSatisfaction >= s}
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
                                                            : "text-[#BDC1C6] dark:text-dark-text-muted/30"}
                                                        aria-hidden="true"
                                                    />
                                                </button>
                                            {/each}
                                        </div>
                                    </div>

                                    <div class="mb-4">
                                        <label
                                            for="difficulty-slider"
                                            class="block text-sm font-bold text-[#5F6368] dark:text-dark-text-muted mb-2"
                                            >{$t("feedback.difficulty")}</label
                                        >
                                        <input
                                            id="difficulty-slider"
                                            type="range"
                                            min="1"
                                            max="5"
                                            step="1"
                                            bind:value={feedbackDifficulty}
                                            class="w-full accent-[#4285F4] h-2 bg-gray-200 dark:bg-dark-border rounded-lg appearance-none cursor-pointer"
                                        />
                                        <div
                                            class="flex justify-between text-xs text-[#9AA0A6] dark:text-dark-text-muted mt-2 font-medium"
                                        >
                                            <span>{$t("feedback.too_easy")}</span>
                                            <span>{$t("feedback.just_right")}</span>
                                            <span>{$t("feedback.too_hard")}</span>
                                        </div>
                                    </div>

                                    <div class="mb-6">
                                        <label
                                            for="feedback-comments"
                                            class="block text-sm font-bold text-[#5F6368] dark:text-dark-text-muted mb-2"
                                            >{$t("feedback.comments_optional")}</label
                                        >
                                        <textarea
                                            id="feedback-comments"
                                            bind:value={feedbackComment}
                                            class="w-full bg-transparent border border-[#DADCE0] dark:border-dark-border rounded-lg p-3 text-sm text-[#3C4043] dark:text-dark-text focus:border-[#4285F4] outline-none transition-colors"
                                            rows="3"
                                            placeholder={$t("feedback.comments_placeholder")}
                                        ></textarea>
                                    </div>

                                    <button
                                        onclick={handleFeedbackSubmit}
                                        disabled={feedbackSubmitting}
                                        class="w-full bg-[#4285F4] text-white py-3 rounded-full font-bold hover:bg-[#1A73E8] disabled:opacity-50 transition-all shadow-md active:scale-95"
                                    >
                                        {feedbackSubmitting
                                            ? $t("feedback.submitting")
                                            : $t("feedback.submit")}
                                    </button>
                                </div>
                            {/if}
                        {:else}
                            <div
                                class="bg-[#E6F4EA] dark:bg-green-500/10 text-[#137333] dark:text-green-400 px-8 py-6 rounded-2xl mb-12 flex flex-col items-center gap-2 border border-[#CEEAD6] dark:border-green-500/20"
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
                                onclick={handleCertificateClick}
                                class="bg-[#4285F4] text-white hover:bg-[#1A73E8] px-8 py-3 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2 {!canGetCertificate ? 'opacity-70' : ''}"
                            >
                                <FileText size={20} />
                                {$t("feedback.get_certificate")}
                            </a>
                            <a
                                href="/codelabs/{id}/live"
                                class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border text-[#4285F4] hover:bg-[#F8F9FA] dark:hover:bg-white/5 px-8 py-3 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                            >
                                <Users size={20} />
                                {$t("feedback.view_live_status")}
                            </a>
                        </div>
                    </div>
                {:else if currentStep}
                    <div
                        class="prose dark:prose-invert max-w-none text-[#3C4043] dark:text-dark-text"
                        in:fade={{ duration: 300 }}
                    >
                        <h1
                            class="text-[32px] leading-tight font-bold text-[#202124] dark:text-dark-text border-b border-[#F1F3F4] dark:border-dark-border pb-6 mb-10 transition-colors flex items-center justify-between"
                        >
                            <span>{currentStepIndex + 1}. {currentStep.title}</span>
                            <button
                                onclick={handleTtsToggle}
                                class="p-2 rounded-full hover:bg-[#F1F3F4] dark:hover:bg-white/10 transition-all {isSpeaking ? 'text-[#4285F4] bg-[#E8F0FE] dark:bg-[#4285F4]/20' : 'text-[#5F6368] dark:text-dark-text-muted'}"
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
                    </div>
                {/if}
            </div>

            <!-- Floating Help Button -->
            {#if !isFinished && !loading}
                <button
                    onclick={handleRequestHelp}
                    disabled={helpSent}
                    class="fixed bottom-24 right-20 sm:right-24 p-3 sm:p-4 border rounded-full shadow-lg hover:shadow-xl transition-all active:scale-95 group z-20 flex items-center gap-2 {helpSent
                        ? 'bg-[#34A853] border-[#34A853] text-white cursor-not-allowed'
                        : 'bg-white dark:bg-dark-surface border-[#E8EAED] dark:border-dark-border text-[#EA4335] hover:border-[#EA4335]'}"
                >
                    <div
                        class="p-2 rounded-full transition-colors {helpSent
                            ? 'bg-white/20'
                            : 'bg-[#EA4335]/10 dark:bg-[#EA4335]/20 group-hover:bg-[#EA4335] group-hover:text-white'}"
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
                    class="fixed z-50 bg-white dark:bg-dark-surface text-[#4285F4] px-4 py-2 rounded-lg shadow-xl border border-[#D2E3FC] dark:border-[#4285F4]/30 flex items-center gap-2 font-bold text-sm animate-in fade-in zoom-in-95 duration-200 hover:bg-[#F8F9FA] dark:hover:bg-white/10 active:scale-95 cursor-pointer"
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
                    onClose={() => (showGeminiModal = false)}
                />
            {/if}
        </main>

        <!-- Chat Sidebar -->
        {#if showChat}
            <aside
                transition:fly={{ x: 320, duration: 300 }}
                class="fixed inset-y-0 right-0 z-40 w-full sm:w-80 bg-white dark:bg-dark-surface border-l border-[#E8EAED] dark:border-dark-border flex flex-col pt-16 lg:pt-0"
            >
                <div class="border-b border-[#E8EAED] dark:border-dark-border bg-[#F8F9FA] dark:bg-white/5">
                    <div class="p-4 flex items-center justify-between pb-2">
                        <h3
                            class="font-bold text-[#3C4043] dark:text-dark-text flex items-center gap-2"
                        >
                            <MessageSquare size={18} />
                            {chatTab === "public"
                                ? $t("editor.public_chat")
                                : $t("editor.direct_messages")}
                        </h3>
                        <button
                            onclick={() => (showChat = false)}
                            class="p-1 hover:bg-[#E8EAED] dark:hover:bg-white/10 rounded-full dark:text-dark-text"
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
                                ? 'text-[#4285F4] border-b-2 border-[#4285F4]'
                                : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#3C4043] dark:hover:text-dark-text'}"
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
                                ? 'text-[#4285F4] border-b-2 border-[#4285F4]'
                                : 'text-[#5F6368] dark:text-dark-text-muted hover:text-[#3C4043] dark:hover:text-dark-text'}"
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
                                    class="text-[10px] text-[#5F6368] dark:text-dark-text-muted font-bold mb-1 ml-1 mr-1 uppercase tracking-tight"
                                >
                                    {msg.sender} &bull; {msg.time}
                                </span>
                            {:else}
                                <span
                                    class="text-[10px] text-[#5F6368] dark:text-dark-text-muted font-bold mb-1 ml-1 mr-1 uppercase tracking-tight {msg.self
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
                                      ? 'bg-[#FEEFC3] dark:bg-yellow-500/20 text-[#3C4043] dark:text-yellow-200 rounded-tl-none border border-[#FAD2CF] dark:border-yellow-500/30'
                                      : 'bg-[#F1F3F4] dark:bg-dark-surface text-[#3C4043] dark:text-dark-text rounded-tl-none border border-transparent dark:border-dark-border'}"
                            >
                                {msg.text}
                            </div>
                        </div>
                    {/each}
                    {#if filteredMessages.length === 0}
                        <div
                            class="flex flex-col items-center justify-center h-full text-center text-[#9AA0A6] dark:text-dark-text-muted px-6"
                        >
                            <div
                                class="w-12 h-12 bg-[#F1F3F4] dark:bg-white/5 rounded-full flex items-center justify-center mb-4"
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

                <div class="p-4 border-t border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface">
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
                            class="w-full pl-4 pr-12 py-3 bg-[#F8F9FA] dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-xl outline-none focus:border-[#4285F4] transition-all text-sm text-[#3C4043] dark:text-dark-text"
                        />
                        <button
                            type="submit"
                            class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-[#4285F4] hover:bg-[#4285F4] hover:text-white rounded-lg transition-all"
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
        class="h-20 border-t border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface flex items-center justify-center px-4 sm:px-8 sticky bottom-0 z-30 transition-colors"
    >
        <div class="max-w-3xl w-full flex justify-between items-center">
            <button
                onclick={prevStep}
                disabled={currentStepIndex === 0 || isFinished}
                class="flex items-center gap-1 sm:gap-2 px-3 sm:px-6 py-2 rounded-full font-bold transition-all {currentStepIndex ===
                    0 || isFinished
                    ? 'text-[#DADCE0] dark:text-dark-text-muted/30 cursor-not-allowed'
                    : 'text-[#5F6368] dark:text-dark-text-muted hover:bg-[#F1F3F4] dark:hover:bg-white/10'}"
            >
                <ChevronLeft size={20} />
                <span class="hidden xs:inline">{$t("editor.back")}</span>
            </button>
            <div
                class="flex items-center text-xs sm:text-sm font-bold text-[#5F6368] dark:text-dark-text-muted bg-[#F1F3F4] dark:bg-white/10 px-3 sm:px-4 py-1.5 rounded-full"
            >
                {isFinished ? steps.length : currentStepIndex + 1} / {steps.length}
            </div>
            {#if currentStepIndex === steps.length - 1 && !isFinished}
                <button
                    onclick={finishCodelab}
                    class="bg-[#1E8E3E] hover:bg-[#178037] text-white px-6 sm:px-10 py-2.5 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                >
                    <span class="hidden xs:inline">{$t("editor.finish")}</span>
                    <span class="xs:hidden">{$t("editor.finish")}</span>
                </button>
            {:else if isFinished}
                <div class="w-[60px] sm:w-[100px]"></div>
            {:else}
                <button
                    onclick={nextStep}
                    class="bg-[#4285F4] hover:bg-[#1A73E8] text-white px-6 sm:px-10 py-2.5 rounded-full font-bold shadow-sm hover:shadow-md transition-all flex items-center gap-2"
                >
                    <span class="hidden xs:inline">{$t("editor.next")}</span>
                    <span class="xs:hidden">{$t("editor.next")}</span>
                    <ChevronRight size={20} />
                </button>
            {/if}
        </div>
    </footer>
</div>

