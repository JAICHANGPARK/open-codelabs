import { describe, expect, test } from "bun:test";
import { encrypt, decrypt, encryptForBackend } from "../crypto";

describe("crypto helpers", () => {
    test("encrypt/decrypt round trips with default key", () => {
        const plaintext = "secret-value";
        const cipher = encrypt(plaintext);
        expect(cipher).not.toBe("");
        const back = decrypt(cipher);
        expect(back).toBe(plaintext);
    });

    test("encryptForBackend matches backend magic-crypt output", () => {
        // Mirrors backend/src/handlers/admin.rs test
        const password = "admin";
        const plaintext = "secret-api-key";
        const cipher = encryptForBackend(plaintext, password);
        expect(cipher).toBe("URh6eeDLAKxc2nYWOrhyjg==");
    });
});
