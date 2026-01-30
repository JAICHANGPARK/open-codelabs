<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import {
        getCodelab,
        getAttendees,
        getWsUrl,
        getChatHistory,
        type Codelab,
        type Step,
        type Attendee,
        type ChatMessage,
    } from "$lib/api";
    import {
        Home,
        MessageSquare,
        Send,
        Users,
        Clock,
        CheckCircle2,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { goto } from "$app/navigation";

    let id = page.params.id as string;
    let codelab = $state<Codelab | null>(null);
    let steps = $state<Step[]>([]);
    let attendees = $state<Attendee[]>([]);
    let attendee = $state<Attendee | null>(null);
    let loading = $state(true);
    let ws = $state<WebSocket | null>(null);

    // Chat State
    let chatMessage = $state("");
    let messages = $state<
        {
            sender: string;
            text: string;
            time: string;
            self: boolean;
            type: "chat" | "dm";
        }[]
    >([]);

    onMount(async () => {
        const savedAttendee = localStorage.getItem(`attendee_${id}`);
        if (!savedAttendee) {
            goto(`/codelabs/${id}/entry`);
            return;
        }
        attendee = JSON.parse(savedAttendee);

        try {
            const [codelabData, attendeesData] = await Promise.all([
                getCodelab(id),
                getAttendees(id),
            ]);
            codelab = codelabData[0];
            steps = codelabData[1];
            attendees = attendeesData;

            await loadChatHistory();
            initWebSocket();
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    $effect(() => {
        return () => {
            if (ws) ws.close();
        };
    });

    async function loadChatHistory() {
        if (!attendee) return;
        try {
            const history = await getChatHistory(id);
            messages = history
                .filter((msg) => msg.msg_type === "chat") // Only show public chat in this view for now, or maybe all?
                .map((msg) => ({
                    sender: msg.sender_name,
                    text: msg.message,
                    time: msg.created_at
                        ? new Date(msg.created_at).toLocaleTimeString([], {
                              hour: "2-digit",
                              minute: "2-digit",
                          })
                        : "",
                    self: msg.sender_name === attendee?.name,
                    type: "chat",
                }));
            scrollToBottom();
        } catch (e) {
            console.error(e);
        }
    }

    function initWebSocket() {
        const wsUrl = getWsUrl(id);
        ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            if (attendee) {
                ws?.send(JSON.stringify({ attendee_id: attendee.id }));
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
                    scrollToBottom();
                } else if (data.type === "step_progress") {
                    attendees = attendees.map((a) =>
                        a.id === data.attendee_id
                            ? { ...a, current_step: data.step_number }
                            : a,
                    );
                    // If new attendee joins
                    if (!attendees.find((a) => a.id === data.attendee_id)) {
                        // Ideally we should refetch attendees or the event should contain name
                        refreshAttendees();
                    }
                }
            } catch (e) {
                console.error(e);
            }
        };
    }

    async function refreshAttendees() {
        try {
            attendees = await getAttendees(id);
        } catch (e) {
            console.error(e);
        }
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

    function scrollToBottom() {
        setTimeout(() => {
            const el = document.getElementById("chat-feed");
            if (el) el.scrollTop = el.scrollHeight;
        }, 50);
    }
</script>

<div class="min-h-screen bg-[#F8F9FA] dark:bg-dark-bg flex flex-col font-sans text-[#3C4043] dark:text-dark-text">
    <!-- Header -->
    <header
        class="bg-white dark:bg-dark-surface border-b border-[#E8EAED] dark:border-dark-border py-4 px-8 sticky top-0 z-30 shadow-sm"
    >
        <div class="max-w-screen-2xl mx-auto flex justify-between items-center">
            <div class="flex items-center gap-4">
                <a href="/codelabs" class="text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text" aria-label={$t("common.title")}>
                    <Home size={24} />
                </a>
                <h1 class="text-xl font-bold text-[#202124] dark:text-dark-text">
                    {codelab?.title} - {$t("live.status_title")}
                </h1>
            </div>
            <div class="flex items-center gap-2 text-sm text-[#5F6368] dark:text-dark-text-muted">
                <Users size={18} />
                <span class="font-bold">{$t("live.participants", { values: { count: attendees.length } })}</span>
            </div>
        </div>
    </header>

    <main
        class="max-w-screen-2xl mx-auto w-full p-6 grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1 items-start"
    >
        <!-- Progress Board -->
        <div
            class="lg:col-span-2 bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden min-h-[500px]"
        >
            <div class="p-6 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/50 dark:bg-dark-bg/50">
                <h2
                    class="font-bold text-lg text-[#202124] dark:text-dark-text flex items-center gap-2"
                >
                    <Clock size={20} class="text-[#4285F4]" />
                    {$t("live.realtime_progress")}
                </h2>
            </div>
            <div class="p-6">
                <div class="space-y-6">
                    {#each attendees as att}
                        <div>
                            <div class="flex justify-between text-sm mb-2">
                                <span class="font-bold text-[#3C4043] dark:text-dark-text"
                                    >{att.name}
                                    {att.id === attendee?.id
                                        ? $t("live.you")
                                        : ""}</span
                                >
                                <span class="text-[#5F6368] dark:text-dark-text-muted"
                                    >{$t("live.step_progress", { values: { current: att.current_step || 0, total: steps.length } })}</span
                                >
                            </div>
                            <div
                                class="h-2 bg-[#F1F3F4] dark:bg-dark-border rounded-full overflow-hidden"
                            >
                                <div
                                    class="h-full bg-[#34A853] transition-all duration-500 rounded-full"
                                    style="width: {steps.length > 0
                                        ? ((att.current_step || 0) /
                                              steps.length) *
                                          100
                                        : 0}%"
                                ></div>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        </div>

        <!-- Chat -->
        <div
            class="bg-white dark:bg-dark-surface rounded-2xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden h-[600px] flex flex-col"
        >
            <div class="p-4 border-b border-[#F1F3F4] dark:border-dark-border bg-[#F8F9FA]/50 dark:bg-dark-bg/50">
                <h2
                    class="font-bold text-lg text-[#202124] dark:text-dark-text flex items-center gap-2"
                >
                    <MessageSquare size={20} class="text-[#4285F4]" />
                    {$t("live.live_chat")}
                </h2>
            </div>

            <div
                id="chat-feed"
                class="flex-1 overflow-y-auto p-4 space-y-4 bg-[#F8F9FA]/30 dark:bg-dark-bg/40"
                aria-live="polite"
            >
                {#each messages as msg}
                    <div
                        class="flex flex-col {msg.self
                            ? 'items-end'
                            : 'items-start'}"
                    >
                        <span
                            class="text-[10px] text-[#5F6368] dark:text-dark-text-muted font-bold mb-1 ml-1 mr-1 uppercase tracking-tight"
                        >
                            {msg.sender}
                        </span>
                        <div
                            class="max-w-[90%] px-4 py-2.5 rounded-2xl text-sm leading-relaxed shadow-sm {msg.self
                                ? 'bg-[#4285F4] text-white rounded-tr-none'
                                : 'bg-white dark:bg-dark-bg text-[#3C4043] dark:text-dark-text rounded-tl-none border border-[#E8EAED] dark:border-dark-border'}"
                        >
                            {msg.text}
                        </div>
                    </div>
                {/each}
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
                        placeholder={$t("live.say_hello")}
                        aria-label={$t("live.say_hello")}
                        class="w-full pl-4 pr-12 py-3 bg-[#F8F9FA] dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-xl outline-none focus:border-[#4285F4] transition-all text-sm dark:text-dark-text placeholder-[#9AA0A6] dark:placeholder-dark-text-muted/60"
                    />
                    <button
                        type="submit"
                        class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-[#4285F4] hover:bg-[#4285F4] hover:text-white dark:hover:bg-[#4285F4]/20 dark:hover:text-[#8AB4F8] rounded-lg transition-all"
                        aria-label={$t("editor.send_dm")}
                    >
                        <Send size={18} />
                    </button>
                </form>
            </div>
        </div>
    </main>
</div>
