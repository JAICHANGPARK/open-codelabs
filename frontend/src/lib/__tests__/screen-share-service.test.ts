import { afterAll, beforeAll, beforeEach, describe, expect, test } from "bun:test";

type ScreenShareModule = typeof import("../ScreenShareService");

const originalWindow = (globalThis as any).window;
const originalNavigator = (globalThis as any).navigator;
const originalPeerConnection = (globalThis as any).RTCPeerConnection;
const originalSessionDescription = (globalThis as any).RTCSessionDescription;
const originalIceCandidate = (globalThis as any).RTCIceCandidate;
const originalSetInterval = globalThis.setInterval;
const originalClearInterval = globalThis.clearInterval;
const originalDateNow = Date.now;
const originalConsoleWarn = console.warn;
const originalConsoleError = console.error;

let screenShare: ScreenShareModule;

type MockSender = {
    track: any;
    params: any;
    throwOnSet: boolean;
    setCalls: any[];
    getParameters: () => any;
    setParameters: (params: any) => Promise<void>;
};

class MockTrack {
    kind = "video";
    stopped = false;
    settings = { width: 1920, height: 1080 };
    throwOnConstraints = false;

    stop() {
        this.stopped = true;
    }

    getSettings() {
        return this.settings;
    }

    async applyConstraints() {
        if (this.throwOnConstraints) throw new Error("constraints failed");
    }
}

class MockMediaStream {
    tracks: MockTrack[];

    constructor(track?: MockTrack) {
        this.tracks = [track ?? new MockTrack()];
    }

    getTracks() {
        return this.tracks;
    }

    getVideoTracks() {
        return this.tracks.filter((t) => t.kind === "video");
    }
}

class MockRTCPeerConnection {
    static instances: MockRTCPeerConnection[] = [];
    onicecandidate: ((event: any) => void) | null = null;
    ontrack: ((event: any) => void) | null = null;
    senders: MockSender[] = [];
    transceivers: any[] = [];
    signalingState = "stable";
    forceStableOnOffer = false;
    remoteDescriptions: any[] = [];
    localDescriptions: any[] = [];
    iceCandidates: any[] = [];
    closeCalled = false;
    throwOnStats = false;
    statsReport: Map<string, any> = new Map();

    constructor() {
        MockRTCPeerConnection.instances.push(this);
    }

    addTrack(track: any) {
        const sender: MockSender = {
            track,
            params: {},
            throwOnSet: false,
            setCalls: [],
            getParameters() {
                return { ...this.params };
            },
            async setParameters(params: any) {
                if (this.throwOnSet) throw new Error("setParameters failed");
                this.params = params;
                this.setCalls.push(params);
            },
        };
        this.senders.push(sender);
    }

    getSenders() {
        return this.senders;
    }

    async getStats() {
        if (this.throwOnStats) throw new Error("stats failed");
        return this.statsReport;
    }

    getTransceivers() {
        return this.transceivers;
    }

    addTransceiver(type: string, options: any) {
        this.transceivers.push({
            type,
            options,
            receiver: { track: { kind: type } },
        });
    }

    async createOffer() {
        return { type: "offer", sdp: "offer-sdp" };
    }

    async createAnswer() {
        return { type: "answer", sdp: "answer-sdp" };
    }

    async setLocalDescription(desc: any) {
        this.localDescriptions.push(desc);
    }

    async setRemoteDescription(desc: any) {
        this.remoteDescriptions.push(desc);
        if (desc?.type === "offer" && !this.forceStableOnOffer) {
            this.signalingState = "have-remote-offer";
        }
    }

    async addIceCandidate(candidate: any) {
        this.iceCandidates.push(candidate);
    }

    close() {
        this.closeCalled = true;
    }
}

describe("ScreenShareService", () => {
    beforeAll(async () => {
        (globalThis as any).RTCPeerConnection = MockRTCPeerConnection;
        (globalThis as any).RTCSessionDescription = function RTCSessionDescription(desc: any) {
            return desc;
        };
        (globalThis as any).RTCIceCandidate = function RTCIceCandidate(candidate: any) {
            return candidate;
        };
        screenShare = await import("../ScreenShareService");
    });

    beforeEach(() => {
        MockRTCPeerConnection.instances.length = 0;
        (globalThis as any).window = {};
        (globalThis as any).navigator = {
            mediaDevices: {
                async getDisplayMedia() {
                    return new MockMediaStream();
                },
            },
        };
        console.warn = () => {};
        console.error = () => {};
    });

    afterAll(() => {
        (globalThis as any).window = originalWindow;
        (globalThis as any).navigator = originalNavigator;
        (globalThis as any).RTCPeerConnection = originalPeerConnection;
        (globalThis as any).RTCSessionDescription = originalSessionDescription;
        (globalThis as any).RTCIceCandidate = originalIceCandidate;
        globalThis.setInterval = originalSetInterval;
        globalThis.clearInterval = originalClearInterval;
        Date.now = originalDateNow;
        console.warn = originalConsoleWarn;
        console.error = originalConsoleError;
    });

    test("starts and stops screen share, including startup errors", async () => {
        const onSignalCalls: any[] = [];
        const onStreamCalls: any[] = [];
        const service = new screenShare.ScreenShareService(
            (signal, targetId) => onSignalCalls.push({ signal, targetId }),
            (stream) => onStreamCalls.push(stream),
        );

        const originalWindowLocal = (globalThis as any).window;
        const originalNavigatorLocal = (globalThis as any).navigator;
        try {
            delete (globalThis as any).window;
            delete (globalThis as any).navigator;
            expect(await service.startScreenShare("auto")).toBeNull();
        } finally {
            (globalThis as any).window = originalWindowLocal;
            (globalThis as any).navigator = originalNavigatorLocal;
        }

        const stream = await service.startScreenShare("high");
        expect(stream).not.toBeNull();
        expect(service.getPreset()).toBe("high");

        await service.createOffer("target-1");
        const pc = MockRTCPeerConnection.instances[0];
        expect(pc.getSenders().length).toBeGreaterThan(0);

        pc.onicecandidate?.({
            candidate: { toJSON: () => ({ candidate: "ice" }) },
        });
        pc.onicecandidate?.({ candidate: null });
        pc.ontrack?.({ streams: [new MockMediaStream()] });
        pc.ontrack?.({ streams: [] });
        expect(onSignalCalls.length).toBeGreaterThan(0);
        expect(onStreamCalls.length).toBe(1);

        await (service as any).applyEncodingToPC(pc);

        service.stopScreenShare();
        expect(pc.closeCalled).toBe(true);
        const localTrack = (stream as any).getTracks()[0];
        expect(localTrack.stopped).toBe(true);

        (globalThis as any).navigator.mediaDevices.getDisplayMedia = async () => {
            throw new Error("denied");
        };
        expect(await service.startScreenShare("low")).toBeNull();

        const emptyService = new screenShare.ScreenShareService(() => {}, () => {});
        emptyService.stopScreenShare();
    });

    test("applies quality presets and handles sender/constraint failures", async () => {
        const service = new screenShare.ScreenShareService(() => {}, () => {});
        const stream = (await service.startScreenShare("auto")) as any;
        await service.createOffer("target-2");
        const pc = MockRTCPeerConnection.instances[0];
        const sender = pc.getSenders()[0];

        await service.applyQualityPreset("medium");
        expect(sender.params.encodings[0].maxBitrate).toBe(screenShare.QUALITY_PRESETS.medium.maxBitrate);
        expect(sender.params.encodings[0].scaleResolutionDownBy).toBeGreaterThanOrEqual(1);
        expect(sender.params.degradationPreference).toBe("maintain-resolution");

        await service.applyQualityPreset("auto");
        expect(sender.params.encodings[0].scaleResolutionDownBy).toBe(1);
        expect(sender.params.degradationPreference).toBe("balanced");

        sender.throwOnSet = true;
        await service.applyQualityPreset("low");
        sender.throwOnSet = false;

        const videoTrack = stream.getVideoTracks()[0];
        videoTrack.throwOnConstraints = true;
        await service.applyQualityPreset("high");
        videoTrack.throwOnConstraints = false;
    });

    test("collects network stats, including failure handling", async () => {
        const service = new screenShare.ScreenShareService(() => {}, () => {});

        expect(await service.getNetworkStats()).toBeNull();

        await service.startScreenShare("auto");
        await service.createOffer("target-3");
        const pc = MockRTCPeerConnection.instances[0];

        pc.statsReport = new Map([
            [
                "outbound",
                {
                    type: "outbound-rtp",
                    kind: "video",
                    bytesSent: 1000,
                    frameWidth: 1280,
                    frameHeight: 720,
                    framesPerSecond: 22,
                    qualityLimitationReason: "bandwidth",
                },
            ],
            [
                "remote",
                {
                    type: "remote-inbound-rtp",
                    kind: "video",
                    packetsLost: 1,
                    packetsReceived: 9,
                },
            ],
        ]);

        let now = 1000;
        Date.now = () => now;
        const first = await service.getNetworkStats();
        expect(first).toEqual({
            bitrate: 0,
            packetLoss: 10,
            resolution: "1280x720",
            frameRate: 22,
            qualityLimited: true,
        });

        now = 3000;
        pc.statsReport = new Map([
            [
                "outbound",
                {
                    type: "outbound-rtp",
                    kind: "video",
                    bytesSent: 3000,
                    frameWidth: 1280,
                    frameHeight: 720,
                    framesPerSecond: 20,
                    qualityLimitationReason: "none",
                },
            ],
            [
                "remote",
                {
                    type: "remote-inbound-rtp",
                    kind: "video",
                    packetsLost: 0,
                    packetsReceived: 10,
                },
            ],
        ]);
        const second = await service.getNetworkStats();
        expect(second?.bitrate).toBeGreaterThan(0);
        expect(second?.qualityLimited).toBe(false);

        pc.throwOnStats = true;
        expect(await service.getNetworkStats()).toBeNull();
    });

    test("starts and stops periodic stats monitoring", async () => {
        const service = new screenShare.ScreenShareService(() => {}, () => {});
        await service.startScreenShare("auto");
        await service.createOffer("target-4");
        const pc = MockRTCPeerConnection.instances[0];
        pc.statsReport = new Map();

        let intervalCallback: (() => void | Promise<void>) | null = null;
        globalThis.setInterval = ((cb: any) => {
            intervalCallback = cb;
            return 123 as any;
        }) as any;
        let cleared: number[] = [];
        globalThis.clearInterval = ((id: any) => {
            cleared.push(Number(id));
        }) as any;

        let callbackCount = 0;
        service.startStatsMonitoring(() => {
            callbackCount += 1;
        }, 50);

        const runInterval = intervalCallback as (() => void | Promise<void>) | null;
        if (!runInterval) throw new Error("missing interval callback");
        await runInterval();
        expect(callbackCount).toBe(1);

        pc.throwOnStats = true;
        await runInterval();
        expect(callbackCount).toBe(1);

        service.stopStatsMonitoring();
        expect(cleared).toContain(123);

        service.stopStatsMonitoring();
    });

    test("handles offer/answer/candidate signaling flows", async () => {
        const onSignalCalls: any[] = [];
        const service = new screenShare.ScreenShareService(
            (signal, targetId) => onSignalCalls.push({ signal, targetId }),
            () => {},
        );

        await service.createOffer("recv-target");
        const pc = MockRTCPeerConnection.instances[0];
        expect(pc.getTransceivers().length).toBe(1);
        await service.createOffer("recv-target");
        expect(pc.getTransceivers().length).toBe(1);

        await service.handleSignal(
            { type: "offer", sdp: { type: "offer", sdp: "sdp-offer" } },
            "recv-target",
        );
        expect(onSignalCalls.some((x) => x.signal.type === "answer")).toBe(true);

        await service.handleSignal(
            { type: "answer", sdp: { type: "answer", sdp: "sdp-answer" } },
            "recv-target",
        );
        expect(pc.remoteDescriptions.length).toBeGreaterThan(1);

        await service.handleSignal(
            { type: "candidate", candidate: { candidate: "ice-candidate" } },
            "recv-target",
        );
        expect(pc.iceCandidates.length).toBe(1);

        pc.forceStableOnOffer = true;
        await service.handleSignal(
            { type: "offer", sdp: { type: "offer", sdp: "sdp-no-answer" } },
            "recv-target",
        );
    });
});
