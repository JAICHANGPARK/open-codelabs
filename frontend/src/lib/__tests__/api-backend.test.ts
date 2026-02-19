import { afterAll, beforeAll, beforeEach, describe, expect, test } from "bun:test";

type ApiBackendModule = typeof import("../api-backend");

type FetchCall = {
    url: string;
    init: RequestInit;
};

type LocationLike = {
    hostname: string;
    port: string;
    origin: string;
    protocol: string;
};

const calls: FetchCall[] = [];
const queue: Response[] = [];
const createdAnchors: Array<{ href: string; download: string; clicked: boolean; click: () => void }> = [];
const createdObjectUrls: string[] = [];
const revokedObjectUrls: string[] = [];
const appendedNodes: unknown[] = [];
const removedNodes: unknown[] = [];

const originalFetch = globalThis.fetch;
const originalWindow = (globalThis as any).window;
const originalDocument = (globalThis as any).document;
const originalSessionStorage = (globalThis as any).sessionStorage;

let api: ApiBackendModule;

function makeJsonResponse(status: number, payload: unknown) {
    return new Response(JSON.stringify(payload), {
        status,
        headers: { "Content-Type": "application/json" },
    });
}

function makeTextResponse(status: number, text: string) {
    return new Response(text, { status });
}

function makeBlobResponse(status: number, text = "blob-content") {
    return new Response(text, { status });
}

function makeRejectingTextResponse(status: number) {
    return {
        ok: false,
        status,
        text: async () => {
            throw new Error("text failed");
        },
    } as unknown as Response;
}

function enqueue(response: Response) {
    queue.push(response);
}

function makeFile(name = "demo.txt", content = "hello") {
    return new File([content], name, { type: "text/plain" });
}

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
        removeItem(key: string) {
            delete store[key];
        },
    };
}

function findCall(pathPart: string, method?: string): FetchCall {
    const upper = method?.toUpperCase();
    const hit = calls.find((call) => {
        const callMethod = (call.init.method || "GET").toUpperCase();
        const methodMatches = upper ? callMethod === upper : true;
        return methodMatches && call.url.includes(pathPart);
    });
    if (!hit) throw new Error(`Missing fetch call for ${method || "ANY"} ${pathPart}`);
    return hit;
}

describe("api-backend", () => {
    beforeAll(async () => {
        (globalThis as any).window = {
            location: {
                hostname: "demo.ngrok.io",
                port: "",
                origin: "https://demo.ngrok.io",
                protocol: "https:",
            },
            URL: {
                createObjectURL(blob: Blob) {
                    const url = `blob:mock-${createdObjectUrls.length + 1}`;
                    createdObjectUrls.push(url);
                    void blob;
                    return url;
                },
                revokeObjectURL(url: string) {
                    revokedObjectUrls.push(url);
                },
            },
        };

        (globalThis as any).document = {
            cookie: "__Host-oc_csrf=token-1; oc_csrf=token-2",
            body: {
                appendChild(node: unknown) {
                    appendedNodes.push(node);
                    return node;
                },
                removeChild(node: unknown) {
                    removedNodes.push(node);
                    return node;
                },
            },
            createElement(tag: string) {
                if (tag !== "a") return {};
                const anchor = {
                    href: "",
                    download: "",
                    clicked: false,
                    click() {
                        this.clicked = true;
                    },
                };
                createdAnchors.push(anchor);
                return anchor;
            },
        };

        (globalThis as any).sessionStorage = makeSessionStorage({ adminPassword: "admin-pw" });

        globalThis.fetch = async (input: RequestInfo | URL, init: RequestInit = {}) => {
            calls.push({ url: String(input), init });
            const next = queue.shift();
            if (!next) {
                throw new Error(`No queued response for ${String(input)}`);
            }
            return next;
        };

        api = await import(`../api-backend.ts?backend-suite-${Date.now()}`);
    });

    beforeEach(() => {
        calls.length = 0;
        queue.length = 0;
        createdAnchors.length = 0;
        createdObjectUrls.length = 0;
        revokedObjectUrls.length = 0;
        appendedNodes.length = 0;
        removedNodes.length = 0;
        (globalThis as any).document.cookie = "__Host-oc_csrf=token-1; oc_csrf=token-2";
        (globalThis as any).sessionStorage = makeSessionStorage({ adminPassword: "admin-pw" });
    });

    afterAll(() => {
        globalThis.fetch = originalFetch;
        (globalThis as any).window = originalWindow;
        (globalThis as any).document = originalDocument;
        (globalThis as any).sessionStorage = originalSessionStorage;
    });

    test("resolves backend base URL and websocket URL across host patterns", () => {
        const tunnelLoc: LocationLike = {
            hostname: "x.ngrok.io",
            port: "",
            origin: "https://x.ngrok.io",
            protocol: "https:",
        };
        const localLoc: LocationLike = {
            hostname: "localhost",
            port: "5173",
            origin: "http://localhost:5173",
            protocol: "http:",
        };
        const remoteWithPort: LocationLike = {
            hostname: "192.168.1.25",
            port: "3000",
            origin: "http://192.168.1.25:3000",
            protocol: "http:",
        };

        expect(api.resolveBackendBaseUrl(undefined, undefined)).toBe("http://localhost:8080");
        expect(api.resolveBackendBaseUrl("https://api.example.com", tunnelLoc)).toBe("https://api.example.com");
        expect(api.resolveBackendBaseUrl("http://backend:8080", tunnelLoc)).toBe("https://x.ngrok.io");
        expect(api.resolveBackendBaseUrl(undefined, localLoc)).toBe("http://localhost:8080");
        expect(api.resolveBackendBaseUrl(undefined, remoteWithPort)).toBe("http://192.168.1.25:8080");

        expect(api.getWsUrl("lab")).toMatch(/\/api\/ws\/lab$/);
        expect(api.getWsUrl("lab", "admin", "tok")).toContain("as=admin");
        expect(api.getWsUrl("lab", "admin", "tok")).toContain("token=tok");
    });

    test("covers success paths across backend API functions", async () => {
        const codelabPayload = { title: "T", description: "D", author: "A" };

        enqueue(makeJsonResponse(200, [{ id: "c1" }]));
        expect(await api.listCodelabs()).toEqual([{ id: "c1" }]);

        enqueue(makeJsonResponse(200, [{ id: "c1" }, [{ id: "s1" }]]));
        expect(await api.getCodelab("c1")).toEqual([{ id: "c1" }, [{ id: "s1" }]]);

        enqueue(makeJsonResponse(200, { id: "c2" }));
        expect(await api.createCodelab(codelabPayload)).toEqual({ id: "c2" });

        enqueue(makeJsonResponse(200, { id: "c2" }));
        expect(await api.updateCodelab("c2", codelabPayload)).toEqual({ id: "c2" });

        enqueue(makeTextResponse(204, ""));
        await api.saveSteps("c1", [{ title: "s", content_markdown: "m" }]);

        enqueue(makeTextResponse(204, ""));
        await api.deleteCodelab("c1");

        enqueue(makeJsonResponse(200, { id: "copied" }));
        expect(await api.copyCodelab("c1")).toEqual({ id: "copied" });

        enqueue(makeJsonResponse(200, { status: "ok", token: "jwt" }));
        expect(await api.login("admin", "pw")).toEqual({ status: "ok", token: "jwt" });

        enqueue(makeTextResponse(204, ""));
        await api.logout();

        enqueue(makeJsonResponse(200, { role: "admin", sub: "u1", exp: 10 }));
        expect(await api.getSession()).toEqual({ role: "admin", sub: "u1", exp: 10 });

        enqueue(makeJsonResponse(200, [{ id: "a1" }]));
        expect(await api.getAuditLogs({ limit: 10, offset: 20, codelab_id: "c1", action: "login" })).toEqual([{ id: "a1" }]);

        enqueue(makeTextResponse(200, ""));
        await api.saveAdminSettings({ gemini_api_key: "raw-key" });

        enqueue(makeBlobResponse(200, "zip-1"));
        await api.exportCodelab("c1");

        enqueue(makeJsonResponse(200, { id: "imported" }));
        expect(await api.importCodelab(makeFile("import.zip"))).toEqual({ id: "imported" });

        enqueue(makeBlobResponse(200, "zip-2"));
        await api.exportBackup();

        enqueue(makeTextResponse(204, ""));
        await api.restoreBackup(makeFile("backup.zip"));

        enqueue(makeJsonResponse(200, { version: 1 }));
        expect(await api.inspectBackup(makeFile("backup.zip"))).toEqual({ version: 1 });

        enqueue(makeJsonResponse(200, { id: "att" }));
        expect(await api.registerAttendee("c1", "name", "code", "e@test.com")).toEqual({ id: "att" });

        enqueue(makeTextResponse(204, ""));
        await api.requestHelp("c1", 2);

        enqueue(makeJsonResponse(200, [{ id: "h1" }]));
        expect(await api.getHelpRequests("c1")).toEqual([{ id: "h1" }]);

        enqueue(makeTextResponse(204, ""));
        await api.resolveHelpRequest("c1", "h1");

        enqueue(makeJsonResponse(200, [{ id: "a1" }]));
        expect(await api.getAttendees("c1")).toEqual([{ id: "a1" }]);

        enqueue(makeJsonResponse(200, [{ id: "m1" }]));
        expect(await api.getChatHistory("c1")).toEqual([{ id: "m1" }]);

        enqueue(makeJsonResponse(200, [{ id: "t1" }]));
        expect(await api.getInlineComments("c1", { target_type: "step", target_step_id: "s1" })).toEqual([{ id: "t1" }]);

        enqueue(makeJsonResponse(200, [{ id: "t2" }]));
        expect(await api.getInlineComments("c1")).toEqual([{ id: "t2" }]);

        enqueue(makeJsonResponse(200, { id: "thread-1" }));
        expect(await api.createInlineComment("c1", {
            anchor_key: "a1",
            target_type: "step",
            target_step_id: "s1",
            start_offset: 0,
            end_offset: 2,
            selected_text: "hi",
            message: "hi",
            content_hash: "h",
        })).toEqual({ id: "thread-1" });

        enqueue(makeJsonResponse(200, { id: "thread-2" }));
        expect(await api.replyInlineComment("c1", "t1", { message: "reply", content_hash: "h2" })).toEqual({ id: "thread-2" });

        enqueue(makeTextResponse(204, ""));
        await api.deleteInlineComment("c1", "t1", "cmt1");

        enqueue(makeJsonResponse(200, { url: "/img.png" }));
        expect(await api.uploadImage(makeFile("image.png", "img"))).toEqual({ url: "/img.png" });

        enqueue(makeTextResponse(204, ""));
        await api.submitFeedback("c1", { difficulty: 3, satisfaction: 4, comments: "good" });

        enqueue(makeJsonResponse(200, [{ id: "f1" }]));
        expect(await api.getFeedback("c1")).toEqual([{ id: "f1" }]);

        enqueue(makeJsonResponse(200, { id: "sub-link" }));
        expect(await api.submitSubmissionLink("c1", "a1", "https://example.com", "title")).toEqual({ id: "sub-link" });

        enqueue(makeJsonResponse(200, { frontend: { update_available: false }, backend: { update_available: false } }));
        expect(await api.getUpdateStatus()).toEqual({
            frontend: { update_available: false },
            backend: { update_available: false },
        });

        enqueue(makeTextResponse(204, ""));
        await api.completeCodelab("c1");

        enqueue(makeJsonResponse(200, { id: "cert" }));
        expect(await api.getCertificate("a1")).toEqual({ id: "cert" });

        enqueue(makeJsonResponse(200, [{ id: "m1" }]));
        expect(await api.getMaterials("c1")).toEqual([{ id: "m1" }]);

        enqueue(makeJsonResponse(200, [{ id: "q1" }]));
        expect(await api.getQuizzes("c1")).toEqual([{ id: "q1" }]);

        enqueue(makeTextResponse(204, ""));
        await api.submitQuiz("c1", { submissions: [{ quiz_id: "q1", answer: "1", is_correct: true }] });

        enqueue(makeJsonResponse(200, [{ id: "qs1" }]));
        expect(await api.getQuizSubmissions("c1")).toEqual([{ id: "qs1" }]);

        enqueue(makeTextResponse(204, ""));
        await api.updateQuizzes("c1", [{ question: "Q", options: ["A", "B"], correct_answer: 0 }]);

        enqueue(makeJsonResponse(200, { id: "mat1" }));
        expect(await api.addMaterial("c1", { title: "link", material_type: "link", link_url: "https://x" })).toEqual({ id: "mat1" });

        enqueue(makeTextResponse(204, ""));
        await api.deleteMaterial("c1", "m1");

        enqueue(makeJsonResponse(200, { url: "/m", original_name: "m.pdf" }));
        expect(await api.uploadMaterial(makeFile("m.pdf", "pdf"))).toEqual({ url: "/m", original_name: "m.pdf" });

        enqueue(makeJsonResponse(200, { id: "sf1" }));
        expect(await api.submitFile("c1", "a1", makeFile("ans.txt", "A"))).toEqual({ id: "sf1" });

        enqueue(makeJsonResponse(200, [{ id: "s1" }]));
        expect(await api.getSubmissions("c1")).toEqual([{ id: "s1" }]);

        enqueue(makeTextResponse(204, ""));
        await api.deleteSubmission("c1", "a1", "s1");

        enqueue(makeJsonResponse(200, { structure_type: "branch" }));
        expect(await api.createCodeServer("c1", [{ path: "main.dart", content: "void main() {}" }], "branch")).toEqual({ structure_type: "branch" });

        enqueue(makeJsonResponse(200, { structure_type: "folder" }));
        expect(await api.getCodeServerInfo("c1")).toEqual({ structure_type: "folder" });

        enqueue(makeTextResponse(204, ""));
        await api.createCodeServerBranch("c1", 1, "start");

        enqueue(makeTextResponse(204, ""));
        await api.createCodeServerFolder("c1", 1, "end", [{ path: "a.txt", content: "x" }]);

        enqueue(makeBlobResponse(200, "workspace"));
        await api.downloadCodeServerWorkspace("c1");

        enqueue(makeJsonResponse(200, ["main"]));
        expect(await api.getWorkspaceBranches("c1")).toEqual(["main"]);

        enqueue(makeJsonResponse(200, ["README.md"]));
        expect(await api.getWorkspaceFiles("c1", "main")).toEqual(["README.md"]);

        enqueue(makeTextResponse(200, "file-content"));
        expect(await api.getWorkspaceFileContent("c1", "main", "README.md")).toBe("file-content");

        enqueue(makeTextResponse(204, ""));
        await api.updateWorkspaceBranchFiles("c1", "main", [{ path: "README.md", content: "v2" }], ["old.txt"], "commit");

        enqueue(makeTextResponse(204, ""));
        await api.updateWorkspaceBranchFiles("c1", "main", [{ path: "README.md", content: "v3" }]);

        enqueue(makeJsonResponse(200, ["step1"]));
        expect(await api.getWorkspaceFolders("c1")).toEqual(["step1"]);

        enqueue(makeJsonResponse(200, ["main.dart"]));
        expect(await api.getWorkspaceFolderFiles("c1", "step1")).toEqual(["main.dart"]);

        enqueue(makeTextResponse(200, "folder-file"));
        expect(await api.getWorkspaceFolderFileContent("c1", "step1", "main.dart")).toBe("folder-file");

        enqueue(makeTextResponse(204, ""));
        await api.updateWorkspaceFolderFiles("c1", "step1", [{ path: "main.dart", content: "updated" }], ["old.dart"]);

        enqueue(makeTextResponse(204, ""));
        await api.deleteCodeServer("c1");

        enqueue(makeJsonResponse(200, { id: "ai1" }));
        expect(await api.saveAiConversation({ codelab_id: "c1", question: "Q", answer: "A" })).toEqual({ id: "ai1" });

        enqueue(makeJsonResponse(200, [{ id: "conv1" }]));
        expect(await api.getAiConversations("c1")).toEqual([{ id: "conv1" }]);

        expect(createdAnchors.length).toBeGreaterThanOrEqual(3);
        expect(createdAnchors.some((a) => a.clicked)).toBe(true);
        expect(createdObjectUrls.length).toBe(3);
        expect(revokedObjectUrls.length).toBe(3);
        expect(appendedNodes.length).toBeGreaterThanOrEqual(3);
        expect(removedNodes.length).toBe(1);

        const getCallHeaders = new Headers(findCall("/api/codelabs", "GET").init.headers);
        const postCallHeaders = new Headers(findCall("/api/codelabs", "POST").init.headers);
        expect(getCallHeaders.get("X-CSRF-Token")).toBeNull();
        expect(postCallHeaders.get("X-CSRF-Token")).toBe("token-1");
    });

    test("covers failure paths across backend API functions", async () => {
        (globalThis as any).sessionStorage = makeSessionStorage({ adminPassword: "admin-pw" });

        const simpleErrorCases: Array<{ run: () => Promise<unknown> }> = [
            { run: () => api.listCodelabs() },
            { run: () => api.createCodelab({ title: "t", description: "d", author: "a" }) },
            { run: () => api.updateCodelab("c1", { title: "t", description: "d", author: "a" }) },
            { run: () => api.saveSteps("c1", [{ title: "x", content_markdown: "y" }]) },
            { run: () => api.deleteCodelab("c1") },
            { run: () => api.copyCodelab("c1") },
            { run: () => api.login("id", "pw") },
            { run: () => api.logout() },
            { run: () => api.getAuditLogs() },
            { run: () => api.exportCodelab("c1") },
            { run: () => api.importCodelab(makeFile("import.zip")) },
            { run: () => api.exportBackup() },
            { run: () => api.getHelpRequests("c1") },
            { run: () => api.resolveHelpRequest("c1", "h1") },
            { run: () => api.getAttendees("c1") },
            { run: () => api.getChatHistory("c1") },
            { run: () => api.getInlineComments("c1") },
            {
                run: () =>
                    api.createInlineComment("c1", {
                        anchor_key: "a1",
                        target_type: "step",
                        target_step_id: "s1",
                        start_offset: 0,
                        end_offset: 1,
                        selected_text: "m",
                        message: "m",
                        content_hash: "h",
                    }),
            },
            { run: () => api.replyInlineComment("c1", "t1", { message: "m", content_hash: "h" }) },
            { run: () => api.deleteInlineComment("c1", "t1", "m1") },
            { run: () => api.uploadImage(makeFile("a.png", "img")) },
            { run: () => api.getFeedback("c1") },
            { run: () => api.completeCodelab("c1") },
            { run: () => api.getCertificate("a1") },
            { run: () => api.getMaterials("c1") },
            { run: () => api.getQuizzes("c1") },
            { run: () => api.submitQuiz("c1", { submissions: [{ quiz_id: "q1", answer: "0", is_correct: false }] }) },
            { run: () => api.getQuizSubmissions("c1") },
            { run: () => api.updateQuizzes("c1", [{ question: "Q", options: ["A"], correct_answer: 0 }]) },
            { run: () => api.addMaterial("c1", { title: "L", material_type: "link", link_url: "https://x" }) },
            { run: () => api.deleteMaterial("c1", "m1") },
            { run: () => api.uploadMaterial(makeFile("a.pdf", "pdf")) },
            { run: () => api.getSubmissions("c1") },
            { run: () => api.deleteSubmission("c1", "a1", "s1") },
            { run: () => api.createCodeServer("c1") },
            { run: () => api.getCodeServerInfo("c1") },
            { run: () => api.createCodeServerBranch("c1", 1, "end") },
            { run: () => api.createCodeServerFolder("c1", 2, "start", [{ path: "x", content: "y" }]) },
            { run: () => api.downloadCodeServerWorkspace("c1") },
            { run: () => api.getWorkspaceBranches("c1") },
            { run: () => api.getWorkspaceFiles("c1", "main") },
            { run: () => api.getWorkspaceFileContent("c1", "main", "a.txt") },
            { run: () => api.updateWorkspaceBranchFiles("c1", "main", [{ path: "x", content: "y" }]) },
            { run: () => api.getWorkspaceFolders("c1") },
            { run: () => api.getWorkspaceFolderFiles("c1", "f") },
            { run: () => api.getWorkspaceFolderFileContent("c1", "f", "a.txt") },
            { run: () => api.updateWorkspaceFolderFiles("c1", "f", [{ path: "x", content: "y" }]) },
            { run: () => api.deleteCodeServer("c1") },
            { run: () => api.saveAiConversation({ codelab_id: "c1", question: "Q", answer: "A" }) },
            { run: () => api.getAiConversations("c1") },
        ];

        for (const item of simpleErrorCases) {
            enqueue(makeTextResponse(500, "error"));
            await expect(item.run()).rejects.toThrow();
        }

        enqueue(makeTextResponse(403, "denied"));
        await expect(api.getCodelab("private")).rejects.toThrow("PRIVATE_CODELAB");

        enqueue(makeTextResponse(500, "error"));
        await expect(api.getCodelab("broken")).rejects.toThrow("Failed to fetch codelab");

        enqueue(makeTextResponse(401, "unauthorized"));
        expect(await api.getSession()).toBeNull();

        enqueue(makeTextResponse(500, "error"));
        await expect(api.getSession()).rejects.toThrow("Failed to fetch session");

        (globalThis as any).sessionStorage = makeSessionStorage();
        await expect(api.saveAdminSettings({ gemini_api_key: "need-password" })).rejects.toThrow("ENCRYPTION_PASSWORD_MISSING");
        (globalThis as any).sessionStorage = makeSessionStorage({ adminPassword: "admin-pw" });

        enqueue(makeTextResponse(500, "save-fail"));
        await expect(api.saveAdminSettings({ gemini_api_key: "raw-key" })).rejects.toThrow("Failed to save settings to server");

        enqueue(makeTextResponse(500, "restore-error"));
        await expect(api.restoreBackup(makeFile("backup.zip"))).rejects.toThrow("Backup restore failed: restore-error");

        enqueue(makeTextResponse(500, "inspect-error"));
        await expect(api.inspectBackup(makeFile("backup.zip"))).rejects.toThrow("Backup inspect failed: inspect-error");

        enqueue(makeTextResponse(409, "dup"));
        await expect(api.registerAttendee("c1", "same", "code")).rejects.toThrow("DUPLICATE_NAME");

        enqueue(makeTextResponse(500, "registration-error"));
        await expect(api.registerAttendee("c1", "n", "c")).rejects.toThrow("registration-error");

        enqueue(makeTextResponse(500, "bad-help"));
        await expect(api.requestHelp("c1", 3)).rejects.toThrow("Help request failed (500): bad-help");

        enqueue(makeTextResponse(409, "dup-feedback"));
        await expect(api.submitFeedback("c1", { difficulty: 1, satisfaction: 1, comments: "x" })).rejects.toThrow("ALREADY_SUBMITTED");

        enqueue(makeTextResponse(500, "bad-feedback"));
        await expect(api.submitFeedback("c1", { difficulty: 1, satisfaction: 1, comments: "x" })).rejects.toThrow("Feedback submission failed");

        enqueue(makeTextResponse(500, "bad-link"));
        await expect(api.submitSubmissionLink("c1", "a1", "https://x")).rejects.toThrow("bad-link");

        enqueue(makeTextResponse(500, "bad-updates"));
        await expect(api.getUpdateStatus()).rejects.toThrow("bad-updates");

        enqueue(makeTextResponse(500, "submit-file-error"));
        await expect(api.submitFile("c1", "a1", makeFile("ans.txt", "A"))).rejects.toThrow("submit-file-error");
    });

    test("handles fallback errors when response text parsing fails", async () => {
        enqueue(makeRejectingTextResponse(500));
        await expect(api.restoreBackup(makeFile("backup.zip"))).rejects.toThrow("Backup restore failed");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.inspectBackup(makeFile("backup.zip"))).rejects.toThrow("Backup inspect failed");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.registerAttendee("c1", "name", "code")).rejects.toThrow("Registration failed");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.requestHelp("c1", 2)).rejects.toThrow("Help request failed (500): Unknown error");

        enqueue(makeRejectingTextResponse(500));
        await expect(
            api.createInlineComment("c1", {
                anchor_key: "a1",
                target_type: "step",
                target_step_id: "s1",
                start_offset: 0,
                end_offset: 1,
                selected_text: "x",
                message: "x",
                content_hash: "h",
            }),
        ).rejects.toThrow("Failed to create inline comment");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.replyInlineComment("c1", "t1", { message: "x", content_hash: "h" })).rejects.toThrow("Failed to reply to inline comment");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.deleteInlineComment("c1", "t1", "m1")).rejects.toThrow("Failed to delete inline comment");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.uploadImage(makeFile("x.png", "img"))).rejects.toThrow("Upload failed");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.submitSubmissionLink("c1", "a1", "https://example.com")).rejects.toThrow("Link submission failed");

        enqueue(makeRejectingTextResponse(500));
        await expect(api.getUpdateStatus()).rejects.toThrow("Failed to check updates");
    });
});
