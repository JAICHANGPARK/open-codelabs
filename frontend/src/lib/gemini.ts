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

export interface TokenUsage {
    promptTokenCount: number;
    candidatesTokenCount: number;
    totalTokenCount: number;
}

export interface GenerationResult {
    content: string;
    usage?: TokenUsage;
}

export interface GeminiResponseChunk {
    text?: string;
    groundingMetadata?: any;
    usageMetadata?: TokenUsage;
}

const envApiUrl = import.meta.env.VITE_API_URL;
const isBrowser = () => typeof window !== "undefined" && typeof document !== "undefined";
type LocationLike = {
    hostname: string;
    port: string;
    origin: string;
    protocol: string;
};

export function resolveGeminiBaseUrl(apiUrl?: string, location?: LocationLike): string {
    let baseUrl = apiUrl || 'http://localhost:8080';
    if (!location) return baseUrl;
    if (!(apiUrl === 'http://backend:8080' || !apiUrl || apiUrl.includes('localhost'))) return baseUrl;

    const hostname = location.hostname;
    const isTunnelHost = hostname.includes('ngrok') || hostname.includes('bore') || hostname.includes('trycloudflare.com');
    const isDefaultPort = location.port === '' || location.port === '443' || location.port === '80';
    const isLocalhost = hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '::1';

    if (isTunnelHost || (!isLocalhost && isDefaultPort)) {
        return location.origin;
    }
    return `${location.protocol}//${location.hostname}:8080`;
}

const BASE_URL = resolveGeminiBaseUrl(envApiUrl, isBrowser() ? window.location : undefined);

const AI_PROXY_URL = `${BASE_URL}/api/ai/stream`;

function isServerlessMode() {
    const useSupabase = import.meta.env.VITE_USE_SUPABASE === "true";
    const useFirebase = import.meta.env.VITE_USE_FIREBASE === "true" && !useSupabase;
    return useFirebase || useSupabase;
}

function getCookie(name: string): string | null {
    if (!isBrowser()) return null;
    const match = document.cookie.match(new RegExp(`(?:^|; )${name}=([^;]*)`));
    return match ? decodeURIComponent(match[1]) : null;
}

function getCsrfToken(): string | null {
    return getCookie("__Host-oc_csrf") || getCookie("oc_csrf");
}

export function withCsrf(headers?: HeadersInit): Headers {
    const merged = new Headers(headers || {});
    const token = getCsrfToken();
    if (token) merged.set("X-CSRF-Token", token);
    return merged;
}

export async function* streamGeminiResponseRobust(
    prompt: string,
    context: string,
    config: GeminiStructuredConfig
): AsyncGenerator<GeminiResponseChunk, void, unknown> {
    const apiKeyRequired = () => {
        if (!config.apiKey) throw new Error("API Key is required for backend mode");
    };

    if (isServerlessMode()) {
        if (!config.apiKey) throw new Error("API Key is required for serverless mode");
        // Direct call for serverless mode
        const model = config.model || "gemini-3-flash-preview";
        // alt=sse is required to use parseGoogleStream logic
        const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?alt=sse&key=${config.apiKey}`;

        const payload: any = {
            contents: [{ role: "user", parts: [{ text: `Context:\n${context}\n\nQuestion:\n${prompt}` }] }]
        };

        if (config.tools && config.tools.length > 0) {
            payload.tools = config.tools;
        }

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
        if (isBrowser()) {
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
                model: config.model || "gemini-3-flash-preview",
                tools: config.tools
            }),
        });

        if (!response.ok) throw new Error(`Backend AI Error: ${response.status}`);
        yield* parseGoogleStream(response);
    }
}

export async function* streamGeminiChat(
    messages: Array<{ role: "user" | "model" | "assistant", content: string }>,
    systemPrompt: string,
    config: GeminiStructuredConfig
): AsyncGenerator<GeminiResponseChunk, TokenUsage | undefined, unknown> {
    const apiKeyRequired = () => {
        if (!config.apiKey) throw new Error("API Key is required");
    };

    // Normalize roles for Gemini (assistant -> model)
    const contents = messages.map(m => ({
        role: m.role === "assistant" ? "model" : m.role,
        parts: [{ text: m.content }]
    }));

    if (isServerlessMode()) {
        apiKeyRequired();
        const model = config.model || "gemini-3-flash-preview";
        const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?alt=sse&key=${config.apiKey}`;

        const payload: any = {
            system_instruction: { parts: [{ text: systemPrompt }] },
            contents
        };

        if (config.tools && config.tools.length > 0) {
            payload.tools = config.tools;
        }

        const response = await fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
        });

        if (!response.ok) throw new Error(`API Error: ${response.status}`);
        const result = yield* parseGoogleStream(response);
        return result;
    } else {
        apiKeyRequired();
        let apiKey = config.apiKey;
        if (isBrowser()) {
            const adminPw = getEncryptionPassword({ interactive: false });
            if (adminPw) apiKey = encryptForBackend(apiKey, adminPw);
        }

        const response = await fetch(AI_PROXY_URL, {
            method: "POST",
            headers: withCsrf({ "Content-Type": "application/json" }),
            credentials: "include",
            body: JSON.stringify({
                system_instruction: systemPrompt,
                contents,
                api_key: apiKey || undefined,
                model: config.model || "gemini-3-flash-preview",
                tools: config.tools
            }),
        });

        if (!response.ok) throw new Error(`Backend AI Error: ${response.status}`);
        const result = yield* parseGoogleStream(response);
        return result;
    }
}

async function* parseGoogleStream(response: Response): AsyncGenerator<GeminiResponseChunk, TokenUsage | undefined, unknown> {
    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    let lastUsage: TokenUsage | undefined;

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

                // Capture usage metadata
                if (data.usageMetadata) {
                    lastUsage = {
                        promptTokenCount: data.usageMetadata.promptTokenCount || 0,
                        candidatesTokenCount: data.usageMetadata.candidatesTokenCount || 0,
                        totalTokenCount: data.usageMetadata.totalTokenCount || 0,
                    };
                }

                // Handle different response formats (backend proxy might return direct part or full candidate)
                const candidate = data.candidates?.[0];
                if (candidate?.content?.parts?.[0]?.text) {
                    yield { text: candidate.content.parts[0].text };
                }

                if (candidate?.groundingMetadata) {
                    yield { groundingMetadata: candidate.groundingMetadata };
                }

                if (data.usageMetadata) {
                    yield { usageMetadata: lastUsage };
                }
            } catch (e) {
                // ignore
            }
        }
    }

    return lastUsage;
}

export async function* streamGeminiStructuredOutput(
    prompt: string,
    systemPrompt: string,
    schema: object,
    config: GeminiStructuredConfig
): AsyncGenerator<ThinkingContent, TokenUsage | undefined, unknown> {
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

    if (isServerlessMode()) {
        if (!config.apiKey) throw new Error("API Key is required for serverless mode");
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
        const result = yield* parseStructuredStream(response);
        return result;
    } else {
        apiKeyRequired();
        let apiKey = config.apiKey;
        if (isBrowser()) {
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
        const result = yield* parseStructuredStream(response);
        return result;
    }
}

async function* parseStructuredStream(response: Response): AsyncGenerator<ThinkingContent, TokenUsage | undefined, unknown> {
    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    let lastUsage: TokenUsage | undefined;

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

                // Capture usage metadata
                if (data.usageMetadata) {
                    lastUsage = {
                        promptTokenCount: data.usageMetadata.promptTokenCount || 0,
                        candidatesTokenCount: data.usageMetadata.candidatesTokenCount || 0,
                        totalTokenCount: data.usageMetadata.totalTokenCount || 0,
                    };
                }

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

    return lastUsage;
}
