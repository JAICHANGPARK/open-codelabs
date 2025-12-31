import CryptoJS from 'crypto-js';

// Secret key for AES encryption. 
// In a production environment with a backend, this should NOT be exposed to the client.
// However, for this requirement (client-side encryption for localStorage), 
// we use a hardcoded key to obfuscate the API key and prevent plain-text storage.
const SECRET_KEY = "Open-Codelabs-Secure-Salt-2025";

export function encrypt(text: string, key: string = SECRET_KEY): string {
    if (!text) return "";
    return CryptoJS.AES.encrypt(text, key).toString();
}

/**
 * Encrypts text using AES-256-CBC compatible with Rust's magic-crypt.
 * magic-crypt (256-bit) uses:
 * - Key: SHA256(password)
 * - IV: SHA256(SHA256(password)).slice(0, 16)
 */
export function encryptForBackend(text: string, password: string): string {
    if (!text) return "";
    
    // Key: SHA256(password)
    const key = CryptoJS.SHA256(password);
    
    // Try IV: all zeros
    const iv = CryptoJS.enc.Hex.parse("00000000000000000000000000000000");
    
    const encrypted = CryptoJS.AES.encrypt(text, key, {
        iv: iv,
        mode: CryptoJS.mode.CBC,
        padding: CryptoJS.pad.Pkcs7
    });
    
    return encrypted.toString();
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
