<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import {
        ScreenShareService,
        type QualityPreset,
        type NetworkStats,
        QUALITY_PRESETS,
    } from "$lib/ScreenShareService";
    import {
        Maximize2,
        Minimize2,
        X,
        MonitorUp,
        Loader2,
        Laptop,
        Signal,
        ChevronDown,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { fade } from "svelte/transition";

    let { ws, codelabId } = $props<{
        ws: WebSocket | null;
        codelabId: string;
    }>();

    let isFacilitatorSharing = $state(false);
    let remoteStream = $state<MediaStream | null>(null);
    let service: ScreenShareService | null = null;
    let videoElement = $state<HTMLVideoElement | null>(null);

    // Attendee Sharing State
    let isMeSharing = $state(false);
    let myStream = $state<MediaStream | null>(null);
    let myService: ScreenShareService | null = null;
    let myPreset = $state<QualityPreset>("auto");
    let myStats = $state<NetworkStats | null>(null);
    let showMyQualityMenu = $state(false);

    const presetOptions: {
        value: QualityPreset;
        labelKey: string;
        desc: string;
    }[] = [
        {
            value: "auto",
            labelKey: "screen_share.quality_auto",
            desc: "Adaptive",
        },
        { value: "low", labelKey: "screen_share.quality_low", desc: "480p" },
        {
            value: "medium",
            labelKey: "screen_share.quality_medium",
            desc: "720p",
        },
        { value: "high", labelKey: "screen_share.quality_high", desc: "1080p" },
    ];

    let isMinimized = $state(false);
    let isMonitorVisible = $state(false);

    // Position for draggable (from bottom-right)
    let position = $state({ right: 20, bottom: 80 });
    let dimensions = $state({ width: 320, height: 220 });
    let isDragging = $state(false);
    let isResizing = $state(false);
    let startMousePos = { x: 0, y: 0 };
    let startPosition = { right: 0, bottom: 0 };
    let startDimensions = { width: 0, height: 0 };

    onMount(() => {
        service = new ScreenShareService(
            (signal, targetId) => {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(
                        JSON.stringify({
                            type: "webrtc_signal",
                            target_id: targetId,
                            signal,
                            stream_type: "facilitator_to_attendee",
                        }),
                    );
                }
            },
            (stream) => {
                remoteStream = stream;
                if (videoElement) {
                    videoElement.srcObject = stream;
                }
            },
        );

        myService = new ScreenShareService(
            (signal, targetId) => {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(
                        JSON.stringify({
                            type: "webrtc_signal",
                            target_id: "facilitator", // Send to facilitator
                            signal,
                            stream_type: "attendee_to_facilitator",
                        }),
                    );
                }
            },
            (stream) => {
                // Local stream
            },
        );
    });

    // Re-clamp PiP position when the browser window is resized
    $effect(() => {
        if (!isMonitorVisible) return;

        const handleResize = () => clampPosition();
        window.addEventListener("resize", handleResize);

        return () => {
            window.removeEventListener("resize", handleResize);
        };
    });

    onDestroy(() => {
        service?.stopScreenShare();
        if (isMeSharing) stopMyShare();
    });

    async function startMyShare() {
        if (!myService) return;
        const stream = await myService.startScreenShare(myPreset);
        if (stream) {
            myStream = stream;
            isMeSharing = true;

            // Handle browser stop button
            stream.getVideoTracks()[0].onended = () => stopMyShare();

            // Start stats monitoring
            myService.startStatsMonitoring((stats) => {
                myStats = stats;
            }, 2000);

            // Notify facilitator
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(
                    JSON.stringify({
                        type: "attendee_screen_status",
                        status: "started",
                    }),
                );
            }
        }
    }

    function stopMyShare() {
        myService?.stopScreenShare();
        isMeSharing = false;
        myStream = null;
        myStats = null;

        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(
                JSON.stringify({
                    type: "attendee_screen_status",
                    status: "stopped",
                }),
            );
        }
    }

    async function changeMyPreset(preset: QualityPreset) {
        myPreset = preset;
        showMyQualityMenu = false;
        if (isMeSharing && myService) {
            await myService.applyQualityPreset(preset);
        }
    }

    function getSignalStrength(stats: NetworkStats | null): {
        color: string;
        bars: number;
    } {
        if (!stats || stats.bitrate === 0)
            return { color: "text-gray-400", bars: 0 };
        if (stats.packetLoss > 10 || stats.qualityLimited)
            return { color: "text-red-400", bars: 1 };
        if (stats.packetLoss > 3 || stats.bitrate < 200)
            return { color: "text-yellow-400", bars: 2 };
        return { color: "text-green-400", bars: 3 };
    }

    $effect(() => {
        if (!ws) return;

        const handler = (event: MessageEvent) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === "screen_share_status") {
                    console.log(
                        "Attendee received screen_share_status:",
                        data.status,
                    );
                    if (data.status === "facilitator_started") {
                        isFacilitatorSharing = true;
                        isMonitorVisible = true;
                        service?.createOffer("facilitator");
                    } else if (data.status === "facilitator_stopped") {
                        isFacilitatorSharing = false;
                        isMonitorVisible = false;
                        remoteStream = null;
                        service?.stopScreenShare();
                    }
                } else if (data.type === "webrtc_signal") {
                    if (
                        data.sender_id === "facilitator" &&
                        data.stream_type === "facilitator_to_attendee"
                    ) {
                        service?.handleSignal(data.signal, data.sender_id);
                    } else if (
                        isMeSharing &&
                        data.stream_type === "attendee_to_facilitator"
                    ) {
                        // Handle answer from facilitator
                        myService?.handleSignal(data.signal, data.sender_id);
                    }
                }
            } catch (e) {
                // ignore
            }
        };

        ws.addEventListener("message", handler);

        return () => {
            ws.removeEventListener("message", handler);
        };
    });

    $effect(() => {
        if (videoElement && remoteStream) {
            videoElement.srcObject = remoteStream;
        }
    });

    function clampPosition() {
        const margin = 10;
        const w = isMinimized ? 200 : dimensions.width;
        const h = isMinimized ? 40 : dimensions.height;

        position.right = Math.max(
            margin,
            Math.min(window.innerWidth - w - margin, position.right),
        );
        position.bottom = Math.max(
            margin,
            Math.min(window.innerHeight - h - margin, position.bottom),
        );
    }

    function handleMouseDown(e: MouseEvent) {
        isDragging = true;
        startMousePos.x = e.clientX;
        startMousePos.y = e.clientY;
        startPosition.right = position.right;
        startPosition.bottom = position.bottom;

        window.addEventListener("mousemove", handleMouseMove);
        window.addEventListener("mouseup", handleMouseUp);
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isDragging) return;

        const dx = e.clientX - startMousePos.x;
        const dy = e.clientY - startMousePos.y;

        position.right = startPosition.right - dx;
        position.bottom = startPosition.bottom - dy;

        clampPosition();
    }

    function handleMouseUp() {
        isDragging = false;
        isResizing = false;
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
        window.removeEventListener("mousemove", handleResizeMove);
    }

    function handleResizeDown(e: MouseEvent) {
        e.stopPropagation();
        isResizing = true;
        startMousePos.x = e.clientX;
        startMousePos.y = e.clientY;
        startDimensions.width = dimensions.width;
        startDimensions.height = dimensions.height;
        startPosition.right = position.right;
        startPosition.bottom = position.bottom;

        window.addEventListener("mousemove", handleResizeMove);
        window.addEventListener("mouseup", handleMouseUp);
    }

    function handleResizeMove(e: MouseEvent) {
        if (!isResizing) return;

        const dx = e.clientX - startMousePos.x;
        const dy = e.clientY - startMousePos.y;

        // Since we are anchored bottom-right:
        // Moving mouse LEFT (negative dx) increases width
        // Moving mouse UP (negative dy) increases height
        const newWidth = Math.max(
            200,
            Math.min(800, startDimensions.width - dx),
        );
        const newHeight = Math.max(
            150,
            Math.min(600, startDimensions.height - dy),
        );

        dimensions.width = newWidth;
        dimensions.height = newHeight;

        // Boundaries check for right/bottom already implicitly handled by keeping right/bottom fixed
        // but if we want to prevent it from going off top/left:
        const maxRight = window.innerWidth - dimensions.width - 10;
        const maxBottom = window.innerHeight - dimensions.height - 10;

        if (position.right > maxRight) position.right = maxRight;
        if (position.bottom > maxBottom) position.bottom = maxBottom;
    }
</script>

<!-- Share Button (Inline in Header) -->
{#if isFacilitatorSharing && !isMonitorVisible}
    <button
        onclick={() => (isMonitorVisible = true)}
        class="flex items-center gap-2 px-3 py-1.5 bg-emerald-500 text-white rounded-full transition-all shadow-lg hover:scale-105 active:scale-95 animate-bounce"
        title={$t("screen_share.view_facilitator")}
    >
        <Laptop size={16} />
        <span class="text-[10px] font-bold uppercase hidden sm:inline"
            >{$t("screen_share.attendee_pip_title")}</span
        >
    </button>
{/if}

<div class="flex items-center gap-1 relative">
    {#if isMeSharing}
        <!-- Quality selector while sharing -->
        <div class="relative">
            <button
                type="button"
                onclick={() => (showMyQualityMenu = !showMyQualityMenu)}
                class="flex items-center gap-0.5 px-1.5 py-1 rounded-full text-[9px] font-medium transition-all
                       bg-surface-100 dark:bg-surface-700 text-surface-500 dark:text-surface-400
                       hover:bg-surface-200 dark:hover:bg-surface-600 border border-surface-200 dark:border-surface-600"
                title={$t("screen_share.quality")}
            >
                <Signal size={10} />
                <span class="hidden sm:inline"
                    >{myPreset === "auto"
                        ? "Auto"
                        : QUALITY_PRESETS[myPreset].label}</span
                >
                <ChevronDown size={8} />
            </button>

            {#if showMyQualityMenu}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                    class="absolute bottom-full right-0 mb-1 w-40 bg-white dark:bg-surface-800 rounded-lg shadow-xl
                           border border-surface-200 dark:border-surface-600 z-50 py-1 overflow-hidden"
                    onmouseleave={() => (showMyQualityMenu = false)}
                >
                    {#each presetOptions as opt}
                        <button
                            type="button"
                            onclick={() => changeMyPreset(opt.value)}
                            class="w-full text-left px-3 py-1.5 text-xs hover:bg-primary/10 transition flex justify-between items-center
                                   {myPreset === opt.value
                                ? 'bg-primary/5 text-primary font-bold'
                                : 'text-surface-700 dark:text-surface-300'}"
                        >
                            <span>{$t(opt.labelKey)}</span>
                            <span class="text-[10px] text-surface-400"
                                >{opt.desc}</span
                            >
                        </button>
                    {/each}
                </div>
            {/if}
        </div>

        <!-- Stats indicator -->
        {#if myStats}
            {@const signal = getSignalStrength(myStats)}
            <div
                class="flex items-center gap-px h-3 {signal.color}"
                title="{myStats.resolution} · {myStats.bitrate}kbps"
            >
                <div
                    class="w-[2px] rounded-sm {signal.bars >= 1
                        ? 'bg-current'
                        : 'bg-surface-300 dark:bg-surface-600'}"
                    style="height: 33%"
                ></div>
                <div
                    class="w-[2px] rounded-sm {signal.bars >= 2
                        ? 'bg-current'
                        : 'bg-surface-300 dark:bg-surface-600'}"
                    style="height: 66%"
                ></div>
                <div
                    class="w-[2px] rounded-sm {signal.bars >= 3
                        ? 'bg-current'
                        : 'bg-surface-300 dark:bg-surface-600'}"
                    style="height: 100%"
                ></div>
            </div>
        {/if}
    {/if}

    {#if !isMeSharing}
        <button
            onclick={startMyShare}
            class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-full transition-colors text-muted-foreground dark:text-dark-text-muted hover:text-primary"
            title={$t("screen_share.share_with_facilitator")}
        >
            <MonitorUp size={20} />
        </button>
    {:else}
        <button
            onclick={stopMyShare}
            class="p-2 bg-red-500/10 text-red-500 hover:bg-red-500/20 rounded-full transition-colors animate-pulse"
            title={$t("screen_share.stop_sharing_facilitator")}
        >
            <div class="relative">
                <MonitorUp size={20} />
                <div
                    class="absolute -top-1 -right-1 w-2 h-2 bg-red-500 rounded-full"
                ></div>
            </div>
        </button>
    {/if}
</div>

{#if isMonitorVisible}
    <div
        class="fixed z-[100] bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-xl shadow-2xl overflow-hidden transition-shadow duration-300 {isDragging
            ? 'shadow-3xl ring-2 ring-primary/20'
            : ''}"
        style="right: {position.right}px; bottom: {position.bottom}px; width: {isMinimized
            ? '200px'
            : dimensions.width + 'px'}; height: {isMinimized
            ? 'auto'
            : dimensions.height + 'px'};"
        transition:fade
    >
        <!-- Header / Drag Handle -->
        <div
            class="flex items-center justify-between px-3 py-2 bg-accent/50 dark:bg-white/5 cursor-move"
            onmousedown={handleMouseDown}
            role="button"
            tabindex="0"
        >
            <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"
                ></span>
                <span
                    class="text-[10px] font-bold uppercase tracking-wider text-muted-foreground dark:text-dark-text-muted"
                >
                    {$t("screen_share.attendee_pip_title")}
                </span>
            </div>
            <div class="flex items-center gap-1">
                <button
                    onclick={() => (isMinimized = !isMinimized)}
                    class="p-1 hover:bg-black/5 dark:hover:bg-white/5 rounded transition-colors"
                >
                    {#if isMinimized}
                        <Maximize2 size={14} />
                    {:else}
                        <Minimize2 size={14} />
                    {/if}
                </button>
                <button
                    onclick={() => (isMonitorVisible = false)}
                    class="p-1 hover:bg-red-500/10 text-red-500 rounded transition-colors"
                >
                    <X size={14} />
                </button>
            </div>
        </div>

        <!-- Video Content -->
        {#if !isMinimized}
            <div
                class="bg-black relative flex items-center justify-center grow overflow-hidden"
            >
                {#if !remoteStream}
                    <div
                        class="text-white/50 text-[10px] flex flex-col items-center gap-2"
                    >
                        <Loader2 size={20} class="animate-spin" />
                        <span>{$t("screen_share.connecting")}</span>
                    </div>
                {/if}
                <video
                    bind:this={videoElement}
                    autoplay
                    playsinline
                    class="w-full h-full object-contain"
                >
                    <track kind="captions" />
                </video>
            </div>

            <!-- Resize Handle -->
            <div
                class="absolute top-0 left-0 w-4 h-4 cursor-nw-resize hover:bg-primary/20 transition-colors z-[110] flex items-center justify-center"
                onmousedown={handleResizeDown}
                role="button"
                tabindex="0"
                title={$t("screen_share.resize")}
            >
                <div
                    class="w-1.5 h-1.5 border-t-2 border-l-2 border-primary/40"
                ></div>
            </div>
        {/if}
    </div>
{/if}
