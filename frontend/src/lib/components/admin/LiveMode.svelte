<script lang="ts">
    import {
        Users,
        MessageSquare,
        Bell,
        Send,
        X,
        Image,
        ChevronLeft
    } from "lucide-svelte";
    import { slide } from "svelte/transition";
    import { t } from "svelte-i18n";
    import { browser } from "$app/environment";
    import { ASSET_URL } from "$lib/api";
    import type { Attendee, HelpRequest } from "$lib/api";

    let {
        attendees,
        helpRequests,
        totalSteps = 0,
        chatTab = $bindable(),
        dmTarget = $bindable(),
        dmMessage = $bindable(),
        chatMessage = $bindable(),
        messages,
        handleResolveHelp,
        sendChat,
        sendDM,
        attachImage
    } = $props<{
        attendees: Attendee[];
        helpRequests: HelpRequest[];
        totalSteps: number;
        chatTab: "public" | "direct";
        dmTarget: Attendee | null;
        dmMessage: string;
        chatMessage: string;
        messages: any[];
        handleResolveHelp: (id: string) => void;
        sendChat: () => void;
        sendDM: () => void;
        attachImage: (file: File) => void;
    }>();

    let imageInput = $state<HTMLInputElement | null>(null);
    let chatImageLightboxUrl = $state<string | null>(null);

    const assetBaseUrl =
        ASSET_URL ||
        (browser
            ? `${window.location.protocol}//${window.location.hostname}:8080`
            : "");

    // Track which attendees have unread DMs
    let unreadAttendees = $state<Set<string>>(new Set());
    // Track the currently selected attendee for DM view
    let selectedDmAttendee = $state<Attendee | null>(null);

    // Get unique attendees who have DMs
    let dmAttendees = $derived(
        Array.from(new Set(
            messages
                .filter((m: any) => m.type === "dm" && m.senderId)
                .map((m: any) => m.senderId as string)
        ))
        .map((id: any) => attendees.find((a: Attendee) => a.id === id))
        .filter(Boolean) as Attendee[]
    );

    // Filter messages for the selected attendee
    let filteredDmMessages = $derived(
        selectedDmAttendee
            ? messages.filter((m: any) =>
                m.type === "dm" &&
                (m.senderId === selectedDmAttendee?.id ||
                 (m.self && m.senderId === selectedDmAttendee?.id))
            )
            : []
    );

    // Track previous dmTarget to detect changes
    let prevDmTargetId = $state<string | null>(null);

    // Sync selectedDmAttendee when dmTarget changes from outside
    $effect(() => {
        const currentDmTargetId = dmTarget?.id ?? null;
        if (currentDmTargetId !== prevDmTargetId) {
            prevDmTargetId = currentDmTargetId;
            if (dmTarget && dmTarget.id !== selectedDmAttendee?.id) {
                selectedDmAttendee = dmTarget;
            }
        }
    });

    // Mark attendee as read when selected
    function selectAttendee(attendee: Attendee) {
        selectedDmAttendee = attendee;
        dmTarget = attendee;
        prevDmTargetId = attendee.id;
        unreadAttendees.delete(attendee.id);
        unreadAttendees = new Set(unreadAttendees);
    }

    // Track last processed message to avoid re-processing
    let lastProcessedMsgId = $state<string | null>(null);

    // Mark new DMs as unread
    $effect(() => {
        const lastMessage = messages[messages.length - 1];
        if (lastMessage && lastMessage.time !== lastProcessedMsgId) {
            lastProcessedMsgId = lastMessage.time;
            if (lastMessage.type === "dm" && lastMessage.senderId && !lastMessage.self) {
                if (selectedDmAttendee?.id !== lastMessage.senderId) {
                    unreadAttendees.add(lastMessage.senderId);
                    unreadAttendees = new Set(unreadAttendees);
                }
            }
        }
    });

    function getImageUrl(text: string) {
        const mdMatch = text.match(/!\[[^\]]*]\(([^)]+)\)/);
        if (mdMatch?.[1]) {
            const url = mdMatch[1];
            return url.startsWith("/uploads/") ? `${assetBaseUrl}${url}` : url;
        }
        const urlMatch = text.match(
            /(https?:\/\/[^\s]+?\.(png|jpe?g|gif|webp))$/i,
        );
        if (urlMatch?.[1]) return urlMatch[1];
        if (text.startsWith("/uploads/")) return `${assetBaseUrl}${text}`;
        return "";
    }

    function openChatImage(url: string) {
        if (!url) return;
        chatImageLightboxUrl = url;
    }

    function closeChatImage() {
        chatImageLightboxUrl = null;
    }

    function handleChatPaste(event: ClipboardEvent) {
        const items = event.clipboardData?.items;
        if (!items) return;

        // Find the first image item
        let imageFile: File | null = null;
        for (const item of Array.from(items)) {
            if (item.type.startsWith("image/")) {
                const file = item.getAsFile();
                if (file) {
                    imageFile = file;
                    break;
                }
            }
        }

        // If an image was found, prevent default paste and upload only the first image
        if (imageFile) {
            event.preventDefault();
            attachImage(imageFile);
        }
    }
</script>

<div class="grid grid-cols-1 xl:grid-cols-2 gap-6 sm:gap-8 h-full">
    <!-- Left: Activity & Help -->
    <div class="space-y-6 flex flex-col h-full min-w-0">
        <div class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col">
            <div class="p-4 bg-red-50 dark:bg-red-500/10 border-b border-red-100 dark:border-red-500/20 flex items-center gap-2">
                <Bell size={18} class="text-red-500" />
                <h3 class="font-bold text-red-500">
                    {$t("help.request")} ({helpRequests.length})
                </h3>
            </div>
            <div class="p-4 space-y-3 max-h-60 overflow-y-auto">
                {#each helpRequests as hr}
                    <div class="p-3 bg-red-50/50 dark:bg-red-500/5 rounded-xl border border-red-100 dark:border-red-500/10 flex justify-between items-center" in:slide>
                        <div>
                            <p class="font-bold text-foreground dark:text-dark-text text-sm">
                                {hr.attendee_name}
                            </p>
                            <p class="text-xs text-red-500">
                                {$t("editor.stuck_on_step", { values: { step: hr.step_number } })}
                            </p>
                        </div>
                        <button
                            onclick={() => handleResolveHelp(hr.id)}
                            class="text-xs font-bold text-white bg-red-500 px-3 py-1.5 rounded-full hover:bg-red-600 transition-colors shadow-sm"
                        >
                            {$t("editor.resolve")}
                        </button>
                    </div>
                {:else}
                    <p class="text-center py-6 text-muted-foreground/80 dark:text-dark-text-muted text-sm">
                        No pending help requests
                    </p>
                {/each}
            </div>
        </div>

        <div class="flex-1 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col min-h-[300px] lg:min-h-[400px]">
            <div class="p-4 bg-muted dark:bg-white/5 border-b border-border dark:border-dark-border flex items-center gap-2">
                <Users size={18} class="text-primary" />
                <h3 class="font-bold text-foreground dark:text-dark-text">
                    {$t("common.attendee")} ({attendees.length})
                </h3>
            </div>
            <div class="p-4 space-y-2 overflow-y-auto">
                {#each attendees as attendee}
                    <div class="flex items-center justify-between p-2 hover:bg-accent/60 dark:hover:bg-white/5 rounded-lg transition-colors group">
                        <div class="flex items-center gap-3">
                            <div class="w-8 h-8 rounded-full bg-border dark:bg-white/10 flex items-center justify-center text-muted-foreground dark:text-dark-text-muted text-xs font-bold uppercase">
                                {attendee.name.charAt(0)}
                            </div>
                            <div>
                                <p class="text-sm font-bold text-foreground dark:text-dark-text">
                                    {attendee.name}
                                </p>
                                <p class="text-[10px] text-muted-foreground/80 dark:text-dark-text-muted">
                                    {$t("submission_panel.attendee_code")}: {attendee.code}
                                </p>
                                <p class="text-[10px] text-muted-foreground dark:text-dark-text-muted">
                                    {$t("live.step_progress", { values: { current: attendee.current_step || 0, total: totalSteps } })}
                                </p>
                            </div>
                        </div>
                        <button
                            type="button"
                            onclick={() => {
                                dmTarget = attendee;
                                chatTab = "direct";
                            }}
                            class="p-2 text-primary hover:bg-accent/70 dark:hover:bg-primary/10 rounded-lg opacity-0 lg:opacity-0 group-hover:opacity-100 transition-all"
                            title={$t("editor.send_dm")}
                            aria-label={$t("editor.send_dm")}
                        >
                            <MessageSquare size={16} />
                        </button>
                    </div>
                {/each}
            </div>
        </div>
    </div>

    <!-- Right: Live Chat -->
    <div class="bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col h-full min-h-[500px] lg:min-h-[600px]">
        <div class="flex border-b border-border dark:border-dark-border">
            <button
                onclick={() => {
                    chatTab = "public";
                    selectedDmAttendee = null;
                }}
                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab === 'public'
                    ? 'text-primary border-b-2 border-primary bg-muted dark:bg-white/5'
                    : 'text-muted-foreground dark:text-dark-text-muted hover:bg-accent/60 dark:hover:bg-white/5'}"
            >
                <Users size={16} /> Public Chat
            </button>
            <button
                onclick={() => (chatTab = "direct")}
                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab === 'direct'
                    ? 'text-primary border-b-2 border-primary bg-muted dark:bg-white/5'
                    : 'text-muted-foreground dark:text-dark-text-muted hover:bg-accent/60 dark:hover:bg-white/5'}"
            >
                <MessageSquare size={16} /> Direct Messages
                {#if unreadAttendees.size > 0}
                    <span class="bg-red-500 text-white text-[10px] px-1.5 py-0.5 rounded-full">
                        {unreadAttendees.size}
                    </span>
                {/if}
            </button>
        </div>

        {#if chatTab === "public"}
            <!-- Public Chat View -->
            <div class="flex-1 p-4 space-y-4 overflow-y-auto bg-muted dark:bg-dark-bg/50" id="chat-messages">
                {#each messages.filter((m: any) => m.type === "chat") as msg}
                    <div class="flex flex-col {msg.self ? 'items-end' : 'items-start'}">
                        <span class="text-[10px] text-muted-foreground dark:text-dark-text-muted font-bold mb-1 mx-1 uppercase">
                            {msg.sender} &bull; {msg.time}
                        </span>
                        <div class="max-w-[85%] p-3 rounded-2xl text-sm shadow-sm {msg.self
                            ? 'bg-primary text-white rounded-tr-none'
                            : 'bg-white dark:bg-dark-surface text-foreground dark:text-dark-text rounded-tl-none'}">
                            {#if getImageUrl(msg.text)}
                                <img
                                    src={getImageUrl(msg.text)}
                                    alt="chat image"
                                    class="max-w-full rounded-lg border border-white/20 cursor-zoom-in"
                                    onclick={() => openChatImage(getImageUrl(msg.text))}
                                />
                            {:else}
                                {msg.text}
                            {/if}
                        </div>
                    </div>
                {:else}
                    <div class="h-full flex flex-col items-center justify-center text-muted-foreground/60 space-y-2 opacity-60">
                        <MessageSquare size={48} strokeWidth={1} />
                        <p class="text-sm font-medium">{$t("editor.no_messages")}</p>
                    </div>
                {/each}
            </div>

            <div class="p-4 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface">
                <form onsubmit={(e) => { e.preventDefault(); sendChat(); }}>
                    <div class="relative flex items-center gap-2">
                        <input
                            type="text"
                            bind:value={chatMessage}
                            placeholder="Send a message to everyone..."
                            aria-label={$t("editor.chat_placeholder")}
                            onpaste={handleChatPaste}
                            class="flex-1 pl-4 pr-12 py-3 bg-muted dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary text-sm text-foreground dark:text-dark-text"
                        />
                        <input
                            type="file"
                            accept="image/*"
                            bind:this={imageInput}
                            onchange={(e) => {
                                const input = e.currentTarget as HTMLInputElement;
                                const file = input.files?.[0];
                                if (file) attachImage(file);
                                input.value = "";
                            }}
                            class="hidden"
                        />
                        <button
                            type="button"
                            class="absolute right-10 top-1/2 -translate-y-1/2 p-2 text-muted-foreground hover:text-primary hover:bg-accent/70 dark:hover:bg-primary/10 rounded-lg transition-all"
                            onclick={() => imageInput?.click()}
                            aria-label={$t("common.upload")}
                            title={$t("common.upload")}
                        >
                            <Image size={18} />
                        </button>
                        <button
                            type="submit"
                            class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-primary hover:bg-accent/70 dark:hover:bg-primary/10 rounded-lg transition-all"
                            aria-label={$t("editor.send_dm")}
                        >
                            <Send size={18} />
                        </button>
                    </div>
                </form>
            </div>
        {:else}
            <!-- Direct Messages View -->
            {#if !selectedDmAttendee}
                <!-- Attendee List for DMs -->
                <div class="flex-1 overflow-y-auto p-4">
                    {#if dmAttendees.length > 0}
                        <div class="space-y-2">
                            {#each dmAttendees as attendee}
                                <button
                                    onclick={() => selectAttendee(attendee)}
                                    class="w-full flex items-center justify-between p-3 hover:bg-accent/60 dark:hover:bg-white/5 rounded-lg transition-colors text-left"
                                >
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 rounded-full bg-primary/10 dark:bg-primary/20 flex items-center justify-center text-primary text-sm font-bold uppercase">
                                            {attendee.name.charAt(0)}
                                        </div>
                                        <div>
                                            <p class="text-sm font-bold text-foreground dark:text-dark-text">
                                                {attendee.name}
                                            </p>
                                            <p class="text-[10px] text-muted-foreground dark:text-dark-text-muted">
                                                {$t("submission_panel.attendee_code")}: {attendee.code}
                                            </p>
                                        </div>
                                    </div>
                                    {#if unreadAttendees.has(attendee.id)}
                                        <span class="bg-red-500 text-white text-[10px] px-2 py-0.5 rounded-full">
                                            New
                                        </span>
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    {:else}
                        <div class="h-full flex flex-col items-center justify-center text-muted-foreground/60 space-y-2 opacity-60">
                            <MessageSquare size={48} strokeWidth={1} />
                            <p class="text-sm font-medium">No direct messages yet</p>
                            <p class="text-xs">Select an attendee from the list to start a conversation</p>
                        </div>
                    {/if}
                </div>
            {:else}
                <!-- DM Conversation View -->
                <div class="flex flex-col h-full">
                    <div class="p-3 border-b border-border dark:border-dark-border flex items-center gap-3 bg-muted/50 dark:bg-white/5">
                        <button
                            onclick={() => {
                                selectedDmAttendee = null;
                                dmTarget = null;
                            }}
                            class="p-2 hover:bg-accent/70 dark:hover:bg-white/10 rounded-lg transition-colors"
                        >
                            <ChevronLeft size={18} />
                        </button>
                        <div class="w-8 h-8 rounded-full bg-primary/10 dark:bg-primary/20 flex items-center justify-center text-primary text-xs font-bold uppercase">
                            {selectedDmAttendee.name.charAt(0)}
                        </div>
                        <div class="flex-1 min-w-0">
                            <p class="text-sm font-bold text-foreground dark:text-dark-text truncate">
                                {selectedDmAttendee.name}
                            </p>
                            <p class="text-[10px] text-muted-foreground dark:text-dark-text-muted">
                                {$t("submission_panel.attendee_code")}: {selectedDmAttendee.code}
                            </p>
                        </div>
                    </div>

                    <div class="flex-1 p-4 space-y-4 overflow-y-auto bg-muted dark:bg-dark-bg/50">
                        {#each filteredDmMessages as msg}
                            <div class="flex flex-col {msg.self ? 'items-end' : 'items-start'}">
                                <span class="text-[10px] text-muted-foreground dark:text-dark-text-muted font-bold mb-1 mx-1 uppercase">
                                    {msg.sender} &bull; {msg.time}
                                </span>
                                <div class="max-w-[85%] p-3 rounded-2xl text-sm shadow-sm {msg.self
                                    ? 'bg-primary text-white rounded-tr-none'
                                    : 'bg-white dark:bg-dark-surface text-foreground dark:text-dark-text rounded-tl-none'}">
                                    {#if getImageUrl(msg.text)}
                                        <img
                                            src={getImageUrl(msg.text)}
                                            alt="chat image"
                                            class="max-w-full rounded-lg border border-white/20 cursor-zoom-in"
                                            onclick={() => openChatImage(getImageUrl(msg.text))}
                                        />
                                    {:else}
                                        {msg.text}
                                    {/if}
                                </div>
                            </div>
                        {:else}
                            <div class="h-full flex flex-col items-center justify-center text-muted-foreground/60 space-y-2 opacity-60">
                                <MessageSquare size={48} strokeWidth={1} />
                                <p class="text-sm font-medium">Start a conversation</p>
                                <p class="text-xs">Send a message to {selectedDmAttendee.name}</p>
                            </div>
                        {/each}
                    </div>

                    <div class="p-4 border-t border-border dark:border-dark-border bg-white dark:bg-dark-surface">
                        <form onsubmit={(e) => { e.preventDefault(); sendDM(); }}>
                            <div class="relative flex items-center gap-2">
                                <input
                                    type="text"
                                    bind:value={dmMessage}
                                    placeholder="Type a message..."
                                    aria-label={$t("editor.chat_placeholder")}
                                    onpaste={handleChatPaste}
                                    class="flex-1 pl-4 pr-12 py-3 bg-muted dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary text-sm text-foreground dark:text-dark-text"
                                />
                                <input
                                    type="file"
                                    accept="image/*"
                                    bind:this={imageInput}
                                    onchange={(e) => {
                                        const input = e.currentTarget as HTMLInputElement;
                                        const file = input.files?.[0];
                                        if (file) attachImage(file);
                                        input.value = "";
                                    }}
                                    class="hidden"
                                />
                                <button
                                    type="button"
                                    class="absolute right-10 top-1/2 -translate-y-1/2 p-2 text-muted-foreground hover:text-primary hover:bg-accent/70 dark:hover:bg-primary/10 rounded-lg transition-all"
                                    onclick={() => imageInput?.click()}
                                    aria-label={$t("common.upload")}
                                    title={$t("common.upload")}
                                >
                                    <Image size={18} />
                                </button>
                                <button
                                    type="submit"
                                    class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-primary hover:bg-accent/70 dark:hover:bg-primary/10 rounded-lg transition-all"
                                    aria-label={$t("editor.send_dm")}
                                >
                                    <Send size={18} />
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            {/if}
        {/if}
    </div>
</div>
{#if chatImageLightboxUrl}
    <div
        class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-6"
        role="dialog"
        aria-modal="true"
        onclick={closeChatImage}
    >
        <img
            src={chatImageLightboxUrl}
            alt="chat image enlarged"
            class="max-h-[90vh] max-w-[90vw] rounded-xl shadow-2xl border border-white/10"
            onclick={(e) => e.stopPropagation()}
        />
        <button
            type="button"
            class="absolute top-4 right-4 text-white/80 hover:text-white text-2xl font-bold"
            aria-label="Close image"
            onclick={closeChatImage}
        >
            ×
        </button>
    </div>
{/if}
