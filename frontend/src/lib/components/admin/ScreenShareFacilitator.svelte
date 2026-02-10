<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import {
        Monitor,
        MonitorOff,
        Loader2,
        Signal,
        ChevronDown,
    } from "lucide-svelte";
    import {
        ScreenShareService,
        type WebRTCSignal,
        type QualityPreset,
        type NetworkStats,
        QUALITY_PRESETS,
    } from "$lib/ScreenShareService";
    import { t } from "svelte-i18n";

    let { ws, codelabId } = $props<{
        ws: WebSocket | null;
        codelabId: string;
    }>();

    let isSharing = $state(false);
    let isLoading = $state(false);
    let service: ScreenShareService | null = null;
    let localStream = $state<MediaStream | null>(null);
    let selectedPreset = $state<QualityPreset>("auto");
    let networkStats = $state<NetworkStats | null>(null);
    let showQualityMenu = $state(false);

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
        {
            value: "low",
            labelKey: "screen_share.quality_low",
            desc: "480p · 10fps",
        },
        {
            value: "medium",
            labelKey: "screen_share.quality_medium",
            desc: "720p · 15fps",
        },
        {
            value: "high",
            labelKey: "screen_share.quality_high",
            desc: "1080p · 30fps",
        },
    ];

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
                // For facilitator, we might want to show a preview if needed
            },
        );
    });

    onDestroy(() => {
        if (isSharing) {
            stopSharing();
        }
    });

    async function toggleSharing() {
        if (isSharing) {
            stopSharing();
        } else {
            await startSharing();
        }
    }

    async function startSharing() {
        if (!service) return;
        isLoading = true;
        try {
            const stream = await service.startScreenShare(selectedPreset);
            if (stream) {
                localStream = stream;
                isSharing = true;

                // Track ending share via browser's built-in button
                stream.getVideoTracks()[0].onended = () => {
                    stopSharing();
                };

                // Start network stats monitoring
                service.startStatsMonitoring((stats) => {
                    networkStats = stats;
                }, 2000);

                // Notify backend
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(
                        JSON.stringify({
                            type: "screen_share_status",
                            status: "facilitator_started",
                        }),
                    );
                }
            }
        } catch (e) {
            console.error(e);
        } finally {
            isLoading = false;
        }
    }

    function stopSharing() {
        if (service) {
            service.stopScreenShare();
        }
        isSharing = false;
        localStream = null;
        networkStats = null;

        // Notify backend
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(
                JSON.stringify({
                    type: "screen_share_status",
                    status: "facilitator_stopped",
                }),
            );
        }
    }

    async function changePreset(preset: QualityPreset) {
        selectedPreset = preset;
        showQualityMenu = false;
        if (isSharing && service) {
            await service.applyQualityPreset(preset);
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

    // Handle incoming signals
    $effect(() => {
        if (!ws) return;

        const handler = (event: MessageEvent) => {
            try {
                const data = JSON.parse(event.data);
                if (
                    data.type === "webrtc_signal" &&
                    isSharing &&
                    data.stream_type === "facilitator_to_attendee"
                ) {
                    service?.handleSignal(data.signal, data.sender_id);
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
</script>

<div class="flex items-center gap-1.5 relative">
    <!-- Quality Preset Selector -->
    <div class="relative">
        <button
            type="button"
            onclick={() => (showQualityMenu = !showQualityMenu)}
            class="flex items-center gap-1 px-2 py-1.5 rounded-full text-[10px] font-medium transition-all
                   bg-surface-100 dark:bg-surface-700 text-surface-600 dark:text-surface-300
                   hover:bg-surface-200 dark:hover:bg-surface-600 border border-surface-200 dark:border-surface-600"
            title={$t("screen_share.quality")}
        >
            <Signal size={12} />
            <span class="hidden sm:inline">
                {selectedPreset === "auto"
                    ? "Auto"
                    : QUALITY_PRESETS[selectedPreset].label}
            </span>
            <ChevronDown size={10} />
        </button>

        {#if showQualityMenu}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="absolute top-full right-0 mt-1 w-44 bg-white dark:bg-surface-800 rounded-lg shadow-xl
                       border border-surface-200 dark:border-surface-600 z-50 py-1 overflow-hidden"
                onmouseleave={() => (showQualityMenu = false)}
            >
                {#each presetOptions as opt}
                    <button
                        type="button"
                        onclick={() => changePreset(opt.value)}
                        class="w-full text-left px-3 py-2 text-xs hover:bg-primary/10 transition flex justify-between items-center
                               {selectedPreset === opt.value
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

    <!-- Share Button -->
    <button
        type="button"
        onclick={toggleSharing}
        disabled={isLoading}
        class="flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-bold transition-all shadow-sm active:scale-95 {isSharing
            ? 'bg-red-100 text-red-600 hover:bg-red-200 dark:bg-red-900/20 dark:text-red-400'
            : 'bg-primary/10 text-primary hover:bg-primary/20'}"
        title={isSharing ? $t("screen_share.stop") : $t("screen_share.start")}
    >
        {#if isLoading}
            <Loader2 size={16} class="animate-spin" />
        {:else if isSharing}
            <MonitorOff size={16} />
        {:else}
            <Monitor size={16} />
        {/if}
        <span class="hidden sm:inline">
            {isSharing
                ? $t("screen_share.sharing")
                : $t("screen_share.share_screen")}
        </span>
    </button>

    <!-- Live Network Stats (shown while sharing) -->
    {#if isSharing && networkStats}
        {@const signal = getSignalStrength(networkStats)}
        <div
            class="flex items-center gap-1.5 px-2 py-1 rounded-full text-[10px] font-mono
                    bg-surface-100 dark:bg-surface-700 border border-surface-200 dark:border-surface-600
                    {signal.color}"
            title="{networkStats.resolution} · {networkStats.frameRate}fps · {networkStats.bitrate}kbps"
        >
            <!-- Signal bars -->
            <div class="flex items-end gap-px h-3">
                <div
                    class="w-[3px] rounded-sm {signal.bars >= 1
                        ? 'bg-current'
                        : 'bg-surface-300 dark:bg-surface-600'}"
                    style="height: 33%"
                ></div>
                <div
                    class="w-[3px] rounded-sm {signal.bars >= 2
                        ? 'bg-current'
                        : 'bg-surface-300 dark:bg-surface-600'}"
                    style="height: 66%"
                ></div>
                <div
                    class="w-[3px] rounded-sm {signal.bars >= 3
                        ? 'bg-current'
                        : 'bg-surface-300 dark:bg-surface-600'}"
                    style="height: 100%"
                ></div>
            </div>
            <span class="text-surface-500 dark:text-surface-400">
                {networkStats.bitrate > 0 ? `${networkStats.bitrate}k` : "—"}
            </span>
        </div>
    {/if}
</div>
