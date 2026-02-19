import { afterAll, beforeAll, describe, expect, test, mock } from "bun:test";

const LIB_ROOT = "/Users/jaichang/Documents/GitHub/open-codelabs/frontend/src/lib";

type ApiModule = typeof import("../api");

const originalUseSupabase = process.env.VITE_USE_SUPABASE;
const originalUseFirebase = process.env.VITE_USE_FIREBASE;
const originalAlert = (globalThis as any).alert;

const alerts: string[] = [];

const delegated = {
    backend: {
        ASSET_URL: "backend-asset",
        getWsUrl: () => "backend-ws",
        listCodelabs: async () => ["backend-list"],
        login: async () => ({ status: "backend-login" }),
        listenToWsReplacement: () => "backend-listen",
        saveAdminSettings: async () => "backend-save-settings",
        copyCodelab: async () => "backend-copy",
        submitSubmissionLink: async () => "backend-link",
        getUpdateStatus: async () => "backend-update",
        exportCodelab: async () => "backend-export-codelab",
        importCodelab: async () => "backend-import-codelab",
        exportBackup: async () => "backend-export-backup",
        restoreBackup: async () => "backend-restore-backup",
        inspectBackup: async () => "backend-inspect-backup",
        createCodeServer: async () => "backend-create-codeserver",
        getCodeServerInfo: async () => "backend-codeserver-info",
        createCodeServerBranch: async () => "backend-create-branch",
        createCodeServerFolder: async () => "backend-create-folder",
        downloadCodeServerWorkspace: async () => "backend-download-workspace",
        deleteCodeServer: async () => "backend-delete-codeserver",
        saveAiConversation: async () => "backend-save-ai",
        getAiConversations: async () => ["backend-ai"],
        getCertificate: async () => "backend-cert",
        addMaterial: async () => "backend-add-material",
        deleteMaterial: async () => "backend-delete-material",
        uploadMaterial: async () => "backend-upload-material",
        submitFile: async () => "backend-submit-file",
        deleteSubmission: async () => "backend-delete-submission",
        updateQuizzes: async () => "backend-update-quizzes",
        completeCodelab: async () => "backend-complete",
    },
    firebase: {
        listCodelabs: async () => ["firebase-list"],
        login: async () => ({ status: "firebase-login" }),
        loginWithGoogle: async () => ({ status: "firebase-google" }),
        listenToWsReplacement: () => "firebase-listen",
    },
    supabase: {
        listCodelabs: async () => ["supabase-list"],
        login: async () => ({ status: "supabase-login" }),
        loginWithGoogle: async () => ({ status: "supabase-google" }),
        listenToWsReplacement: () => "supabase-listen",
        completeCodelab: async () => "supabase-complete",
        getCertificate: async () => "supabase-cert",
        addMaterial: async () => "supabase-add-material",
        deleteMaterial: async () => "supabase-delete-material",
        uploadMaterial: async () => "supabase-upload-material",
        submitFile: async () => "supabase-submit-file",
        deleteSubmission: async () => "supabase-delete-submission",
        updateQuizzes: async () => "supabase-update-quizzes",
        saveAiConversation: async () => "supabase-save-ai",
        getAiConversations: async () => ["supabase-ai"],
    },
};

beforeAll(() => {
    (globalThis as any).alert = (message: string) => {
        alerts.push(message);
    };

    mock.module(`${LIB_ROOT}/api-backend.ts`, () => delegated.backend as any);
    mock.module(`${LIB_ROOT}/api-firebase.ts`, () => delegated.firebase as any);
    mock.module(`${LIB_ROOT}/api-supabase.ts`, () => delegated.supabase as any);
});

afterAll(() => {
    mock.restore();
    if (originalUseSupabase === undefined) delete process.env.VITE_USE_SUPABASE;
    else process.env.VITE_USE_SUPABASE = originalUseSupabase;
    if (originalUseFirebase === undefined) delete process.env.VITE_USE_FIREBASE;
    else process.env.VITE_USE_FIREBASE = originalUseFirebase;
    (globalThis as any).alert = originalAlert;
});

async function loadApi(mode: string, useSupabase: boolean, useFirebase: boolean): Promise<ApiModule> {
    process.env.VITE_USE_SUPABASE = useSupabase ? "true" : "false";
    process.env.VITE_USE_FIREBASE = useFirebase ? "true" : "false";
    return import(`../api.ts?${mode}-${Date.now()}`);
}

describe("api mode routing", () => {
    test("routes to backend mode and executes backend fallbacks", async () => {
        const api = await loadApi("backend", false, false);

        expect(api.isSupabaseMode()).toBe(false);
        expect(api.isFirebaseMode()).toBe(false);
        expect(api.isServerlessMode()).toBe(false);
        expect(api.ASSET_URL).toBe("backend-asset");
        expect(await api.listCodelabs()).toEqual(["backend-list"]);
        expect(api.getWsUrl("c1")).toBe("backend-ws");
        expect(api.listenToWsReplacement("c1", () => {})).toBe("firebase-listen");
        expect(await api.login("admin", "pw")).toEqual({ status: "backend-login" });

        await expect(api.loginWithGoogle()).rejects.toThrow("Not supported in backend mode");
        await expect(api.onAuthChange(() => {})()).toBeUndefined();
        await expect(api.updateAttendeeProgress("c1", "a1", 1)).resolves.toBeUndefined();
        await expect(api.sendChatMessage("c1", { sender: "x", message: "y", type: "chat" })).resolves.toBeUndefined();
    });

    test("routes to supabase mode and executes serverless unsupported handlers", async () => {
        const api = await loadApi("supabase", true, false);

        expect(api.isSupabaseMode()).toBe(true);
        expect(api.isFirebaseMode()).toBe(false);
        expect(api.isServerlessMode()).toBe(true);
        expect(api.ASSET_URL).toBe("");
        expect(await api.listCodelabs()).toEqual(["supabase-list"]);
        expect(api.listenToWsReplacement("c1", () => {})).toBe("supabase-listen");
        expect(await api.loginWithGoogle()).toEqual({ status: "supabase-google" });
        expect(await api.completeCodelab("c1")).toBe("supabase-complete");

        await expect(api.copyCodelab("c1")).rejects.toThrow("Not supported in serverless mode");
        await expect(api.submitSubmissionLink("c1", "a1", "https://example.com")).rejects.toThrow("Link submission not supported");
        await expect(api.getUpdateStatus()).rejects.toThrow("Update check not supported");
        await expect(api.importCodelab(new File(["x"], "x.zip"))).rejects.toThrow("Import is not supported in serverless mode yet.");
        await expect(api.restoreBackup(new File(["x"], "x.zip"))).rejects.toThrow("Backup restore is not supported in serverless mode yet.");
        await expect(api.inspectBackup(new File(["x"], "x.zip"))).rejects.toThrow("Backup inspect is not supported in serverless mode yet.");
        await expect(api.createCodeServer("c1")).rejects.toThrow("Not supported in serverless mode");
        await expect(api.getCodeServerInfo("c1")).rejects.toThrow("Not supported in serverless mode");
        await expect(api.createCodeServerBranch("c1", 1, "start")).rejects.toThrow("Not supported in serverless mode");
        await expect(api.createCodeServerFolder("c1", 1, "start", [])).rejects.toThrow("Not supported in serverless mode");
        await expect(api.downloadCodeServerWorkspace("c1")).rejects.toThrow("Not supported in serverless mode");
        await expect(api.deleteCodeServer("c1")).rejects.toThrow("Not supported in serverless mode");

        await expect(api.saveAdminSettings({ gemini_api_key: "x" })).resolves.toBeUndefined();
        await expect(api.saveAiConversation({ codelab_id: "c1", question: "q", answer: "a" })).resolves.toBeUndefined();
        await expect(api.getAiConversations("c1")).resolves.toEqual([]);
        await expect(api.exportCodelab("c1")).resolves.toBeUndefined();
        await expect(api.exportBackup()).resolves.toBeUndefined();
        expect(alerts.some((x) => x.includes("Export is not supported"))).toBe(true);
        expect(alerts.some((x) => x.includes("Backup export is not supported"))).toBe(true);
    });

    test("routes to firebase mode and executes firebase unsupported handlers", async () => {
        const api = await loadApi("firebase", false, true);

        expect(api.isSupabaseMode()).toBe(false);
        expect(api.isFirebaseMode()).toBe(true);
        expect(api.isServerlessMode()).toBe(true);
        expect(await api.listCodelabs()).toEqual(["firebase-list"]);
        expect(await api.login("admin", "pw")).toEqual({ status: "firebase-login" });
        expect(await api.loginWithGoogle()).toEqual({ status: "firebase-google" });
        expect(api.listenToWsReplacement("c1", () => {})).toBe("firebase-listen");

        await expect(api.getCertificate("a1")).rejects.toThrow("Not supported in Firebase mode");
        await expect(api.addMaterial("c1", { title: "x", material_type: "link" })).rejects.toThrow("Not supported in Firebase mode");
        await expect(api.deleteMaterial("c1", "m1")).rejects.toThrow("Not supported in Firebase mode");
        await expect(api.uploadMaterial(new File(["x"], "x.txt"))).rejects.toThrow("Not supported in Firebase mode");
        await expect(api.submitFile("c1", "a1", new File(["x"], "x.txt"))).rejects.toThrow("Not supported in Firebase mode");
        await expect(api.deleteSubmission("c1", "a1", "s1")).rejects.toThrow("Not supported in Firebase mode");
        await expect(api.updateQuizzes("c1", [])).rejects.toThrow("Not supported in Firebase mode");

        await expect(api.completeCodelab("c1")).resolves.toBeUndefined();
        await expect(api.submitQuiz("c1", { submissions: [] })).resolves.toBeUndefined();
        await expect(api.saveAiConversation({ codelab_id: "c1", question: "q", answer: "a" })).resolves.toBeUndefined();
        await expect(api.getMaterials("c1")).resolves.toEqual([]);
        await expect(api.getSubmissions("c1")).resolves.toEqual([]);
        await expect(api.getQuizzes("c1")).resolves.toEqual([]);
        await expect(api.getQuizSubmissions("c1")).resolves.toEqual([]);
        await expect(api.getAiConversations("c1")).resolves.toEqual([]);
    });
});
