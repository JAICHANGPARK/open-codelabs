import { browser } from '$app/environment';
import { encryptForBackend, getEncryptionPassword } from './crypto';

export interface GeminiConfig {
    apiKey: string;
    model?: string;
}

export interface GeminiStructuredConfig extends GeminiConfig {
    tools?: Array<{ googleSearch?: {} } | { urlContext?: {} }>;
    thinkingConfig?: { thinkingLevel: "low" | "medium" | "high" };
}

export interface ThinkingContent {
    thinking?: string;
    content: string;
}

const envApiUrl = import.meta.env.VITE_API_URL;
let BASE_URL = envApiUrl || 'http://localhost:8080';
const USE_FIREBASE = import.meta.env.VITE_USE_FIREBASE === 'true';

if (browser && (envApiUrl === 'http://backend:8080' || !envApiUrl || envApiUrl.includes('localhost'))) {
    const hostname = window.location.hostname;
    const isTunnelHost = hostname.includes('ngrok') || hostname.includes('bore') || hostname.includes('trycloudflare.com');
    const isDefaultPort = window.location.port === '' || window.location.port === '443' || window.location.port === '80';
    const isLocalhost = hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '::1';

    if (isTunnelHost || (!isLocalhost && isDefaultPort)) {
        BASE_URL = window.location.origin;
    } else {
        BASE_URL = `${window.location.protocol}//${window.location.hostname}:8080`;
    }
}

const AI_PROXY_URL = `${BASE_URL}/api/ai/stream`;

function getCookie(name: string): string | null {
    if (!browser) return null;
    const match = document.cookie.match(new RegExp(`(?:^|; )${name}=([^;]*)`));
    return match ? decodeURIComponent(match[1]) : null;
}

function getCsrfToken(): string | null {
    return getCookie("__Host-oc_csrf") || getCookie("oc_csrf");
}

function withCsrf(headers?: HeadersInit): Headers {
    const merged = new Headers(headers || {});
    const token = getCsrfToken();
    if (token) merged.set("X-CSRF-Token", token);
    return merged;
}

export async function* streamGeminiResponseRobust(
    prompt: string,
    context: string,
    config: GeminiConfig
): AsyncGenerator<string, void, unknown> {
    const apiKeyRequired = () => {
        if (!config.apiKey) throw new Error("API Key is required for backend mode");
    };

    if (USE_FIREBASE) {
        if (!config.apiKey) throw new Error("API Key is required for Firebase mode");
        // Direct call for Firebase mode
        const model = config.model || "gemini-3-flash-preview";
        // alt=sse is required to use parseGoogleStream logic
        const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?alt=sse&key=${config.apiKey}`;

        const payload = {
            contents: [{ role: "user", parts: [{ text: `Context:\n${context}\n\nQuestion:\n${prompt}` }] }]
        };

        const response = await fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
        });

        if (!response.ok) throw new Error(`API Error: ${response.status}`);
        yield* parseGoogleStream(response);
    } else {
        // Proxy through our backend
        apiKeyRequired();
        let apiKey = config.apiKey;
        if (browser) {
            const adminPw = getEncryptionPassword({ interactive: false });
            if (adminPw) {
                apiKey = encryptForBackend(apiKey, adminPw);
            }
        } else {
            const adminPw = getEncryptionPassword();
            if (adminPw) {
                apiKey = encryptForBackend(apiKey, adminPw);
            }
        }

        const response = await fetch(AI_PROXY_URL, {
            method: "POST",
            headers: withCsrf({ "Content-Type": "application/json" }),
            credentials: "include",
            body: JSON.stringify({
                prompt: `Context:\n${context}\n\nQuestion:\n${prompt}`,
                api_key: apiKey || undefined, // Send encrypted if we have it
                model: config.model || "gemini-3-flash-preview"
            }),
        });

        if (!response.ok) throw new Error(`Backend AI Error: ${response.status}`);
        yield* parseGoogleStream(response);
    }
}

async function* parseGoogleStream(response: Response) {
    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });

        // Backend passes data: {...} lines
        const lines = buffer.split("\n");
        buffer = lines.pop() || ""; // keep last incomplete line

        for (const line of lines) {
            if (!line.startsWith("data:")) continue;
            const jsonStr = line.replace("data:", "").trim();
            if (!jsonStr) continue;

            try {
                const data = JSON.parse(jsonStr);
                // Handle different response formats (backend proxy might return direct part or full candidate)
                const candidate = data.candidates?.[0];
                if (candidate?.content?.parts?.[0]?.text) {
                    yield candidate.content.parts[0].text;
                }
            } catch (e) {
                // ignore
            }
        }
    }
}

export async function* streamGeminiStructuredOutput(
    prompt: string,
    systemPrompt: string,
    schema: object,
    config: GeminiStructuredConfig
): AsyncGenerator<ThinkingContent, void, unknown> {
    const apiKeyRequired = () => {
        if (!config.apiKey) throw new Error("API Key is required for backend mode");
    };

    const generationConfig = {
        responseMimeType: "application/json",
        responseJsonSchema: schema,
        ...(config.thinkingConfig && {
            thinkingConfig: {
                thinkingLevel: config.thinkingConfig.thinkingLevel
            }
        })
    };

    if (USE_FIREBASE) {
        if (!config.apiKey) throw new Error("API Key is required for Firebase mode");
        const model = config.model || "gemini-3-flash-preview";
        const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?alt=sse&key=${config.apiKey}`;

        const payload: any = {
            contents: [
                {
                    role: "user",
                    parts: [{ text: `${systemPrompt}\n\n${prompt}` }]
                }
            ],
            generationConfig
        };

        if (config.tools && config.tools.length > 0) {
            payload.tools = config.tools;
        }

        const response = await fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
        });

        if (!response.ok) throw new Error(`API Error ${response.status}`);
        yield* parseStructuredStream(response);
    } else {
        apiKeyRequired();
        let apiKey = config.apiKey;
        if (browser) {
            const adminPw = getEncryptionPassword({ interactive: false });
            if (adminPw) {
                apiKey = encryptForBackend(apiKey, adminPw);
            }
        } else {
            const adminPw = getEncryptionPassword();
            if (adminPw) {
                apiKey = encryptForBackend(apiKey, adminPw);
            }
        }

        const response = await fetch(AI_PROXY_URL, {
            method: "POST",
            headers: withCsrf({ "Content-Type": "application/json" }),
            credentials: "include",
            body: JSON.stringify({
                prompt: prompt,
                system_instruction: systemPrompt,
                api_key: apiKey || undefined,
                model: config.model || "gemini-3-flash-preview",
                generation_config: generationConfig,
                tools: config.tools
            }),
        });

        if (!response.ok) throw new Error(`Backend AI Error: ${response.status}`);
        yield* parseStructuredStream(response);
    }
}

async function* parseStructuredStream(response: Response) {
    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";

        for (const line of lines) {
            if (!line.startsWith("data:")) continue;
            const jsonStr = line.replace("data:", "").trim();
            if (!jsonStr) continue;

            try {
                const data = JSON.parse(jsonStr);
                const candidate = data.candidates?.[0];
                if (candidate?.content?.parts) {
                    let thinkingText = "";
                    let contentText = "";
                    for (const part of candidate.content.parts) {
                        if (part.thought) thinkingText += part.thought;
                        else if (part.text) contentText += part.text;
                    }
                    if (thinkingText || contentText) {
                        yield {
                            thinking: thinkingText || undefined,
                            content: contentText
                        };
                    }
                }
            } catch (e) { }
        }
    }
}
