import { browser } from "$app/environment";

export type Theme = "light" | "dark";

function createThemeState() {
    let theme = $state<Theme>("light");

    if (browser) {
        // Load initial state
        const savedTheme = localStorage.getItem("theme") as Theme;
        if (savedTheme) {
            theme = savedTheme;
        } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
            theme = "dark";
        }
    }

    return {
        get current() {
            return theme;
        },
        set current(value: Theme) {
            theme = value;
        },
        toggle() {
            theme = theme === "light" ? "dark" : "light";
        }
    };
}

export const themeState = createThemeState();
