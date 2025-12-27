import CryptoJS from 'crypto-js';

// Secret key for AES encryption. 
// In a production environment with a backend, this should NOT be exposed to the client.
// However, for this requirement (client-side encryption for localStorage), 
// we use a hardcoded key to obfuscate the API key and prevent plain-text storage.
const SECRET_KEY = "Open-Codelabs-Secure-Salt-2025";

export function encrypt(text: string): string {
    if (!text) return "";
    return CryptoJS.AES.encrypt(text, SECRET_KEY).toString();
}

export function decrypt(ciphertext: string): string {
    if (!ciphertext) return "";
    try {
        const bytes = CryptoJS.AES.decrypt(ciphertext, SECRET_KEY);
        return bytes.toString(CryptoJS.enc.Utf8);
    } catch (e) {
        console.error("Decryption failed", e);
        return "";
    }
}
