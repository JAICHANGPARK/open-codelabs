import { afterAll, beforeAll, beforeEach, describe, expect, test } from "bun:test";

type GeminiModule = typeof import("../gemini");

type FetchCall = {
    url: string;
    init: RequestInit;
};

const calls: FetchCall[] = [];
const queue: Response[] = [];

const originalFetch = globalThis.fetch;
const originalWindow = (globalThis as any).window;
const originalDocument = (globalThis as any).document;
const originalSessionStorage = (globalThis as any).sessionStorage;
const originalUseFirebase = process.env.VITE_USE_FIREBASE;
const originalUseSupabase = process.env.VITE_USE_SUPABASE;
const originalEncPw = process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;

let gemini: GeminiModule;

function makeSessionStorage(initial: Record<string, string> = {}) {
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

function enqueue(response: Response) {
    queue.push(response);
}

function makeSseResponse(chunks: string[], status = 200) {
    const encoder = new TextEncoder();
    const stream = new ReadableStream<Uint8Array>({
        start(controller) {
            for (const chunk of chunks) {
                controller.enqueue(encoder.encode(chunk));
            }
            controller.close();
        },
    });
    return new Response(stream, {
        status,
        headers: { "Content-Type": "text/event-stream" },
    });
}

async function collectStream<T, R>(gen: AsyncGenerator<T, R, unknown>) {
    const items: T[] = [];
    while (true) {
        const next = await gen.next();
        if (next.done) return { items, result: next.value };
        items.push(next.value);
    }
}

function parseRequestBody(index: number) {
    const raw = calls[index]?.init.body;
    return raw ? JSON.parse(String(raw)) : null;
}

describe("gemini helpers", () => {
    beforeAll(async () => {
        (globalThis as any).window = {
            location: {
                hostname: "demo.ngrok.io",
                port: "",
                origin: "https://demo.ngrok.io",
                protocol: "https:",
            },
        };
        (globalThis as any).document = {
            cookie: "__Host-oc_csrf=csrf-token; oc_csrf=fallback",
        };
        (globalThis as any).sessionStorage = makeSessionStorage({ adminPassword: "admin-pw" });

        globalThis.fetch = async (input: RequestInfo | URL, init: RequestInit = {}) => {
            calls.push({ url: String(input), init });
            const next = queue.shift();
            if (!next) throw new Error(`No queued response for ${String(input)}`);
            return next;
        };

        process.env.VITE_USE_FIREBASE = "false";
        process.env.VITE_USE_SUPABASE = "false";
        process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "";
        gemini = await import("../gemini");
    });

    beforeEach(() => {
        calls.length = 0;
        queue.length = 0;
        (globalThis as any).window = {
            location: {
                hostname: "demo.ngrok.io",
                port: "",
                origin: "https://demo.ngrok.io",
                protocol: "https:",
            },
        };
        (globalThis as any).document = {
            cookie: "__Host-oc_csrf=csrf-token; oc_csrf=fallback",
        };
        (globalThis as any).sessionStorage = makeSessionStorage({ adminPassword: "admin-pw" });
        process.env.VITE_USE_FIREBASE = "false";
        process.env.VITE_USE_SUPABASE = "false";
        process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "";
    });

    afterAll(() => {
        globalThis.fetch = originalFetch;
        (globalThis as any).window = originalWindow;
        (globalThis as any).document = originalDocument;
        (globalThis as any).sessionStorage = originalSessionStorage;
        if (originalUseFirebase === undefined) delete process.env.VITE_USE_FIREBASE;
        else process.env.VITE_USE_FIREBASE = originalUseFirebase;
        if (originalUseSupabase === undefined) delete process.env.VITE_USE_SUPABASE;
        else process.env.VITE_USE_SUPABASE = originalUseSupabase;
        if (originalEncPw === undefined) delete process.env.VITE_ADMIN_ENCRYPTION_PASSWORD;
        else process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = originalEncPw;
    });

    test("resolves base URL variants and merges csrf headers", () => {
        expect(
            gemini.resolveGeminiBaseUrl("https://api.example.com", {
                hostname: "x.ngrok.io",
                port: "",
                origin: "https://x.ngrok.io",
                protocol: "https:",
            }),
        ).toBe("https://api.example.com");

        expect(
            gemini.resolveGeminiBaseUrl(undefined, {
                hostname: "x.ngrok.io",
                port: "",
                origin: "https://x.ngrok.io",
                protocol: "https:",
            }),
        ).toBe("https://x.ngrok.io");

        expect(
            gemini.resolveGeminiBaseUrl(undefined, {
                hostname: "localhost",
                port: "5173",
                origin: "http://localhost:5173",
                protocol: "http:",
            }),
        ).toBe("http://localhost:8080");

        expect(
            gemini.resolveGeminiBaseUrl(undefined, {
                hostname: "192.168.0.10",
                port: "3000",
                origin: "http://192.168.0.10:3000",
                protocol: "http:",
            }),
        ).toBe("http://192.168.0.10:8080");

        const withToken = gemini.withCsrf({ "Content-Type": "application/json" });
        expect(withToken.get("Content-Type")).toBe("application/json");
        expect(withToken.get("X-CSRF-Token")).toBe("csrf-token");

        const originalWindowLocal = (globalThis as any).window;
        const originalDocumentLocal = (globalThis as any).document;
        try {
            delete (globalThis as any).window;
            delete (globalThis as any).document;
            const withoutToken = gemini.withCsrf();
            expect(withoutToken.get("X-CSRF-Token")).toBeNull();
        } finally {
            (globalThis as any).window = originalWindowLocal;
            (globalThis as any).document = originalDocumentLocal;
        }
    });

    test("streams robust response in backend mode with encrypted api key", async () => {
        enqueue(
            makeSseResponse([
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"hello\"}]}}]}\n",
                "data: {\"candidates\":[{\"groundingMetadata\":{\"source\":\"web\"}}]}\n",
                "data: {\"usageMetadata\":{\"promptTokenCount\":1,\"candidatesTokenCount\":2,\"totalTokenCount\":3}}\n",
                "data: {bad-json}\n",
            ]),
        );

        const stream = gemini.streamGeminiResponseRobust("Q?", "CTX", {
            apiKey: "raw-key",
            model: "gemini-test",
            tools: [{ googleSearch: {} }],
        });
        const { items } = await collectStream(stream);

        expect(items).toEqual([
            { text: "hello" },
            { groundingMetadata: { source: "web" } },
            {
                usageMetadata: {
                    promptTokenCount: 1,
                    candidatesTokenCount: 2,
                    totalTokenCount: 3,
                },
            },
        ]);

        expect(calls[0]?.url).toContain("/api/ai/stream");
        const headers = new Headers(calls[0]?.init.headers);
        expect(headers.get("X-CSRF-Token")).toBe("csrf-token");

        const body = parseRequestBody(0);
        expect(body.prompt).toContain("Context:\nCTX\n\nQuestion:\nQ?");
        expect(body.api_key.startsWith("v1:")).toBe(true);
        expect(body.model).toBe("gemini-test");
    });

    test("handles backend/serverless mode validation and request routing", async () => {
        await expect(
            collectStream(
                gemini.streamGeminiResponseRobust("Q", "C", {
                    apiKey: "",
                }),
            ),
        ).rejects.toThrow("API Key is required for backend mode");

        const originalWindowLocal = (globalThis as any).window;
        const originalDocumentLocal = (globalThis as any).document;
        try {
            delete (globalThis as any).window;
            delete (globalThis as any).document;
            process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "node-admin-pw";

            enqueue(makeSseResponse(["data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"server\"}]}}]}\n"]));
            const backendServer = await collectStream(
                gemini.streamGeminiResponseRobust("Q", "C", {
                    apiKey: "plain-key",
                }),
            );
            expect(backendServer.items).toEqual([{ text: "server" }]);
            const backendBody = parseRequestBody(0);
            expect(String(backendBody.api_key).startsWith("v1:")).toBe(true);
        } finally {
            (globalThis as any).window = originalWindowLocal;
            (globalThis as any).document = originalDocumentLocal;
            process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "";
        }

        process.env.VITE_USE_FIREBASE = "true";
        process.env.VITE_USE_SUPABASE = "false";

        enqueue(makeSseResponse(["data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"serverless\"}]}}]}\n"]));
        const serverless = await collectStream(
            gemini.streamGeminiResponseRobust("Q", "C", {
                apiKey: "k1",
                tools: [{ urlContext: {} }],
            }),
        );
        expect(serverless.items).toEqual([{ text: "serverless" }]);
        expect(calls[calls.length - 1]?.url).toContain("generativelanguage.googleapis.com");
        expect(calls[calls.length - 1]?.url).toContain("key=k1");

        await expect(
            collectStream(
                gemini.streamGeminiResponseRobust("Q", "C", {
                    apiKey: "",
                }),
            ),
        ).rejects.toThrow("API Key is required for serverless mode");
    });

    test("streams chat response and returns usage metadata", async () => {
        process.env.VITE_USE_FIREBASE = "false";
        process.env.VITE_USE_SUPABASE = "false";

        enqueue(
            makeSseResponse([
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"chat-1\"}]}}]}\n",
                "data: {\"usageMetadata\":{\"promptTokenCount\":10,\"candidatesTokenCount\":20,\"totalTokenCount\":30}}\n",
            ]),
        );

        const backendChat = await collectStream(
            gemini.streamGeminiChat(
                [
                    { role: "user", content: "hello" },
                    { role: "assistant", content: "previous" },
                ],
                "system prompt",
                { apiKey: "backend-chat-key" },
            ),
        );

        expect(backendChat.items).toEqual([
            { text: "chat-1" },
            {
                usageMetadata: {
                    promptTokenCount: 10,
                    candidatesTokenCount: 20,
                    totalTokenCount: 30,
                },
            },
        ]);
        expect(backendChat.result).toEqual({
            promptTokenCount: 10,
            candidatesTokenCount: 20,
            totalTokenCount: 30,
        });

        const backendBody = parseRequestBody(0);
        expect(backendBody.contents[1].role).toBe("model");
        expect(backendBody.api_key.startsWith("v1:")).toBe(true);

        process.env.VITE_USE_FIREBASE = "true";
        process.env.VITE_USE_SUPABASE = "false";
        enqueue(makeSseResponse(["data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"chat-serverless\"}]}}]}\n"]));
        const serverlessChat = await collectStream(
            gemini.streamGeminiChat(
                [{ role: "user", content: "hello" }],
                "sys",
                { apiKey: "serverless-key", tools: [{ googleSearch: {} }] },
            ),
        );
        expect(serverlessChat.items).toEqual([{ text: "chat-serverless" }]);

        await expect(
            collectStream(
                gemini.streamGeminiChat([{ role: "user", content: "hi" }], "sys", { apiKey: "" }),
            ),
        ).rejects.toThrow("API Key is required");
    });

    test("streams structured output in backend/serverless modes", async () => {
        process.env.VITE_USE_FIREBASE = "false";
        process.env.VITE_USE_SUPABASE = "false";

        enqueue(
            makeSseResponse([
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"thought\":\"think\"},{\"text\":\"answer\"}]}}]}\n",
                "data: {\"usageMetadata\":{\"promptTokenCount\":2,\"candidatesTokenCount\":3,\"totalTokenCount\":5}}\n",
                "data: {invalid-json}\n",
            ]),
        );

        const backendStructured = await collectStream(
            gemini.streamGeminiStructuredOutput(
                "prompt",
                "system",
                { type: "object" },
                { apiKey: "backend-key", thinkingConfig: { thinkingLevel: "medium" } },
            ),
        );

        expect(backendStructured.items).toEqual([{ thinking: "think", content: "answer" }]);
        expect(backendStructured.result).toEqual({
            promptTokenCount: 2,
            candidatesTokenCount: 3,
            totalTokenCount: 5,
        });
        const backendBody = parseRequestBody(0);
        expect(backendBody.generation_config.responseMimeType).toBe("application/json");
        expect(backendBody.generation_config.thinkingConfig.thinkingLevel).toBe("medium");

        const originalWindowLocal = (globalThis as any).window;
        const originalDocumentLocal = (globalThis as any).document;
        try {
            delete (globalThis as any).window;
            delete (globalThis as any).document;
            process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "node-admin-pw";
            calls.length = 0;
            queue.length = 0;

            enqueue(makeSseResponse(["data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"node-mode\"}]}}]}\n"]));
            await collectStream(
                gemini.streamGeminiStructuredOutput(
                    "prompt",
                    "system",
                    { type: "object" },
                    { apiKey: "backend-key" },
                ),
            );

            const nodeBody = parseRequestBody(0);
            expect(String(nodeBody.api_key).startsWith("v1:")).toBe(true);
        } finally {
            (globalThis as any).window = originalWindowLocal;
            (globalThis as any).document = originalDocumentLocal;
            process.env.VITE_ADMIN_ENCRYPTION_PASSWORD = "";
            calls.length = 0;
            queue.length = 0;
        }

        process.env.VITE_USE_FIREBASE = "true";
        process.env.VITE_USE_SUPABASE = "false";
        enqueue(makeSseResponse(["data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"json\"}]}}]}\n"]));
        const serverlessStructured = await collectStream(
            gemini.streamGeminiStructuredOutput(
                "prompt",
                "system",
                { type: "object" },
                { apiKey: "serverless-key", tools: [{ googleSearch: {} }] },
            ),
        );
        expect(serverlessStructured.items).toEqual([{ thinking: undefined, content: "json" }]);
        expect(calls[calls.length - 1]?.url).toContain("generativelanguage.googleapis.com");

        await expect(
            collectStream(
                gemini.streamGeminiStructuredOutput("prompt", "system", { type: "object" }, { apiKey: "" }),
            ),
        ).rejects.toThrow("API Key is required for serverless mode");
    });
});
