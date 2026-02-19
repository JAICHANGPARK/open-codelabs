import CryptoJS from 'crypto-js';

// Secret key for AES encryption. 
// In a production environment with a backend, this should NOT be exposed to the client.
// However, for this requirement (client-side encryption for localStorage), 
// we use a hardcoded key to obfuscate the API key and prevent plain-text storage.
const SECRET_KEY = "Open-Codelabs-Secure-Salt-2025";

function getEnvEncryptionPassword(): string {
    return typeof import.meta !== "undefined" && (import.meta as any).env?.VITE_ADMIN_ENCRYPTION_PASSWORD
        ? String((import.meta as any).env.VITE_ADMIN_ENCRYPTION_PASSWORD)
        : "";
}

function hasBrowserStorage(): boolean {
    return typeof window !== "undefined" && typeof sessionStorage !== "undefined";
}

export function getEncryptionPassword(opts?: { interactive?: boolean }): string | null {
    const envPassword = getEnvEncryptionPassword();

    if (hasBrowserStorage()) {
        const stored = sessionStorage.getItem("adminPassword");
        if (stored) return stored;
        if (envPassword) {
            sessionStorage.setItem("adminPassword", envPassword);
            return envPassword;
        }
        if (opts?.interactive) {
            const pw = prompt("Admin password required for encryption:");
            if (pw) {
                sessionStorage.setItem("adminPassword", pw);
                return pw;
            }
        }
        return null;
    }

    return envPassword || null;
}

export function encrypt(text: string, key: string = SECRET_KEY): string {
    if (!text) return "";
    return CryptoJS.AES.encrypt(text, key).toString();
}

const BACKEND_PREFIX = "v1:";
const PBKDF2_ITERS = 100000;
const SALT_BYTES = 16;
const IV_BYTES = 16;
const ENC_KEY_BYTES = 32;
const MAC_KEY_BYTES = 32;

function deriveKeys(password: string, salt: CryptoJS.lib.WordArray) {
    const keySizeWords = (ENC_KEY_BYTES + MAC_KEY_BYTES) / 4;
    const derived = CryptoJS.PBKDF2(password, salt, {
        keySize: keySizeWords,
        iterations: PBKDF2_ITERS,
        hasher: CryptoJS.algo.SHA256
    });
    const encKey = CryptoJS.lib.WordArray.create(
        derived.words.slice(0, ENC_KEY_BYTES / 4),
        ENC_KEY_BYTES
    );
    const macKey = CryptoJS.lib.WordArray.create(
        derived.words.slice(ENC_KEY_BYTES / 4, keySizeWords),
        MAC_KEY_BYTES
    );
    return { encKey, macKey };
}

/**
 * Encrypts text for backend storage using AES-256-CBC + HMAC-SHA256.
 * Output format: v1:BASE64(salt || iv || ciphertext || tag)
 */
export function encryptForBackend(text: string, password: string): string {
    if (!text) return "";

    const salt = CryptoJS.lib.WordArray.random(SALT_BYTES);
    const iv = CryptoJS.lib.WordArray.random(IV_BYTES);
    const { encKey, macKey } = deriveKeys(password, salt);

    const encrypted = CryptoJS.AES.encrypt(text, encKey, {
        iv,
        mode: CryptoJS.mode.CBC,
        padding: CryptoJS.pad.Pkcs7
    });

    const data = salt.clone().concat(iv).concat(encrypted.ciphertext);
    const tag = CryptoJS.HmacSHA256(data, macKey);
    const payload = data.concat(tag);

    return `${BACKEND_PREFIX}${CryptoJS.enc.Base64.stringify(payload)}`;
}

export function decrypt(ciphertext: string, key: string = SECRET_KEY): string {
    if (!ciphertext) return "";
    try {
        const bytes = CryptoJS.AES.decrypt(ciphertext, key);
        return bytes.toString(CryptoJS.enc.Utf8);
    } catch (e) {
        console.error("Decryption failed", e);
        return "";
    }
}
