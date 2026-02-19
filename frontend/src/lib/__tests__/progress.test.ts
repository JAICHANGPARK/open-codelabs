import { describe, expect, test } from "bun:test";
import { loadProgress, saveProgress } from "../Progress";

function createStorage(initial: Record<string, string> = {}) {
    const store: Record<string, string> = { ...initial };
    return {
        getItem(key: string) {
            return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
        },
        setItem(key: string, value: string) {
            store[key] = String(value);
        },
    };
}

describe("Progress storage helpers", () => {
    test("saveProgress and loadProgress use localStorage when available", () => {
        const original = (globalThis as any).localStorage;
        try {
            (globalThis as any).localStorage = createStorage();
            saveProgress("lab-1", 3);
            expect(loadProgress("lab-1")).toBe(3);
        } finally {
            (globalThis as any).localStorage = original;
        }
    });

    test("loadProgress returns 0 when key does not exist", () => {
        const original = (globalThis as any).localStorage;
        try {
            (globalThis as any).localStorage = createStorage();
            expect(loadProgress("missing")).toBe(0);
        } finally {
            (globalThis as any).localStorage = original;
        }
    });

    test("returns defaults safely when localStorage is unavailable", () => {
        const original = (globalThis as any).localStorage;
        try {
            delete (globalThis as any).localStorage;
            expect(() => saveProgress("lab-2", 10)).not.toThrow();
            expect(loadProgress("lab-2")).toBe(0);
        } finally {
            (globalThis as any).localStorage = original;
        }
    });
});

