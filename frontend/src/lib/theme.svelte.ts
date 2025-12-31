import { browser } from "$app/environment";

export type Theme = "light" | "dark";

class ThemeState {
    theme = $state<Theme>("light");
    colorblindMode = $state<boolean>(false);

    constructor() {
        if (browser) {
            // Load initial state
            const savedTheme = localStorage.getItem("theme") as Theme;
            if (savedTheme) {
                this.theme = savedTheme;
            } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
                this.theme = "dark";
            }
            
            const savedColorblind = localStorage.getItem("colorblindMode") === "true";
            this.colorblindMode = savedColorblind;

            // Apply initially - will be tracked by effects if used in components,
            // but for global side effects we use the setters.
            this.applyTheme(this.theme);
            this.applyColorblind(this.colorblindMode);
        }
    }

    private applyTheme(value: Theme) {
        if (!browser) return;
        if (value === "dark") {
            document.documentElement.classList.add("dark");
        } else {
            document.documentElement.classList.remove("dark");
        }
    }

    private applyColorblind(value: boolean) {
        if (!browser) return;
        if (value) {
            document.documentElement.classList.add("colorblind");
        } else {
            document.documentElement.classList.remove("colorblind");
        }
    }

    get current() {
        return this.theme;
    }

    set current(value: Theme) {
        this.theme = value;
        if (browser) {
            localStorage.setItem("theme", value);
            this.applyTheme(value);
        }
    }

    get isColorblind() {
        return this.colorblindMode;
    }

    set isColorblind(value: boolean) {
        this.colorblindMode = value;
        if (browser) {
            localStorage.setItem("colorblindMode", String(value));
            this.applyColorblind(value);
        }
    }

    toggle() {
        this.current = this.theme === "light" ? "dark" : "light";
    }

    toggleColorblind() {
        this.isColorblind = !this.colorblindMode;
    }
}

export const themeState = new ThemeState();