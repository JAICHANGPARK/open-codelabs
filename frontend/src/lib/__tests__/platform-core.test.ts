import { afterAll, beforeAll, describe, expect, test } from "bun:test";

const originalConsoleError = console.error;
const originalDollarState = (globalThis as any)["$state"];
const originalWindow = (globalThis as any).window;
const originalDocument = (globalThis as any).document;
const originalLocalStorage = (globalThis as any).localStorage;
const originalEnv = {
    VITE_FIREBASE_API_KEY: process.env.VITE_FIREBASE_API_KEY,
    VITE_FIREBASE_AUTH_DOMAIN: process.env.VITE_FIREBASE_AUTH_DOMAIN,
    VITE_FIREBASE_PROJECT_ID: process.env.VITE_FIREBASE_PROJECT_ID,
    VITE_FIREBASE_STORAGE_BUCKET: process.env.VITE_FIREBASE_STORAGE_BUCKET,
    VITE_FIREBASE_MESSAGING_SENDER_ID: process.env.VITE_FIREBASE_MESSAGING_SENDER_ID,
    VITE_FIREBASE_APP_ID: process.env.VITE_FIREBASE_APP_ID,
    VITE_FIREBASE_DATABASE_URL: process.env.VITE_FIREBASE_DATABASE_URL,
};

beforeAll(() => {
    process.env.VITE_FIREBASE_API_KEY = "";
    process.env.VITE_FIREBASE_AUTH_DOMAIN = "";
    process.env.VITE_FIREBASE_PROJECT_ID = "";
    process.env.VITE_FIREBASE_STORAGE_BUCKET = "";
    process.env.VITE_FIREBASE_MESSAGING_SENDER_ID = "";
    process.env.VITE_FIREBASE_APP_ID = "";
    process.env.VITE_FIREBASE_DATABASE_URL = "";
});

afterAll(() => {
    console.error = originalConsoleError;
    (globalThis as any)["$state"] = originalDollarState;
    (globalThis as any).window = originalWindow;
    (globalThis as any).document = originalDocument;
    (globalThis as any).localStorage = originalLocalStorage;
    for (const [key, value] of Object.entries(originalEnv)) {
        if (value === undefined) delete (process.env as any)[key];
        else (process.env as any)[key] = value;
    }
});

describe("firebase.ts", () => {
    test("validates config values", async () => {
        const firebaseModule = await import("../firebase");
        expect(firebaseModule.isFirebaseConfigValid("api-key")).toBe(true);
        expect(firebaseModule.isFirebaseConfigValid("")).toBe(false);
        expect(firebaseModule.isFirebaseConfigValid(undefined)).toBe(false);
        expect(firebaseModule.isFirebaseConfigValid("undefined")).toBe(false);
    });

    test("creates services and reuses existing app", async () => {
        const firebaseModule = await import("../firebase");
        const calls: string[] = [];
        const existingApp = { id: "existing" } as any;

        const validConfig = {
            apiKey: "api-key",
            authDomain: "auth.example",
            projectId: "project",
            storageBucket: "bucket",
            messagingSenderId: "123",
            appId: "app",
            databaseURL: "https://db.example",
        };

        const created = firebaseModule.createFirebaseServices(validConfig, {
            initializeApp: (config: any) => {
                calls.push("initializeApp");
                return { config } as any;
            },
            getApps: () => [],
            getFirestore: (app: any) => {
                calls.push("getFirestore");
                return { app, kind: "firestore" } as any;
            },
            getAuth: (app: any) => {
                calls.push("getAuth");
                return { app, kind: "auth" } as any;
            },
            getStorage: (app: any) => {
                calls.push("getStorage");
                return { app, kind: "storage" } as any;
            },
            getDatabase: (app: any) => {
                calls.push("getDatabase");
                return { app, kind: "database" } as any;
            },
        } as any);

        expect(created.app).toBeDefined();
        expect(created.db).toEqual({ app: created.app, kind: "firestore" });
        expect(calls).toEqual(["initializeApp", "getFirestore", "getAuth", "getStorage", "getDatabase"]);

        const reused = firebaseModule.createFirebaseServices(validConfig, {
            initializeApp: () => {
                throw new Error("should not initialize");
            },
            getApps: () => [existingApp],
            getFirestore: () => ({ kind: "firestore" } as any),
            getAuth: () => ({ kind: "auth" } as any),
            getStorage: () => ({ kind: "storage" } as any),
            getDatabase: () => ({ kind: "database" } as any),
        } as any);

        expect(reused.app).toBe(existingApp);
    });

    test("returns empty services for invalid config and logs on init failures", async () => {
        const firebaseModule = await import("../firebase");
        expect(firebaseModule.createFirebaseServices({ apiKey: "" })).toEqual({});

        const validConfig = {
            apiKey: "api-key",
            authDomain: "auth.example",
            projectId: "project",
            storageBucket: "bucket",
            messagingSenderId: "123",
            appId: "app",
            databaseURL: "https://db.example",
        };

        const logged: unknown[][] = [];
        console.error = (...args: unknown[]) => {
            logged.push(args);
        };

        const partial = firebaseModule.createFirebaseServices(validConfig, {
            initializeApp: () => ({ id: "app" } as any),
            getApps: () => [],
            getFirestore: () => ({ kind: "firestore" } as any),
            getAuth: () => ({ kind: "auth" } as any),
            getStorage: () => ({ kind: "storage" } as any),
            getDatabase: () => {
                throw new Error("db failed");
            },
        } as any);

        expect(partial.app).toBeDefined();
        expect(partial.db).toBeDefined();
        expect(partial.auth).toBeDefined();
        expect(partial.storage).toBeDefined();
        expect(partial.rtdb).toBeUndefined();
        expect(logged.length).toBe(1);

        console.error = originalConsoleError;
    });
});

describe("supabase.ts", () => {
    test("validates config values", async () => {
        const supabaseModule = await import("../supabase");
        expect(supabaseModule.isSupabaseConfigValid("https://supabase.example", "anon")).toBe(true);
        expect(supabaseModule.isSupabaseConfigValid("", "anon")).toBe(false);
        expect(supabaseModule.isSupabaseConfigValid("https://supabase.example", "")).toBe(false);
        expect(supabaseModule.isSupabaseConfigValid("undefined", "anon")).toBe(false);
        expect(supabaseModule.isSupabaseConfigValid("https://supabase.example", "undefined")).toBe(false);
    });

    test("creates client only for valid config", async () => {
        const supabaseModule = await import("../supabase");
        const createCalls: any[] = [];

        const client = supabaseModule.createSupabaseClient("https://supabase.example", "anon-key", {
            createClient: (url: string, key: string, options: unknown) => {
                createCalls.push({ url, key, options });
                return { url, key } as any;
            },
        } as any);

        expect(client).toEqual({ url: "https://supabase.example", key: "anon-key" });
        expect(createCalls[0]).toEqual({
            url: "https://supabase.example",
            key: "anon-key",
            options: { auth: supabaseModule.defaultAuthOptions },
        });

        expect(supabaseModule.createSupabaseClient("undefined", "anon-key", { createClient: (() => ({})) as any })).toBeUndefined();
    });
});

describe("i18n/index.ts", () => {
    test("registers all locales and resolves navigator locale", async () => {
        const i18nModule = await import("../i18n");
        const registered = new Map<string, () => Promise<unknown>>();

        i18nModule.registerAllLocales((locale, loader) => {
            registered.set(locale, loader as () => Promise<unknown>);
        });

        expect(registered.size).toBe(i18nModule.allLocales.length);

        for (const locale of i18nModule.allLocales) {
            const loader = registered.get(locale);
            expect(loader).toBeDefined();
            const loaded = await loader!();
            expect(loaded).toBeDefined();
        }

        expect(i18nModule.resolveInitialLocale("zh-tw")).toBe("zh-TW");
        expect(i18nModule.resolveInitialLocale("ko-KR")).toBe("ko");
        expect(i18nModule.resolveInitialLocale("xx-YY")).toBe("en");
        expect(i18nModule.resolveInitialLocale(undefined)).toBe("en");
    });

    test("setup initializes i18n and sets locale", async () => {
        const i18nModule = await import("../i18n");
        const registered: string[] = [];
        const initCalls: unknown[] = [];
        const localeSetCalls: string[] = [];

        const chosen = i18nModule.setupI18n(
            (localeCode) => {
                registered.push(localeCode);
            },
            (config) => {
                initCalls.push(config);
            },
            { set: (value: string) => localeSetCalls.push(value) } as any,
            "ja-JP",
        );

        expect(chosen).toBe("ja");
        expect(registered.length).toBe(i18nModule.allLocales.length);
        expect(initCalls).toEqual([{ fallbackLocale: "en", initialLocale: "ja" }]);
        expect(localeSetCalls).toEqual(["ja"]);
    });
});

describe("theme.svelte.ts", () => {
    test("applies browser state and handles mode transitions", async () => {
        (globalThis as any)["$state"] = <T>(value: T) => value;

        const classSet = new Set<string>();
        const listeners: Array<(event: { matches: boolean }) => void> = [];
        const storage = new Map<string, string>([
            ["colorblindMode", "true"],
            ["themePreset", "invalid-preset"],
            ["themeMode", "system"],
        ]);

        (globalThis as any).window = {
            matchMedia: () => ({
                matches: true,
                addEventListener: (_type: string, cb: (event: { matches: boolean }) => void) => listeners.push(cb),
                removeEventListener: (_type: string, cb: (event: { matches: boolean }) => void) => {
                    const index = listeners.indexOf(cb);
                    if (index >= 0) listeners.splice(index, 1);
                },
            }),
        };
        (globalThis as any).document = {
            documentElement: {
                dataset: {} as Record<string, string>,
                classList: {
                    add: (name: string) => classSet.add(name),
                    remove: (name: string) => classSet.delete(name),
                    toggle: (name: string, force?: boolean) => {
                        if (force === undefined) {
                            if (classSet.has(name)) classSet.delete(name);
                            else classSet.add(name);
                            return classSet.has(name);
                        }
                        if (force) classSet.add(name);
                        else classSet.delete(name);
                        return force;
                    },
                    contains: (name: string) => classSet.has(name),
                },
            },
        };
        (globalThis as any).localStorage = {
            getItem: (key: string) => storage.get(key) ?? null,
            setItem: (key: string, value: string) => storage.set(key, value),
        };

        const themeModule = await import("../theme.svelte.ts");

        expect(themeModule.isThemePreset(null)).toBe(false);
        expect(themeModule.isThemePreset("mint")).toBe(true);

        const state = new themeModule.ThemeState();
        expect(state.isColorblind).toBe(true);
        expect(state.presetId).toBe("default");
        expect(state.modeId).toBe("system");
        expect(state.presets.length).toBeGreaterThan(0);

        state.toggleColorblind();
        expect(classSet.has("colorblind")).toBe(false);

        state.setPreset("ocean");
        expect((globalThis as any).document.documentElement.dataset.theme).toBe("ocean");
        expect(storage.get("themePreset")).toBe("ocean");

        state.setMode("system");
        expect(listeners.length).toBeGreaterThan(0);
        listeners[0]?.({ matches: false });
        expect(classSet.has("dark")).toBe(false);

        state.setMode("dark");
        expect(classSet.has("dark")).toBe(true);

        state.toggleMode();
        expect(state.modeId).toBe("light");

        delete (globalThis as any).window;
        delete (globalThis as any).document;
        delete (globalThis as any).localStorage;
        const serverState = new themeModule.ThemeState();
        expect(serverState.modeId).toBe("system");
        serverState.toggleMode();
        expect(serverState.modeId).toBe("dark");
    });
});
