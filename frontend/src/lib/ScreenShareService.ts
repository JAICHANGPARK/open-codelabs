export interface WebRTCSignal {
    type: 'offer' | 'answer' | 'candidate';
    sdp?: RTCSessionDescriptionInit;
    candidate?: RTCIceCandidateInit;
}

export type QualityPreset = 'auto' | 'low' | 'medium' | 'high';

export interface QualityConfig {
    width: number;
    height: number;
    frameRate: number;
    maxBitrate: number;
    label: string;
}

export interface NetworkStats {
    bitrate: number;        // kbps
    packetLoss: number;     // percentage
    resolution: string;     // e.g. "1280x720"
    frameRate: number;      // actual fps
    qualityLimited: boolean;
}

export const QUALITY_PRESETS: Record<Exclude<QualityPreset, 'auto'>, QualityConfig> = {
    low: { width: 854, height: 480, frameRate: 10, maxBitrate: 300_000, label: '480p' },
    medium: { width: 1280, height: 720, frameRate: 15, maxBitrate: 800_000, label: '720p' },
    high: { width: 1920, height: 1080, frameRate: 30, maxBitrate: 2_000_000, label: '1080p' },
};

// Auto preset defaults to medium constraints but allows WebRTC to adapt freely
const AUTO_CONFIG: QualityConfig = {
    width: 1920, height: 1080, frameRate: 30, maxBitrate: 2_500_000, label: 'Auto'
};

export class ScreenShareService {
    private pcs: Map<string, RTCPeerConnection> = new Map();
    private localStream: MediaStream | null = null;
    private onSignal: (signal: WebRTCSignal, targetId?: string) => void;
    private onStream: (stream: MediaStream) => void;
    private currentPreset: QualityPreset = 'auto';
    private statsInterval: ReturnType<typeof setInterval> | null = null;
    private lastBytesSent = 0;
    private lastStatsTime = 0;
    private iceServers: RTCIceServer[] = [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun1.l.google.com:19302' }
    ];

    constructor(
        onSignal: (signal: WebRTCSignal, targetId?: string) => void,
        onStream: (stream: MediaStream) => void
    ) {
        this.onSignal = onSignal;
        this.onStream = onStream;
    }

    getPreset(): QualityPreset {
        return this.currentPreset;
    }

    async startScreenShare(preset: QualityPreset = 'auto'): Promise<MediaStream | null> {
        if (typeof window === "undefined" || typeof navigator === "undefined") return null;
        this.currentPreset = preset;
        const config = preset === 'auto' ? AUTO_CONFIG : QUALITY_PRESETS[preset];

        try {
            this.localStream = await navigator.mediaDevices.getDisplayMedia({
                video: {
                    displaySurface: 'monitor',
                    width: { ideal: config.width },
                    height: { ideal: config.height },
                    frameRate: { ideal: config.frameRate },
                },
                audio: false
            });
            return this.localStream;
        } catch (e) {
            console.error('Error starting screen share:', e);
            return null;
        }
    }

    stopScreenShare() {
        this.stopStatsMonitoring();
        if (this.localStream) {
            this.localStream.getTracks().forEach(track => track.stop());
            this.localStream = null;
        }
        this.closeAllPeerConnections();
    }

    /** Change quality preset on all active peer connections mid-stream */
    async applyQualityPreset(preset: QualityPreset): Promise<void> {
        this.currentPreset = preset;
        const config = preset === 'auto' ? AUTO_CONFIG : QUALITY_PRESETS[preset];
        const isAuto = preset === 'auto';

        for (const pc of this.pcs.values()) {
            const senders = pc.getSenders().filter(s => s.track?.kind === 'video');
            for (const sender of senders) {
                try {
                    const params = sender.getParameters();
                    if (!params.encodings || params.encodings.length === 0) {
                        params.encodings = [{}];
                    }

                    params.encodings[0].maxBitrate = config.maxBitrate;

                    // For auto, let WebRTC freely adapt; for fixed presets, scale down
                    if (!isAuto && this.localStream) {
                        const videoTrack = this.localStream.getVideoTracks()[0];
                        const settings = videoTrack.getSettings();
                        const sourceWidth = settings.width || 1920;
                        const scaleFactor = Math.max(1, sourceWidth / config.width);
                        params.encodings[0].scaleResolutionDownBy = scaleFactor;
                    } else {
                        params.encodings[0].scaleResolutionDownBy = 1;
                    }

                    // degradationPreference: balanced lets WebRTC auto-adjust both FPS and resolution
                    (params as any).degradationPreference = isAuto ? 'balanced' : 'maintain-resolution';

                    await sender.setParameters(params);
                } catch (e) {
                    console.warn('Failed to apply quality preset to sender:', e);
                }
            }
        }

        // Also apply frameRate constraint to the track itself
        if (this.localStream) {
            const videoTrack = this.localStream.getVideoTracks()[0];
            try {
                await videoTrack.applyConstraints({
                    frameRate: { ideal: config.frameRate },
                });
            } catch (e) {
                console.warn('Failed to apply track constraints:', e);
            }
        }
    }

    /** Get real-time network stats from the first active peer connection */
    async getNetworkStats(): Promise<NetworkStats | null> {
        const pc = this.pcs.values().next().value;
        if (!pc) return null;

        try {
            const stats = await pc.getStats();
            let bitrate = 0;
            let packetLoss = 0;
            let resolution = '—';
            let frameRate = 0;
            let qualityLimited = false;

            stats.forEach((report) => {
                if (report.type === 'outbound-rtp' && report.kind === 'video') {
                    // Calculate bitrate
                    const now = Date.now();
                    if (this.lastStatsTime > 0 && this.lastBytesSent > 0) {
                        const elapsed = (now - this.lastStatsTime) / 1000;
                        const bytes = report.bytesSent - this.lastBytesSent;
                        bitrate = Math.round((bytes * 8) / elapsed / 1000); // kbps
                    }
                    this.lastBytesSent = report.bytesSent;
                    this.lastStatsTime = now;

                    // Resolution and FPS
                    if (report.frameWidth && report.frameHeight) {
                        resolution = `${report.frameWidth}x${report.frameHeight}`;
                    }
                    if (report.framesPerSecond) {
                        frameRate = Math.round(report.framesPerSecond);
                    }
                    if (report.qualityLimitationReason && report.qualityLimitationReason !== 'none') {
                        qualityLimited = true;
                    }
                }
                if (report.type === 'remote-inbound-rtp' && report.kind === 'video') {
                    if (report.packetsLost !== undefined && report.packetsReceived !== undefined) {
                        const total = report.packetsLost + report.packetsReceived;
                        packetLoss = total > 0 ? Math.round((report.packetsLost / total) * 100) : 0;
                    }
                }
            });

            return { bitrate, packetLoss, resolution, frameRate, qualityLimited };
        } catch (e) {
            return null;
        }
    }

    /** Start periodic stats monitoring, calling the callback every interval */
    startStatsMonitoring(callback: (stats: NetworkStats) => void, intervalMs = 2000) {
        this.stopStatsMonitoring();
        this.lastBytesSent = 0;
        this.lastStatsTime = 0;
        this.statsInterval = setInterval(async () => {
            const stats = await this.getNetworkStats();
            if (stats) callback(stats);
        }, intervalMs);
    }

    stopStatsMonitoring() {
        if (this.statsInterval) {
            clearInterval(this.statsInterval);
            this.statsInterval = null;
        }
    }

    private createPeerConnection(targetId: string) {
        if (this.pcs.has(targetId)) return this.pcs.get(targetId)!;

        const pc = new RTCPeerConnection({ iceServers: this.iceServers });
        this.pcs.set(targetId, pc);

        pc.onicecandidate = (event) => {
            if (event.candidate) {
                this.onSignal({
                    type: 'candidate',
                    candidate: event.candidate.toJSON()
                }, targetId);
            }
        };

        pc.ontrack = (event) => {
            if (event.streams && event.streams[0]) {
                this.onStream(event.streams[0]);
            }
        };

        if (this.localStream) {
            this.localStream.getTracks().forEach(track => {
                pc.addTrack(track, this.localStream!);
            });

            // Apply encoding params immediately
            this.applyEncodingToPC(pc);
        }

        return pc;
    }

    private async applyEncodingToPC(pc: RTCPeerConnection) {
        // Wait a tick for senders to be ready
        await new Promise(r => setTimeout(r, 0));
        const config = this.currentPreset === 'auto' ? AUTO_CONFIG : QUALITY_PRESETS[this.currentPreset];
        const isAuto = this.currentPreset === 'auto';

        const senders = pc.getSenders().filter(s => s.track?.kind === 'video');
        for (const sender of senders) {
            try {
                const params = sender.getParameters();
                if (!params.encodings || params.encodings.length === 0) {
                    params.encodings = [{}];
                }
                params.encodings[0].maxBitrate = config.maxBitrate;
                (params as any).degradationPreference = isAuto ? 'balanced' : 'maintain-resolution';

                if (!isAuto && this.localStream) {
                    const videoTrack = this.localStream.getVideoTracks()[0];
                    const settings = videoTrack.getSettings();
                    const sourceWidth = settings.width || 1920;
                    const scaleFactor = Math.max(1, sourceWidth / config.width);
                    params.encodings[0].scaleResolutionDownBy = scaleFactor;
                }

                await sender.setParameters(params);
            } catch (e) {
                // Encoding params may not be available yet, ignore
            }
        }
    }

    async createOffer(targetId: string): Promise<void> {
        const pc = this.createPeerConnection(targetId);

        // If we are a receiver (no local stream), we need to add a transceiver to request video
        if (!this.localStream) {
            const hasVideoTransceiver = pc.getTransceivers().some(t => t.receiver.track.kind === 'video');
            if (!hasVideoTransceiver) {
                pc.addTransceiver('video', { direction: 'recvonly' });
            }
        }

        const offer = await pc.createOffer();
        await pc.setLocalDescription(offer);
        this.onSignal({
            type: 'offer',
            sdp: offer
        }, targetId);
    }

    async handleSignal(signal: WebRTCSignal, senderId: string): Promise<void> {
        const pc = this.createPeerConnection(senderId);

        if (signal.type === 'offer' && signal.sdp) {
            await pc.setRemoteDescription(new RTCSessionDescription(signal.sdp));
            // Only create answer if it's an offer
            if (pc.signalingState === 'have-remote-offer') {
                const answer = await pc.createAnswer();
                await pc.setLocalDescription(answer);
                this.onSignal({
                    type: 'answer',
                    sdp: answer
                }, senderId);
            }
        } else if (signal.type === 'answer' && signal.sdp) {
            await pc.setRemoteDescription(new RTCSessionDescription(signal.sdp));
        } else if (signal.type === 'candidate' && signal.candidate) {
            await pc.addIceCandidate(new RTCIceCandidate(signal.candidate));
        }
    }

    private closeAllPeerConnections() {
        this.pcs.forEach(pc => pc.close());
        this.pcs.clear();
    }
}
