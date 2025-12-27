import { browser } from "$app/environment";

export type Theme = "light" | "dark";

function createThemeState() {
    let theme = $state<Theme>("light");

    if (browser) {
        $effect.root(() => {
            $effect(() => {
                const savedTheme = localStorage.getItem("theme") as Theme;
                if (savedTheme) {
                    theme = savedTheme;
                } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
                    theme = "dark";
                }
                applyTheme(theme);
            });
        });
    }

    function applyTheme(newTheme: Theme) {
        if (!browser) return;
        if (newTheme === "dark") {
            document.documentElement.classList.add("dark");
        } else {
            document.documentElement.classList.remove("dark");
        }
        localStorage.setItem("theme", newTheme);
    }

    return {
        get current() {
            return theme;
        },
        set current(value: Theme) {
            theme = value;
            applyTheme(value);
        },
        toggle() {
            this.current = theme === "light" ? "dark" : "light";
        }
    };
}

export const themeState = createThemeState();
