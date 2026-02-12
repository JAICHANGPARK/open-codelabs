<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { ScreenShareService } from "$lib/ScreenShareService";
    import ScreenShareFacilitator from "./ScreenShareFacilitator.svelte";
    import {
        Loader2,
        Monitor,
        MonitorOff,
        Maximize2,
        X as XIcon,
        LayoutGrid,
        Grid2x2,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { fade } from "svelte/transition";

    let { ws, attendees } = $props<{
        ws: WebSocket | null;
        attendees: any[];
    }>();

    let activeStreams = $state<Map<string, MediaStream>>(new Map());
    let gridMode = $state<"large" | "compact">("large");
    let services = $state<Map<string, ScreenShareService>>(new Map());
    let loadingStreams = $state<Set<string>>(new Set());
    let enlargedAttendeeId = $state<string | null>(null);

    // Filter attendees who are sharing
    let sharingAttendees = $derived(
        attendees.filter((a: any) => a.is_sharing_screen),
    );

    onDestroy(() => {
        services.forEach((s) => s.stopScreenShare());
        activeStreams.clear();
        services.clear();
    });

    function handleStream(attendeeId: string, stream: MediaStream) {
        activeStreams.set(attendeeId, stream);
        loadingStreams.delete(attendeeId);
        // Trigger reactivity
        activeStreams = new Map(activeStreams);
        loadingStreams = new Set(loadingStreams);
    }

    function subscribeToStream(attendeeId: string) {
        if (services.has(attendeeId)) return;

        loadingStreams.add(attendeeId);
        loadingStreams = new Set(loadingStreams); // Reactivity

        const service = new ScreenShareService(
            (signal, targetId) => {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(
                        JSON.stringify({
                            type: "webrtc_signal",
                            target_id: targetId, // This might need adjustment if backend expects "attendee_id"
                            // But sticking to standard we check: backend handles routing by target_id
                            signal,
                            stream_type: "attendee_to_facilitator",
                        }),
                    );
                }
            },
            (stream) => handleStream(attendeeId, stream),
        );

        // Initiate connection
        service.createOffer(attendeeId);
        services.set(attendeeId, service);
        services = new Map(services);
    }

    function unsubscribeFromStream(attendeeId: string) {
        const service = services.get(attendeeId);
        if (service) {
            service.stopScreenShare();
            services.delete(attendeeId);
            services = new Map(services);
        }
        activeStreams.delete(attendeeId);
        activeStreams = new Map(activeStreams);
    }

    // Only cleanup streams for attendees who stopped sharing.
    // New streams are connected on-demand when facilitator clicks "Watch".
    $effect(() => {
        // Cleanup stopped sharers
        services.forEach((_, id) => {
            if (!sharingAttendees.find((a: any) => a.id === id)) {
                unsubscribeFromStream(id);
            }
        });
    });

    // Handle signals
    $effect(() => {
        if (!ws) return;
        const handler = (event: MessageEvent) => {
            try {
                const data = JSON.parse(event.data);
                if (
                    data.type === "webrtc_signal" &&
                    data.stream_type === "attendee_to_facilitator"
                ) {
                    // If it's a signal FROM an attendee
                    const senderId = data.sender_id;
                    if (services.has(senderId)) {
                        services
                            .get(senderId)
                            ?.handleSignal(data.signal, senderId);
                    }
                }
            } catch (e) {}
        };

        ws.addEventListener("message", handler);

        return () => {
            ws.removeEventListener("message", handler);
        };
    });
</script>

<div class="flex flex-col gap-6 p-4">
    <div
        class="bg-accent/20 border border-primary/20 rounded-xl p-6 flex flex-col md:flex-row justify-between items-center gap-4"
    >
        <div class="flex items-center gap-4">
            <div class="bg-primary/10 p-3 rounded-full">
                <Monitor class="text-primary" size={32} />
            </div>
            <div>
                <h2 class="text-xl font-bold">
                    {$t("screen_share.monitoring_title")}
                </h2>
                <p class="text-sm text-muted-foreground">
                    {$t("screen_share.monitoring_desc")}
                </p>
            </div>
        </div>

        <div class="flex items-center gap-3">
            <!-- Grid Layout Toggle -->
            <div
                class="flex items-center bg-surface-100 dark:bg-surface-700 rounded-lg p-0.5 border border-surface-200 dark:border-surface-600"
            >
                <button
                    type="button"
                    onclick={() => (gridMode = "large")}
                    class="p-1.5 rounded-md transition-all {gridMode === 'large'
                        ? 'bg-white dark:bg-surface-600 shadow-sm text-primary'
                        : 'text-surface-400 hover:text-surface-600'}"
                    title="Large view"
                >
                    <Grid2x2 size={16} />
                </button>
                <button
                    type="button"
                    onclick={() => (gridMode = "compact")}
                    class="p-1.5 rounded-md transition-all {gridMode ===
                    'compact'
                        ? 'bg-white dark:bg-surface-600 shadow-sm text-primary'
                        : 'text-surface-400 hover:text-surface-600'}"
                    title="Compact view"
                >
                    <LayoutGrid size={16} />
                </button>
            </div>
            <ScreenShareFacilitator {ws} codelabId={""} />
        </div>
    </div>

    <div
        class="grid gap-4 {gridMode === 'large'
            ? 'grid-cols-1 lg:grid-cols-2'
            : 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3'}"
    >
        {#if sharingAttendees.length === 0}
            <div class="col-span-full text-center py-12 text-muted-foreground">
                <MonitorOff size={48} class="mx-auto mb-4 opacity-50" />
                <p>{$t("screen_share.no_participants_sharing")}</p>
            </div>
        {/if}

        {#each sharingAttendees as attendee (attendee.id)}
            <div
                class="bg-card border border-border rounded-lg overflow-hidden shadow-sm flex flex-col"
            >
                <div
                    class="p-3 border-b border-border flex justify-between items-center bg-accent/30"
                >
                    <span class="font-medium truncate">{attendee.name}</span>
                    {#if services.has(attendee.id)}
                        <button
                            onclick={() => unsubscribeFromStream(attendee.id)}
                            class="text-xs bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400 px-2 py-1 rounded hover:bg-red-200 transition"
                        >
                            {$t("screen_share.stop_watching")}
                        </button>
                    {:else}
                        <button
                            onclick={() => subscribeToStream(attendee.id)}
                            class="text-xs bg-primary/10 text-primary px-2 py-1 rounded hover:bg-primary/20 transition flex items-center gap-1"
                        >
                            <Monitor size={12} />
                            {$t("screen_share.watch")}
                        </button>
                    {/if}
                    {#if activeStreams.has(attendee.id)}
                        <button
                            onclick={() => (enlargedAttendeeId = attendee.id)}
                            class="p-1 hover:bg-primary/10 rounded transition text-primary ml-1"
                            title={$t("screen_share.enlarge")}
                        >
                            <Maximize2 size={14} />
                        </button>
                    {/if}
                </div>

                <div
                    class="aspect-video bg-black relative flex items-center justify-center"
                >
                    {#if services.has(attendee.id)}
                        {#if activeStreams.get(attendee.id)}
                            <!-- Video Element Helper -->
                            <video
                                autoplay
                                playsinline
                                class="w-full h-full object-contain"
                                srcObject={activeStreams.get(attendee.id)}
                            >
                                <track kind="captions" />
                            </video>
                        {:else}
                            <div
                                class="text-white/50 text-xs flex flex-col items-center gap-2"
                            >
                                <Loader2 size={24} class="animate-spin" />
                                <span>{$t("screen_share.connecting")}</span>
                            </div>
                        {/if}
                    {:else}
                        <div
                            class="text-white/30 flex flex-col items-center justify-center h-full"
                        >
                            <Monitor size={32} />
                            <span class="text-xs mt-2"
                                >{$t("screen_share.click_to_view")}</span
                            >
                        </div>
                    {/if}
                </div>
            </div>
        {/each}
    </div>
</div>

{#if enlargedAttendeeId}
    {@const attendee = attendees.find((a: any) => a.id === enlargedAttendeeId)}
    {@const stream = activeStreams.get(enlargedAttendeeId)}
    <div
        class="fixed inset-0 z-[200] bg-black/90 flex flex-col p-4 md:p-8"
        transition:fade
    >
        <div class="flex justify-between items-center mb-4 text-white">
            <h3 class="text-xl font-bold flex items-center gap-2">
                <Monitor />
                {attendee?.name || "Attendee"}
            </h3>
            <button
                onclick={() => (enlargedAttendeeId = null)}
                class="p-2 hover:bg-white/10 rounded-full transition-colors text-white"
            >
                <XIcon size={32} />
            </button>
        </div>

        <div
            class="flex-1 flex items-center justify-center min-h-0 bg-black rounded-2xl overflow-hidden border border-white/10 shadow-2xl"
        >
            {#if stream}
                <video
                    autoplay
                    playsinline
                    class="max-w-full max-h-full w-full h-full object-contain"
                    srcObject={stream}
                >
                    <track kind="captions" />
                </video>
            {:else}
                <div class="flex flex-col items-center gap-4 text-white/50">
                    <Loader2 size={48} class="animate-spin" />
                    <p>{$t("screen_share.connecting")}</p>
                </div>
            {/if}
        </div>
    </div>
{/if}
