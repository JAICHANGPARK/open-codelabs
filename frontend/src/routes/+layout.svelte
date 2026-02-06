<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { browser } from "$app/environment";
    import { locale, waitLocale, t } from "svelte-i18n";
    import "$lib/i18n";
    import "../app.css";
    import { Languages, LogOut, Sun, Moon, Github, FileText as FileIcon, Accessibility, Palette } from "lucide-svelte";
    import { themeState } from "$lib/theme.svelte";
    import { logout, onAuthChange, isFirebaseMode, isSupabaseMode, isServerlessMode, getSession } from "$lib/api";

    let { children } = $props();
    let i18nLoaded = $state(false);
    let sessionRole = $state<string | null>(null);
    let sessionChecked = $state(false);
    let sessionRefreshing = $state(false);

    async function refreshSession() {
        if (sessionRefreshing || !browser || isServerlessMode()) return;
        sessionRefreshing = true;
        try {
            const session = await getSession();
            sessionRole = session?.role ?? null;
        } catch (e) {
            sessionRole = null;
        } finally {
            sessionChecked = true;
            sessionRefreshing = false;
        }
    }

    onMount(async () => {
        let cleanup: (() => void) | undefined;
        if (isFirebaseMode()) {
            onAuthChange((user) => {
                if (!user && page.url.pathname.startsWith("/admin")) {
                    localStorage.removeItem("adminToken");
                    localStorage.removeItem("user");
                    goto("/login");
                } else if (user) {
                    // Sync token if needed
                    user.getIdToken().then(token => {
                        localStorage.setItem("adminToken", token);
                        localStorage.setItem("user", JSON.stringify({
                            uid: user.uid,
                            email: user.email,
                            displayName: user.displayName,
                            photoURL: user.photoURL
                        }));
                    });
                }
            });
        } else if (isSupabaseMode()) {
            cleanup = onAuthChange((user) => {
                if (!user && page.url.pathname.startsWith("/admin")) {
                    localStorage.removeItem("adminToken");
                    localStorage.removeItem("user");
                    goto("/login");
                } else if (user) {
                    const token = (user as any).accessToken || "supabase";
                    localStorage.setItem("adminToken", token);
                    localStorage.setItem("user", JSON.stringify(user));
                }
            });
        } else {
            await refreshSession();
            const handler = () => {
                refreshSession();
            };
            window.addEventListener("session-changed", handler);
            cleanup = () => window.removeEventListener("session-changed", handler);
        }
        try {
            const savedLocale = localStorage.getItem("locale");
            if (savedLocale) {
                locale.set(savedLocale);
            }
        } catch (e) {
            console.warn("localStorage not available", e);
        }

        try {
            // Wait for locale to load, but don't hang for more than 500ms
            await Promise.race([
                waitLocale(),
                new Promise((resolve) => setTimeout(resolve, 500)),
            ]);
        } catch (e) {
            console.warn("i18n load issue", e);
        } finally {
            i18nLoaded = true;
        }

        return cleanup;
    });

    $effect(() => {
        if (!i18nLoaded) return;

        // Update html lang attribute
        if (browser && $locale) {
            document.documentElement.lang = $locale;
        }

        // Track pathname for reactivity
        const pathname = page.url.pathname;
        try {
            const isProtectedPath = pathname.startsWith("/admin");

            if (isServerlessMode()) {
                const token = localStorage.getItem("adminToken");
                if (isProtectedPath && !token) {
                    goto("/login");
                }
            } else if (isProtectedPath) {
                if (sessionRefreshing) {
                    return;
                }
                if (!sessionChecked) {
                    refreshSession();
                } else if (sessionChecked && sessionRole !== "admin") {
                    goto("/login");
                }
            }
        } catch (e) {
            console.error("Auth check failed", e);
        }
    });

    async function handleLogout() {
        try {
            await logout();
            if (isServerlessMode()) {
                localStorage.removeItem("adminToken");
                localStorage.removeItem("user");
            }
            sessionRole = null;
            if (typeof window !== "undefined") {
                window.dispatchEvent(new Event("session-changed"));
            }
            goto("/login");
        } catch (e) {
            console.error("Logout failed", e);
        }
    }

    const availableLocales = [
        { code: "en", name: "English" },
        { code: "ko", name: "한국어" },
        { code: "ja", name: "日本語" },
        { code: "zh", name: "中文" },
    ];

    let langMenuOpen = $state(false);
    let themeMenuOpen = $state(false);

    const availableThemeModes = [
        { id: "system", labelKey: "theme.modes.system" },
        { id: "light", labelKey: "theme.modes.light" },
        { id: "dark", labelKey: "theme.modes.dark" },
    ] as const;

    function changeLanguage(code: string) {
        try {
            locale.set(code);
            localStorage.setItem("locale", code);
            langMenuOpen = false;
        } catch (e) {
            console.error("Language change failed", e);
        }
    }

    function selectThemeMode(mode: "system" | "light" | "dark") {
        themeState.setMode(mode);
        themeMenuOpen = false;
    }

    function selectThemePreset(presetId: "default" | "mint" | "ocean" | "sunset") {
        themeState.setPreset(presetId);
        themeMenuOpen = false;
    }
</script>

<svelte:head>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link
        rel="preconnect"
        href="https://fonts.gstatic.com"
        crossorigin="anonymous"
    />
    <link
        href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;700;800&family=JetBrains+Mono:wght@400;700&display=swap"
        rel="stylesheet"
    />
</svelte:head>

{#if i18nLoaded}
    {@render children()}

    <!-- Language Selector Floating Dial -->
    <div
        class="fixed bottom-6 right-6 z-50 flex flex-col items-end gap-3 print:hidden"
    >
<!--        &lt;!&ndash; External Links &ndash;&gt;-->
<!--        <div class="flex flex-col items-end gap-3 mb-2">-->
<!--            <a-->
<!--                href="https://github.com/JAICHANGPARK/open-codelabs"-->
<!--                target="_blank"-->
<!--                rel="noopener noreferrer"-->
<!--                class="w-10 h-10 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-muted-foreground dark:text-dark-text-muted hover:text-primary dark:hover:text-primary transition-all"-->
<!--                title={$t("common.github_repo")}-->
<!--                aria-label={$t("common.github_repo")}-->
<!--            >-->
<!--                <Github size={18} />-->
<!--            </a>-->
<!--            <a-->
<!--                href="https://jaichangpark.github.io/open-codelabs/"-->
<!--                target="_blank"-->
<!--                rel="noopener noreferrer"-->
<!--                class="w-10 h-10 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-muted-foreground dark:text-dark-text-muted hover:text-primary dark:hover:text-primary transition-all"-->
<!--                title={$t("common.documentation")}-->
<!--                aria-label={$t("common.documentation")}-->
<!--            >-->
<!--                <FileIcon size={18} />-->
<!--            </a>-->
<!--        </div>-->

        <div class="relative">
            <button
                onclick={() => langMenuOpen = !langMenuOpen}
                class="w-12 h-12 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-muted-foreground dark:text-dark-text-muted hover:text-primary transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-dark-bg"
                title={$t("common.change_language")}
                aria-label={$t("common.change_language")}
                aria-haspopup="true"
                aria-expanded={langMenuOpen}
            >
                <Languages size={20} />
            </button>
            {#if langMenuOpen}
                <div
                    class="absolute bottom-full right-0 mb-3 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-2xl overflow-hidden min-w-[120px]"
                >
                    {#each availableLocales as loc}
                        <button
                            onclick={() => changeLanguage(loc.code)}
                            class="w-full text-left px-4 py-3 text-sm font-bold hover:bg-accent/60 dark:hover:bg-accent/40 transition-colors {loc.code ===
                            $locale
                                ? 'text-primary bg-accent'
                                : 'text-muted-foreground dark:text-dark-text-muted'}"
                        >
                            {loc.name}
                        </button>
                    {/each}
                </div>
            {/if}
        </div>

        <div class="relative">
            <button
                onclick={() => themeMenuOpen = !themeMenuOpen}
                class="w-12 h-12 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-muted-foreground dark:text-dark-text-muted hover:text-primary transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-dark-bg"
                title={$t("theme.menu_label")}
                aria-label={$t("theme.menu_label")}
                aria-haspopup="true"
                aria-expanded={themeMenuOpen}
            >
                <Palette size={20} />
            </button>
            {#if themeMenuOpen}
                <div
                    class="absolute bottom-full right-0 mb-3 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-2xl overflow-hidden min-w-[180px]"
                >
                    <div class="px-4 py-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground dark:text-dark-text-muted bg-accent/70 dark:bg-accent/40 border-b border-border dark:border-dark-border">
                        {$t("theme.mode_label")}
                    </div>
                    {#each availableThemeModes as mode}
                        <button
                            onclick={() => selectThemeMode(mode.id)}
                            class="w-full text-left px-4 py-2 text-sm font-bold hover:bg-accent/60 dark:hover:bg-accent/40 transition-colors {themeState.modeId === mode.id
                                ? 'text-primary bg-accent'
                                : 'text-muted-foreground dark:text-dark-text-muted'}"
                        >
                            {$t(mode.labelKey)}
                        </button>
                    {/each}
                    <div class="px-4 py-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground dark:text-dark-text-muted bg-accent/70 dark:bg-accent/40 border-b border-t border-border dark:border-dark-border">
                        {$t("theme.preset_label")}
                    </div>
                    {#each themeState.presets as preset}
                        <button
                            onclick={() => selectThemePreset(preset.id)}
                            class="w-full text-left px-4 py-2 text-sm font-bold hover:bg-accent/60 dark:hover:bg-accent/40 transition-colors {themeState.presetId === preset.id
                                ? 'text-primary bg-accent'
                                : 'text-muted-foreground dark:text-dark-text-muted'}"
                        >
                            {$t(preset.labelKey)}
                        </button>
                    {/each}
                </div>
            {/if}
        </div>
        
        <!-- Colorblind Mode Toggle -->
        <button
            onclick={() => themeState.toggleColorblind()}
            class="w-12 h-12 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-muted-foreground dark:text-dark-text-muted hover:text-primary transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-dark-bg {themeState.isColorblind ? 'ring-2 ring-ring border-transparent' : ''}"
            title={$t("common.toggle_colorblind")}
            aria-label={$t("common.toggle_colorblind")}
            aria-pressed={themeState.isColorblind}
        >
            <span class="sr-only">{$t("common.toggle_colorblind")}</span>
            <Accessibility size={20} class={themeState.isColorblind ? "text-primary" : ""} />
        </button>

        <!-- Theme Toggle -->
        <button
            onclick={() => themeState.toggleMode()}
            class="relative w-12 h-12 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-muted-foreground dark:text-dark-text-muted hover:text-primary transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-dark-bg"
            title={$t("common.toggle_theme")}
            aria-label={$t("common.toggle_theme")}
        >
            <Sun
                size={20}
                class="scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90"
            />
            <Moon
                size={20}
                class="absolute scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0"
            />
        </button>

        <!-- Logout button if in admin -->
        {#if page.url.pathname.startsWith("/admin")}
            <button
                onclick={handleLogout}
                class="w-12 h-12 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-muted-foreground dark:text-red-400 hover:text-red-500 hover:border-red-500 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500 focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-dark-bg"
                title={$t("common.logout")}
                aria-label={$t("common.logout")}
            >
                <LogOut size={20} />
            </button>
        {/if}
    </div>
{:else}
    <div class="min-h-screen flex items-center justify-center bg-background dark:bg-dark-bg">
        <div
            class="animate-spin rounded-full h-12 w-12 border-4 border-border dark:border-dark-border border-t-primary dark:border-t-primary"
        ></div>
    </div>
{/if}
