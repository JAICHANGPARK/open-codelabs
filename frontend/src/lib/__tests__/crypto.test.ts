import { describe, expect, test } from "bun:test";
import CryptoJS from "crypto-js";
import { decrypt, encrypt, encryptForBackend, getEncryptionPassword } from "../crypto";

function createSessionStorage(initial: Record<string, string> = {}) {
    const store: Record<string, string> = { ...initial };
    return {
        getItem(key: string) {
            return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
        },
        setItem(key: string, value: string) {
            store[key] = String(value);
        },
        clear() {
            for (const key of Object.keys(store)) delete store[key];
        },
    };
}

describe("crypto helpers", () => {
    test("encrypt/decrypt round trips with default key", () => {
        const plaintext = "secret-value";
        const cipher = encrypt(plaintext);
        expect(cipher).not.toBe("");
        const back = decrypt(cipher);
        expect(back).toBe(plaintext);
    });

    test("encryptForBackend produces a versioned payload", () => {
        const password = "admin";
        const plaintext = "secret-api-key";
        const cipher = encryptForBackend(plaintext, password);
        expect(cipher.startsWith("v1:")).toBe(true);
        expect(cipher.length).toBeGreaterThan(3);
    });

    test("returns empty string for empty inputs", () => {
        expect(encrypt("")).toBe("");
        expect(decrypt("")).toBe("");
        expect(encryptForBackend("", "admin")).toBe("");
    });

    test("decrypt handles CryptoJS errors gracefully", () => {
        const originalDecrypt = (CryptoJS.AES as any).decrypt;
        const originalConsoleError = console.error;
        let logged = false;
        try {
            (CryptoJS.AES as any).decrypt = () => {
                throw new Error("boom");
            };
            (console as any).error = () => {
                logged = true;
            };
            expect(decrypt("invalid-ciphertext")).toBe("");
            expect(logged).toBe(true);
        } finally {
            (CryptoJS.AES as any).decrypt = originalDecrypt;
            (console as any).error = originalConsoleError;
        }
    });

    test("getEncryptionPassword returns null on server without env password", () => {
        const originalEnv = process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
        const originalWindow = (globalThis as any).window;
        const originalSessionStorage = (globalThis as any).sessionStorage;
        const originalPrompt = (globalThis as any).prompt;
        try {
            delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            delete (globalThis as any).window;
            delete (globalThis as any).sessionStorage;
            delete (globalThis as any).prompt;
            expect(getEncryptionPassword()).toBeNull();
        } finally {
            if (originalEnv === undefined) delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            else process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = originalEnv;
            (globalThis as any).window = originalWindow;
            (globalThis as any).sessionStorage = originalSessionStorage;
            (globalThis as any).prompt = originalPrompt;
        }
    });

    test("getEncryptionPassword prefers a stored browser password", () => {
        const originalEnv = process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
        const originalWindow = (globalThis as any).window;
        const originalSessionStorage = (globalThis as any).sessionStorage;
        const originalPrompt = (globalThis as any).prompt;
        try {
            process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "env-pw";
            (globalThis as any).window = {};
            (globalThis as any).sessionStorage = createSessionStorage({ adminPassword: "stored-pw" });
            (globalThis as any).prompt = () => "ignored";
            expect(getEncryptionPassword()).toBe("stored-pw");
        } finally {
            if (originalEnv === undefined) delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            else process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = originalEnv;
            (globalThis as any).window = originalWindow;
            (globalThis as any).sessionStorage = originalSessionStorage;
            (globalThis as any).prompt = originalPrompt;
        }
    });

    test("getEncryptionPassword uses env password and caches it in browser session", () => {
        const originalEnv = process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
        const originalWindow = (globalThis as any).window;
        const originalSessionStorage = (globalThis as any).sessionStorage;
        const originalPrompt = (globalThis as any).prompt;
        try {
            process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "env-only";
            const sessionStorage = createSessionStorage();
            (globalThis as any).window = {};
            (globalThis as any).sessionStorage = sessionStorage;
            (globalThis as any).prompt = () => "ignored";
            expect(getEncryptionPassword()).toBe("env-only");
            expect((globalThis as any).sessionStorage.getItem("adminPassword")).toBe("env-only");
        } finally {
            if (originalEnv === undefined) delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            else process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = originalEnv;
            (globalThis as any).window = originalWindow;
            (globalThis as any).sessionStorage = originalSessionStorage;
            (globalThis as any).prompt = originalPrompt;
        }
    });

    test("getEncryptionPassword prompts interactively when requested in browser", () => {
        const originalEnv = process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
        const originalWindow = (globalThis as any).window;
        const originalSessionStorage = (globalThis as any).sessionStorage;
        const originalPrompt = (globalThis as any).prompt;
        try {
            delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            (globalThis as any).window = {};
            (globalThis as any).sessionStorage = createSessionStorage();
            (globalThis as any).prompt = () => "typed-pw";
            expect(getEncryptionPassword({ interactive: true })).toBe("typed-pw");
            expect((globalThis as any).sessionStorage.getItem("adminPassword")).toBe("typed-pw");
        } finally {
            if (originalEnv === undefined) delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            else process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = originalEnv;
            (globalThis as any).window = originalWindow;
            (globalThis as any).sessionStorage = originalSessionStorage;
            (globalThis as any).prompt = originalPrompt;
        }
    });

    test("getEncryptionPassword returns null when interactive prompt is canceled", () => {
        const originalEnv = process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
        const originalWindow = (globalThis as any).window;
        const originalSessionStorage = (globalThis as any).sessionStorage;
        const originalPrompt = (globalThis as any).prompt;
        try {
            delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            (globalThis as any).window = {};
            (globalThis as any).sessionStorage = createSessionStorage();
            (globalThis as any).prompt = () => "";
            expect(getEncryptionPassword({ interactive: true })).toBeNull();
        } finally {
            if (originalEnv === undefined) delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
            else process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = originalEnv;
            (globalThis as any).window = originalWindow;
            (globalThis as any).sessionStorage = originalSessionStorage;
            (globalThis as any).prompt = originalPrompt;
        }
    });
});
