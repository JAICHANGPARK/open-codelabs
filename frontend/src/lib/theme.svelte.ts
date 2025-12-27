import { browser } from "$app/environment";

export type Theme = "light" | "dark";

function createThemeState() {
    let theme = $state<Theme>("light");
    let colorblindMode = $state<boolean>(false);

    if (browser) {
        // Load initial state
        const savedTheme = localStorage.getItem("theme") as Theme;
        if (savedTheme) {
            theme = savedTheme;
        } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
            theme = "dark";
        }
        
        const savedColorblind = localStorage.getItem("colorblindMode") === "true";
        colorblindMode = savedColorblind;

        // Apply theme initially
        if (theme === "dark") {
            document.documentElement.classList.add("dark");
        } else {
            document.documentElement.classList.remove("dark");
        }

        if (colorblindMode) {
            document.documentElement.classList.add("colorblind");
        }
    }

    return {
        get current() {
            return theme;
        },
        set current(value: Theme) {
            theme = value;
            if (browser) {
                localStorage.setItem("theme", value);
                if (value === "dark") {
                    document.documentElement.classList.add("dark");
                } else {
                    document.documentElement.classList.remove("dark");
                }
            }
        },
        get colorblindMode() {
            return colorblindMode;
        },
        set colorblindMode(value: boolean) {
            colorblindMode = value;
            if (browser) {
                localStorage.setItem("colorblindMode", String(value));
                if (value) {
                    document.documentElement.classList.add("colorblind");
                } else {
                    document.documentElement.classList.remove("colorblind");
                }
            }
        },
        toggle() {
            this.current = theme === "light" ? "dark" : "light";
        },
        toggleColorblind() {
            this.colorblindMode = !colorblindMode;
        }
    };
}

export const themeState = createThemeState();
