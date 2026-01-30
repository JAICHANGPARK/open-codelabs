import { browser } from "$app/environment";

class ThemeState {
    colorblindMode = $state<boolean>(false);

    constructor() {
        if (!browser) return;

        const savedColorblind = localStorage.getItem("colorblindMode") === "true";
        this.colorblindMode = savedColorblind;
        this.applyColorblind(this.colorblindMode);
    }

    private applyColorblind(value: boolean) {
        if (!browser) return;
        if (value) {
            document.documentElement.classList.add("colorblind");
        } else {
            document.documentElement.classList.remove("colorblind");
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

    toggleColorblind() {
        this.isColorblind = !this.colorblindMode;
    }
}

export const themeState = new ThemeState();
