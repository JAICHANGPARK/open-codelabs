<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { browser } from "$app/environment";
    import { locale, waitLocale, t } from "svelte-i18n";
    import "$lib/i18n";
    import "../app.css";
    import { Languages, LogOut, Sun, Moon, Github, FileText as FileIcon, Eye } from "lucide-svelte";
    import { themeState } from "$lib/theme.svelte";
    import { logout, onAuthChange, isFirebaseMode } from "$lib/api";

    let { children } = $props();
    let i18nLoaded = $state(false);

    onMount(async () => {
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
            const token = localStorage.getItem("adminToken");
            const isProtectedPath = pathname.startsWith("/admin");

            if (isProtectedPath && !token) {
                goto("/login");
            }
        } catch (e) {
            console.error("Auth check failed", e);
        }
    });

    async function handleLogout() {
        try {
            await logout();
            localStorage.removeItem("adminToken");
            localStorage.removeItem("user");
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

    function changeLanguage(code: string) {
        try {
            locale.set(code);
            localStorage.setItem("locale", code);
            langMenuOpen = false;
        } catch (e) {
            console.error("Language change failed", e);
        }
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
<!--                class="w-10 h-10 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"-->
<!--                title={$t("common.github_repo")}-->
<!--                aria-label={$t("common.github_repo")}-->
<!--            >-->
<!--                <Github size={18} />-->
<!--            </a>-->
<!--            <a-->
<!--                href="https://jaichangpark.github.io/open-codelabs/"-->
<!--                target="_blank"-->
<!--                rel="noopener noreferrer"-->
<!--                class="w-10 h-10 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"-->
<!--                title={$t("common.documentation")}-->
<!--                aria-label={$t("common.documentation")}-->
<!--            >-->
<!--                <FileIcon size={18} />-->
<!--            </a>-->
<!--        </div>-->

        <div class="relative">
            <button
                onclick={() => langMenuOpen = !langMenuOpen}
                class="w-12 h-12 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
                title={$t("common.change_language")}
                aria-label={$t("common.change_language")}
                aria-haspopup="true"
                aria-expanded={langMenuOpen}
            >
                <Languages size={20} />
            </button>
            {#if langMenuOpen}
                <div
                    class="absolute bottom-full right-0 mb-3 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-2xl overflow-hidden min-w-[120px]"
                >
                    {#each availableLocales as loc}
                        <button
                            onclick={() => changeLanguage(loc.code)}
                            class="w-full text-left px-4 py-3 text-sm font-bold hover:bg-[#F8F9FA] dark:hover:bg-white/5 transition-colors {loc.code ===
                            $locale
                                ? 'text-[#4285F4] bg-[#E8F0FE] dark:bg-[#4285F4]/10'
                                : 'text-[#5F6368] dark:text-dark-text-muted'}"
                        >
                            {loc.name}
                        </button>
                    {/each}
                </div>
            {/if}
        </div>
        
        <!-- Colorblind Mode Toggle -->
        <button
            onclick={() => themeState.toggleColorblind()}
            class="w-12 h-12 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all {themeState.isColorblind ? 'ring-2 ring-[#4285F4] border-transparent' : ''}"
            title={$t("common.toggle_colorblind")}
            aria-label={$t("common.toggle_colorblind")}
            aria-pressed={themeState.isColorblind}
        >
            <span class="sr-only">{$t("common.toggle_colorblind")}</span>
            <Eye size={20} class={themeState.isColorblind ? "text-[#4285F4]" : ""} />
        </button>

        <!-- Theme Toggle -->
        <button
            onclick={() => themeState.toggle()}
            class="w-12 h-12 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
            title={$t("common.toggle_theme")}
            aria-label={$t("common.toggle_theme")}
        >
            {#if themeState.current === 'light'}
                <Moon size={20} />
            {:else}
                <Sun size={20} />
            {/if}
        </button>

        <!-- Logout button if in admin -->
        {#if page.url.pathname.startsWith("/admin")}
            <button
                onclick={handleLogout}
                class="w-12 h-12 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-red-400 hover:text-red-500 hover:border-red-500 transition-all"
                title={$t("common.logout")}
                aria-label={$t("common.logout")}
            >
                <LogOut size={20} />
            </button>
        {/if}
    </div>
{:else}
    <div class="min-h-screen flex items-center justify-center bg-[#F8F9FA] dark:bg-dark-bg">
        <div
            class="animate-spin rounded-full h-12 w-12 border-4 border-[#E8EAED] dark:border-dark-border border-t-[#4285F4] dark:border-t-[#4285F4]"
        ></div>
    </div>
{/if}
