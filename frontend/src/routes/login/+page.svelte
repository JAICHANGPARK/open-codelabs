<script lang="ts">
    import { onMount } from "svelte";
    import { login, loginWithGoogle, isSupabaseMode, isServerlessMode, getSession } from "$lib/api";
    import { goto } from "$app/navigation";
    import { Lock, User, LogIn, AlertCircle, Github, FileText as FileIcon, Chrome, X } from "lucide-svelte";
    import { fade, fly } from "svelte/transition";
    import { t } from "svelte-i18n";

    let admin_id = $state("");
    let admin_pw = $state("");
    let error = $state("");
    let loading = $state(false);
    let showTrouble = $state(false);
    const supabaseRedirectKey = "supabase_oauth_redirect";

    onMount(async () => {
        if (!isSupabaseMode()) return;
        try {
            const session = await getSession();
            if (!session) return;
            const redirectTo = sessionStorage.getItem(supabaseRedirectKey);
            if (redirectTo) {
                sessionStorage.removeItem(supabaseRedirectKey);
                goto(redirectTo);
                return;
            }
            goto("/admin");
        } catch (e) {
            console.error("Supabase session check failed", e);
        }
    });

    async function handleLogin() {
        if (!admin_id || !admin_pw) {
            error = $t("login.error_fields");
            return;
        }
        loading = true;
        error = "";
        try {
            const result = await login(admin_id, admin_pw);
            if (isServerlessMode() && (result as any)?.token) {
                localStorage.setItem("adminToken", (result as any).token);
            }
            sessionStorage.setItem("adminPassword", admin_pw);
            if (typeof window !== "undefined") {
                window.dispatchEvent(new Event("session-changed"));
            }
            goto("/admin");
        } catch (e) {
            error = $t("login.error_credentials");
        } finally {
            loading = false;
        }
    }

    async function handleGoogleLogin() {
        loading = true;
        error = "";
        try {
            if (isSupabaseMode()) {
                sessionStorage.setItem(supabaseRedirectKey, "/admin");
                await loginWithGoogle();
                return;
            }
            const { token, user } = await loginWithGoogle();
            localStorage.setItem("adminToken", token);
            // Store user info if needed
            localStorage.setItem("user", JSON.stringify({
                uid: user.uid,
                email: user.email,
                displayName: user.displayName,
                photoURL: user.photoURL
            }));
            if (typeof window !== "undefined") {
                window.dispatchEvent(new Event("session-changed"));
            }
            goto("/admin");
        } catch (e: any) {
            if (e.code !== 'auth/popup-closed-by-user') {
                error = "Google login failed: " + e.message;
            }
        } finally {
            loading = false;
        }
    }
</script>

<div class="min-h-screen bg-background dark:bg-dark-bg flex items-center justify-center p-6 transition-colors">
    <div class="w-full max-w-md" in:fly={{ y: 20, duration: 600 }}>
        <div
            class="bg-white dark:bg-dark-surface rounded-[2rem] shadow-2xl overflow-hidden border border-border dark:border-dark-border"
        >
            <div class="bg-primary p-10 text-primary-foreground text-center">
                <div
                    class="w-16 h-16 bg-white/20 rounded-2xl flex items-center justify-center mx-auto mb-6 backdrop-blur-sm"
                >
                    <LogIn size={32} />
                </div>
                <h1 class="text-3xl font-bold mb-2">{$t("login.title")}</h1>
                <p class="text-primary-foreground/80 font-medium">{$t("login.subtitle")}</p>
            </div>

            <div class="p-8 sm:p-10 space-y-6 sm:space-y-8">
                {#if error}
                    <div
                        class="bg-red-50 dark:bg-red-500/10 text-red-600 dark:text-red-400 p-4 rounded-xl flex items-center gap-3 border border-red-100 dark:border-red-500/20"
                        role="alert"
                        in:fade
                    >
                        <AlertCircle size={20} />
                        <span class="text-sm font-bold">{error}</span>
                    </div>
                {/if}

                <div class="space-y-6">
                    <div>
                        <label
                            for="admin_id"
                            class="block text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-widest mb-2 px-1"
                            >{$t("login.admin_id")}</label
                        >
                        <div class="relative group">
                            <div
                                class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-muted-foreground/70 dark:text-dark-text-muted group-focus-within:text-primary transition-colors"
                            >
                                <User size={20} />
                            </div>
                            <input
                                id="admin_id"
                                type="text"
                                bind:value={admin_id}
                                placeholder={$t("login.placeholder_id")}
                                class="w-full bg-background dark:bg-dark-bg border-2 border-border dark:border-dark-border rounded-2xl pl-12 pr-4 py-4 focus:border-primary outline-none transition-all placeholder-muted-foreground/60 dark:placeholder-dark-text-muted/30 font-medium text-foreground dark:text-dark-text"
                                onkeydown={(e) =>
                                    e.key === "Enter" && handleLogin()}
                            />
                        </div>
                    </div>

                    <div>
                        <label
                            for="admin_pw"
                            class="block text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-widest mb-2 px-1"
                            >{$t("login.password")}</label
                        >
                        <div class="relative group">
                            <div
                                class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-muted-foreground/70 dark:text-dark-text-muted group-focus-within:text-primary transition-colors"
                            >
                                <Lock size={20} />
                            </div>
                            <input
                                id="admin_pw"
                                type="password"
                                bind:value={admin_pw}
                                placeholder="••••••••"
                                class="w-full bg-background dark:bg-dark-bg border-2 border-border dark:border-dark-border rounded-2xl pl-12 pr-4 py-4 focus:border-primary outline-none transition-all placeholder-muted-foreground/60 dark:placeholder-dark-text-muted/30 font-medium text-foreground dark:text-dark-text"
                                onkeydown={(e) =>
                                    e.key === "Enter" && handleLogin()}
                            />
                        </div>
                    </div>
                </div>

                <button
                    onclick={handleLogin}
                    disabled={loading}
                    class="w-full bg-primary hover:bg-primary/90 text-primary-foreground font-bold py-4 sm:py-5 rounded-2xl shadow-lg hover:shadow-xl transition-all active:scale-[0.98] disabled:opacity-50 flex items-center justify-center gap-3 text-lg"
                >
                    {#if loading}
                        <div
                            class="w-6 h-6 border-3 border-white border-t-transparent animate-spin rounded-full"
                        ></div>
                        {$t("login.connecting")}
                    {:else}
                        <span>{$t("login.sign_in")}</span>
                        <LogIn size={20} />
                    {/if}
                </button>

                {#if isServerlessMode()}
                    <div class="relative py-2">
                        <div class="absolute inset-0 flex items-center">
                            <div class="w-full border-t border-border dark:border-dark-border"></div>
                        </div>
                        <div class="relative flex justify-center text-xs uppercase">
                            <span class="bg-white dark:bg-dark-surface px-4 text-muted-foreground font-bold">{$t("common.or")}</span>
                        </div>
                    </div>

                    <button
                        onclick={handleGoogleLogin}
                        disabled={loading}
                        class="w-full bg-white dark:bg-dark-surface hover:bg-accent/60 dark:hover:bg-accent/40 text-foreground dark:text-dark-text font-bold py-4 rounded-2xl border-2 border-border dark:border-dark-border shadow-sm hover:shadow-md transition-all active:scale-[0.98] disabled:opacity-50 flex items-center justify-center gap-3 text-lg"
                    >
                        <Chrome size={20} class="text-primary" />
                        <span>{$t("common.google_login")}</span>
                    </button>
                {/if}

                <div class="pt-2 text-center">
                    <button
                        class="text-sm font-bold text-muted-foreground dark:text-dark-text-muted hover:text-primary transition-colors"
                        onclick={() => (showTrouble = true)}
                    >
                        {$t("login.trouble")}
                    </button>
                </div>
            </div>
        </div>

        {#if showTrouble}
            <div class="fixed inset-0 z-40 flex items-center justify-center px-4" role="dialog" aria-modal="true">
                <button
                    class="absolute inset-0 bg-black/40"
                    aria-label={$t("common.close")}
                    onclick={() => (showTrouble = false)}
                ></button>
                <div class="relative w-full max-w-lg bg-white dark:bg-dark-surface rounded-2xl shadow-2xl border border-border dark:border-dark-border p-6" in:fade>
                    <div class="flex items-start justify-between gap-4 mb-4">
                        <div>
                            <h2 class="text-lg font-bold text-foreground dark:text-dark-text">
                                {$t("login.trouble_title")}
                            </h2>
                            {#if isServerlessMode()}
                                <p class="text-sm text-muted-foreground dark:text-dark-text-muted mt-1">
                                    {$t("login.trouble_serverless_desc")}
                                </p>
                            {:else}
                                <p class="text-sm text-muted-foreground dark:text-dark-text-muted mt-1">
                                    {$t("login.trouble_local_desc")}
                                </p>
                            {/if}
                        </div>
                        <button
                            class="p-2 rounded-full hover:bg-accent/60 dark:hover:bg-accent/40 transition-colors"
                            aria-label={$t("common.close")}
                            onclick={() => (showTrouble = false)}
                        >
                            <X size={18} />
                        </button>
                    </div>

                    {#if isServerlessMode()}
                        <div class="text-sm text-muted-foreground dark:text-dark-text-muted">
                            {$t("login.trouble_serverless_hint")}
                        </div>
                    {:else}
                        <div class="space-y-4">
                            <div>
                                <div class="text-xs font-bold uppercase tracking-widest text-muted-foreground dark:text-dark-text-muted">
                                    {$t("login.trouble_local_env_title")}
                                </div>
                                <p class="text-sm text-muted-foreground dark:text-dark-text-muted mt-1">
                                    {$t("login.trouble_local_env_desc")}
                                </p>
                                <pre class="mt-2 text-xs bg-accent/60 dark:bg-dark-bg text-foreground dark:text-dark-text rounded-xl p-3 overflow-auto border border-border dark:border-dark-border">ADMIN_ID=your_admin_id
ADMIN_PW=your_admin_pw
DATABASE_URL=sqlite://backend.db</pre>
                            </div>
                            <div>
                                <div class="text-xs font-bold uppercase tracking-widest text-muted-foreground dark:text-dark-text-muted">
                                    {$t("login.trouble_local_commands_title")}
                                </div>
                                <p class="text-sm text-muted-foreground dark:text-dark-text-muted mt-1">
                                    {$t("login.trouble_local_commands_desc")}
                                </p>
                                <pre class="mt-2 text-xs bg-accent/60 dark:bg-dark-bg text-foreground dark:text-dark-text rounded-xl p-3 overflow-auto border border-border dark:border-dark-border">cd backend && cargo run
cd frontend && bun run dev</pre>
                            </div>
                            <p class="text-xs text-muted-foreground dark:text-dark-text-muted">
                                {$t("login.trouble_local_note")}
                            </p>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}

        <div class="mt-8 flex flex-col items-center gap-4">
            <div class="flex items-center gap-4">
                <a
                    href="https://github.com/JAICHANGPARK/open-codelabs"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="flex items-center gap-2 text-muted-foreground dark:text-dark-text-muted hover:text-primary transition-colors text-sm font-medium"
                >
                    <Github size={18} />
                    GitHub
                </a>
                <div class="w-px h-4 bg-border dark:bg-dark-border"></div>
                <a
                    href="https://jaichangpark.github.io/open-codelabs/"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="flex items-center gap-2 text-muted-foreground dark:text-dark-text-muted hover:text-primary transition-colors text-sm font-medium"
                >
                    <FileIcon size={18} />
                    Docs
                </a>
            </div>
            <p class="text-muted-foreground dark:text-dark-text-muted text-sm font-medium">
                {$t("common.title")} &copy; 2026
            </p>
        </div>
    </div>
</div>
