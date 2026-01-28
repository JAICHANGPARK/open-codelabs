<script lang="ts">
    import { 
        Users, 
        MessageSquare, 
        Bell, 
        Send, 
        X 
    } from "lucide-svelte";
    import { slide } from "svelte/transition";
    import { t } from "svelte-i18n";
    import type { Attendee, HelpRequest } from "$lib/api";

    let {
        attendees,
        helpRequests,
        chatTab = $bindable(),
        dmTarget = $bindable(),
        dmMessage = $bindable(),
        chatMessage = $bindable(),
        filteredMessages,
        handleResolveHelp,
        sendChat,
        sendDM
    } = $props<{
        attendees: Attendee[];
        helpRequests: HelpRequest[];
        chatTab: "public" | "direct";
        dmTarget: Attendee | null;
        dmMessage: string;
        chatMessage: string;
        filteredMessages: any[];
        handleResolveHelp: (id: string) => void;
        sendChat: () => void;
        sendDM: () => void;
    }>();
</script>

<div class="grid grid-cols-1 xl:grid-cols-2 gap-6 sm:gap-8 h-full">
    <!-- Left: Activity & Help -->
    <div class="space-y-6 flex flex-col h-full min-w-0">
        <div class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col">
            <div class="p-4 bg-red-50 dark:bg-red-500/10 border-b border-red-100 dark:border-red-500/20 flex items-center gap-2">
                <Bell size={18} class="text-[#EA4335]" />
                <h3 class="font-bold text-[#EA4335]">
                    {$t("help.request")} ({helpRequests.length})
                </h3>
            </div>
            <div class="p-4 space-y-3 max-h-60 overflow-y-auto">
                {#each helpRequests as hr}
                    <div class="p-3 bg-red-50/50 dark:bg-red-500/5 rounded-xl border border-red-100 dark:border-red-500/10 flex justify-between items-center" in:slide>
                        <div>
                            <p class="font-bold text-[#202124] dark:text-dark-text text-sm">
                                {hr.attendee_name}
                            </p>
                            <p class="text-xs text-[#EA4335]">
                                {$t("editor.stuck_on_step", { values: { step: hr.step_number } })}
                            </p>
                        </div>
                        <button
                            onclick={() => handleResolveHelp(hr.id)}
                            class="text-xs font-bold text-white bg-[#EA4335] px-3 py-1.5 rounded-full hover:bg-[#D93025] transition-colors shadow-sm"
                        >
                            {$t("editor.resolve")}
                        </button>
                    </div>
                {:else}
                    <p class="text-center py-6 text-[#9AA0A6] dark:text-dark-text-muted text-sm">
                        No pending help requests
                    </p>
                {/each}
            </div>
        </div>

        <div class="flex-1 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col min-h-[300px] lg:min-h-[400px]">
            <div class="p-4 bg-[#F8F9FA] dark:bg-white/5 border-b border-[#E8EAED] dark:border-dark-border flex items-center gap-2">
                <Users size={18} class="text-[#4285F4]" />
                <h3 class="font-bold text-[#3C4043] dark:text-dark-text">
                    {$t("common.attendee")} ({attendees.length})
                </h3>
            </div>
            <div class="p-4 space-y-2 overflow-y-auto">
                {#each attendees as attendee}
                    <div class="flex items-center justify-between p-2 hover:bg-[#F8F9FA] dark:hover:bg-white/5 rounded-lg transition-colors group">
                        <div class="flex items-center gap-3">
                            <div class="w-8 h-8 rounded-full bg-[#E8EAED] dark:bg-white/10 flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted text-xs font-bold uppercase">
                                {attendee.name.charAt(0)}
                            </div>
                            <div>
                                <p class="text-sm font-bold text-[#202124] dark:text-dark-text">
                                    {attendee.name}
                                </p>
                                <p class="text-[10px] text-[#9AA0A6] dark:text-dark-text-muted">
                                    {$t("submission_panel.attendee_code")}: {attendee.code}
                                </p>
                            </div>
                        </div>
                        <button
                            type="button"
                            onclick={() => (dmTarget = attendee)}
                            class="p-2 text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-lg opacity-0 lg:opacity-0 group-hover:opacity-100 transition-all"
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
    <div class="bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl overflow-hidden shadow-sm flex flex-col h-full min-h-[500px] lg:min-h-[600px]">
        <div class="flex border-b border-[#E8EAED] dark:border-dark-border">
            <button
                onclick={() => (chatTab = "public")}
                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab === 'public'
                    ? 'text-[#4285F4] border-b-2 border-[#4285F4] bg-[#F8F9FA] dark:bg-white/5'
                    : 'text-[#5F6368] dark:text-dark-text-muted hover:bg-[#F1F3F4] dark:hover:bg-white/5'}"
            >
                <Users size={16} /> Public Chat
            </button>
            <button
                onclick={() => (chatTab = "direct")}
                class="flex-1 py-3 text-sm font-bold transition-all flex justify-center items-center gap-2 {chatTab === 'direct'
                    ? 'text-[#4285F4] border-b-2 border-[#4285F4] bg-[#F8F9FA] dark:bg-white/5'
                    : 'text-[#5F6368] dark:text-dark-text-muted hover:bg-[#F1F3F4] dark:hover:bg-white/5'}"
            >
                <MessageSquare size={16} /> Direct Messages
            </button>
        </div>

        <div class="flex-1 p-4 space-y-4 overflow-y-auto bg-[#F8F9FA] dark:bg-dark-bg/50" id="chat-messages">
            {#each filteredMessages as msg}
                <div class="flex flex-col {msg.self ? 'items-end' : 'items-start'}">
                    <span class="text-[10px] text-[#5F6368] dark:text-dark-text-muted font-bold mb-1 mx-1 uppercase">
                        {msg.sender} &bull; {msg.time}
                    </span>
                    <div class="max-w-[85%] p-3 rounded-2xl text-sm shadow-sm {msg.self
                        ? 'bg-[#4285F4] text-white rounded-tr-none'
                        : 'bg-white dark:bg-dark-surface text-[#3C4043] dark:text-dark-text rounded-tl-none'}">
                        {msg.text}
                    </div>
                </div>
            {:else}
                <div class="h-full flex flex-col items-center justify-center text-[#BDC1C6] space-y-2 opacity-60">
                    <MessageSquare size={48} strokeWidth={1} />
                    <p class="text-sm font-medium">{$t("editor.no_messages")}</p>
                </div>
            {/each}
        </div>

        <div class="p-4 border-t border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface">
            <form onsubmit={(e) => { e.preventDefault(); chatTab === "public" ? sendChat() : sendDM(); }}>
                {#if chatTab === "direct" && !dmTarget}
                    <div class="mb-3 p-3 bg-amber-50 dark:bg-amber-500/10 border border-amber-100 dark:border-amber-500/20 rounded-xl flex items-center gap-3 text-amber-700 dark:text-amber-400 text-xs font-bold">
                        <MessageSquare size={14} aria-hidden="true" />
                        {$t("editor.dm_select_prompt")}
                    </div>
                {/if}

                <div class="relative flex items-center gap-2">
                    {#if chatTab === "direct" && dmTarget}
                        <div class="absolute left-3 top-1/2 -translate-y-1/2 flex items-center gap-2 bg-[#4285F4] text-white px-2 py-1 rounded-lg text-[10px] font-bold shadow-sm">
                            <span>To: {dmTarget.name}</span>
                            <button type="button" onclick={() => (dmTarget = null)} class="hover:text-red-200" aria-label={$t("common.close")}><X size={12} /></button>
                        </div>
                    {/if}

                    {#if chatTab === "public"}
                        <input
                            type="text"
                            bind:value={chatMessage}
                            placeholder="Send a message to everyone..."
                            aria-label={$t("editor.chat_placeholder")}
                            class="flex-1 pl-4 pr-12 py-3 bg-[#F8F9FA] dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-xl outline-none focus:border-[#4285F4] text-sm text-[#202124] dark:text-dark-text"
                        />
                    {:else}
                        <input
                            type="text"
                            bind:value={dmMessage}
                            placeholder="Type a message..."
                            aria-label={$t("editor.chat_placeholder")}
                            class="flex-1 {dmTarget ? 'pl-24' : 'pl-4'} pr-12 py-3 bg-[#F8F9FA] dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-xl outline-none focus:border-[#4285F4] text-sm text-[#202124] dark:text-dark-text"
                        />
                    {/if}
                    <button
                        type="submit"
                        class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-[#4285F4] hover:bg-[#E8F0FE] dark:hover:bg-[#4285F4]/10 rounded-lg transition-all"
                        disabled={chatTab === "direct" && !dmTarget}
                        aria-label={$t("editor.send_dm")}
                    >
                        <Send size={18} />
                    </button>
                </div>
            </form>
        </div>
    </div>
</div>
