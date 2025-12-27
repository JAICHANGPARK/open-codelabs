<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { locale, waitLocale, t } from "svelte-i18n";
    import "$lib/i18n";
    import "../app.css";
    import { Languages, LogOut, Sun, Moon, Github, FileText as FileIcon } from "lucide-svelte";
    import { themeState } from "$lib/theme.svelte";

    let { children } = $props();
    let i18nLoaded = $state(false);

    onMount(async () => {
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

    function handleLogout() {
        try {
            localStorage.removeItem("adminToken");
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

    function changeLanguage(code: string) {
        try {
            locale.set(code);
            localStorage.setItem("locale", code);
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
        <!-- External Links -->
        <div class="flex flex-col items-end gap-3 mb-2">
            <a
                href="https://github.com/JAICHANGPARK/open-codelabs"
                target="_blank"
                rel="noopener noreferrer"
                class="w-10 h-10 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
                title="GitHub Repository"
            >
                <Github size={18} />
            </a>
            <a
                href="https://jaichangpark.github.io/open-codelabs/"
                target="_blank"
                rel="noopener noreferrer"
                class="w-10 h-10 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
                title="Documentation"
            >
                <FileIcon size={18} />
            </a>
        </div>

        <div class="group relative">
            <button
                class="w-12 h-12 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
                title="Change Language"
            >
                <Languages size={20} />
            </button>
            <div
                class="absolute bottom-full right-0 mb-3 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-2xl overflow-hidden opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all min-w-[120px]"
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
        </div>

        <!-- Theme Toggle -->
        <button
            onclick={() => themeState.toggle()}
            class="w-12 h-12 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-full shadow-lg flex items-center justify-center text-[#5F6368] dark:text-dark-text-muted hover:text-[#4285F4] dark:hover:text-[#4285F4] transition-all"
            title="Toggle Theme"
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
