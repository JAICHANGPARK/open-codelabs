const isBrowser = () => typeof window !== "undefined" && typeof document !== "undefined";

export const THEME_PRESETS = [
    { id: "default", labelKey: "theme.presets.default" },
    { id: "mint", labelKey: "theme.presets.mint" },
    { id: "ocean", labelKey: "theme.presets.ocean" },
    { id: "sunset", labelKey: "theme.presets.sunset" },
    { id: "forest", labelKey: "theme.presets.forest" },
    { id: "berry", labelKey: "theme.presets.berry" },
    { id: "slate", labelKey: "theme.presets.slate" },
] as const;

export type ThemePresetId = (typeof THEME_PRESETS)[number]["id"];
export type ThemeMode = "system" | "light" | "dark";

const THEME_MODE_STORAGE_KEY = "themeMode";
const LEGACY_MODE_STORAGE_KEY = "theme";

export function isThemePreset(value: string | null): value is ThemePresetId {
    if (!value) return false;
    return THEME_PRESETS.some((preset) => preset.id === value);
}

export class ThemeState {
    colorblindMode = $state<boolean>(false);
    preset = $state<ThemePresetId>("default");
    mode = $state<ThemeMode>("system");
    private systemMedia: MediaQueryList | null = null;
    private systemListener: ((event: MediaQueryListEvent) => void) | null = null;

    constructor() {
        if (!isBrowser()) return;

        const savedColorblind = localStorage.getItem("colorblindMode") === "true";
        this.colorblindMode = savedColorblind;
        this.applyColorblind(this.colorblindMode);

        const savedPreset = localStorage.getItem("themePreset");
        if (isThemePreset(savedPreset)) {
            this.preset = savedPreset;
        }
        this.applyPreset(this.preset);

        const savedMode = localStorage.getItem(THEME_MODE_STORAGE_KEY) ?? localStorage.getItem(LEGACY_MODE_STORAGE_KEY);
        if (savedMode === "light" || savedMode === "dark" || savedMode === "system") {
            this.mode = savedMode;
        }
        this.applyMode(this.mode);
    }

    private applyColorblind(value: boolean) {
        if (!isBrowser()) return;
        if (value) {
            document.documentElement.classList.add("colorblind");
        } else {
            document.documentElement.classList.remove("colorblind");
        }
    }

    private applyPreset(value: ThemePresetId) {
        if (!isBrowser()) return;
        document.documentElement.dataset.theme = value;
    }

    private applyMode(value: ThemeMode) {
        if (!isBrowser()) return;
        if (this.systemMedia && this.systemListener) {
            this.systemMedia.removeEventListener("change", this.systemListener);
            this.systemMedia = null;
            this.systemListener = null;
        }

        if (value === "system") {
            const media = window.matchMedia("(prefers-color-scheme: dark)");
            const apply = (isDark: boolean) => {
                document.documentElement.classList.toggle("dark", isDark);
            };
            apply(media.matches);
            const listener = (event: MediaQueryListEvent) => apply(event.matches);
            media.addEventListener("change", listener);
            this.systemMedia = media;
            this.systemListener = listener;
            return;
        }

        document.documentElement.classList.toggle("dark", value === "dark");
    }

    get isColorblind() {
        return this.colorblindMode;
    }

    set isColorblind(value: boolean) {
        this.colorblindMode = value;
        if (isBrowser()) {
            localStorage.setItem("colorblindMode", String(value));
            this.applyColorblind(value);
        }
    }

    toggleColorblind() {
        this.isColorblind = !this.colorblindMode;
    }

    get presets() {
        return THEME_PRESETS;
    }

    get presetId() {
        return this.preset;
    }

    get modeId() {
        return this.mode;
    }

    setPreset(value: ThemePresetId) {
        this.preset = value;
        if (isBrowser()) {
            localStorage.setItem("themePreset", value);
            this.applyPreset(value);
        }
    }

    setMode(value: ThemeMode) {
        this.mode = value;
        if (isBrowser()) {
            localStorage.setItem(THEME_MODE_STORAGE_KEY, value);
            localStorage.setItem(LEGACY_MODE_STORAGE_KEY, value);
            this.applyMode(value);
        }
    }

    toggleMode() {
        const isDark = isBrowser()
            ? document.documentElement.classList.contains("dark")
            : this.mode === "dark";
        this.setMode(isDark ? "light" : "dark");
    }
}

export const themeState = new ThemeState();
