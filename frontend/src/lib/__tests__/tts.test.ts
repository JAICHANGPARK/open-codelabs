import { describe, expect, test } from "bun:test";
import { createTtsPlayer } from "../tts";

describe("createTtsPlayer", () => {
    test("is a safe no-op when speech synthesis is unavailable", () => {
        const originalWindow = (globalThis as any).window;
        try {
            delete (globalThis as any).window;
            const player = createTtsPlayer();
            expect(() => player.speak("hello")).not.toThrow();
            expect(() => player.pause()).not.toThrow();
            expect(() => player.resume()).not.toThrow();
            expect(() => player.stop()).not.toThrow();
        } finally {
            (globalThis as any).window = originalWindow;
        }
    });

    test("speaks stripped text and maps common locale codes", () => {
        const originalWindow = (globalThis as any).window;
        const originalUtterance = (globalThis as any).SpeechSynthesisUtterance;
        const calls: any = {
            spoken: [] as Array<{ text: string; lang: string; rate: number; pitch: number }>,
            cancel: 0,
            pause: 0,
            resume: 0,
        };

        class MockUtterance {
            text: string;
            lang = "";
            rate = 0;
            pitch = 0;
            constructor(text: string) {
                this.text = text;
            }
        }

        try {
            (globalThis as any).SpeechSynthesisUtterance = MockUtterance;
            (globalThis as any).window = {
                speechSynthesis: {
                    speak(utterance: any) {
                        calls.spoken.push({
                            text: utterance.text,
                            lang: utterance.lang,
                            rate: utterance.rate,
                            pitch: utterance.pitch,
                        });
                    },
                    cancel() {
                        calls.cancel += 1;
                    },
                    pause() {
                        calls.pause += 1;
                    },
                    resume() {
                        calls.resume += 1;
                    },
                },
            };

            const player = createTtsPlayer();
            player.speak("<b>hello</b>", "ko");
            player.speak("bonjour", "fr");
            player.pause();
            player.resume();
            player.stop();

            expect(calls.cancel).toBe(3);
            expect(calls.pause).toBe(1);
            expect(calls.resume).toBe(1);

            expect(calls.spoken[0]).toEqual({
                text: "hello",
                lang: "ko-KR",
                rate: 1,
                pitch: 1,
            });
            expect(calls.spoken[1]).toEqual({
                text: "bonjour",
                lang: "fr",
                rate: 1,
                pitch: 1,
            });
        } finally {
            (globalThis as any).window = originalWindow;
            (globalThis as any).SpeechSynthesisUtterance = originalUtterance;
        }
    });
});

