<script lang="ts">
    import { onMount } from "svelte";
    import { fly, fade } from "svelte/transition";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { getCodelab, registerAttendee, loginWithGoogle, onAuthChange, isSupabaseMode, isServerlessMode, type Codelab } from "$lib/api";
    import { User, KeyRound, ArrowRight, Loader2, Chrome, Lock, X } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let id = page.params.id as string;
    let codelab = $state<Codelab | null>(null);
    let name = $state("");
    let email = $state("");
    let code = $state("");
    let loading = $state(true);
    let submitting = $state(false);
    let error = $state("");
    let errorType = $state("");
    const supabaseJoinKey = "supabase_oauth_join";

    onMount(async () => {
        let cleanup: (() => void) | undefined;
        try {
            const data = await getCodelab(id);
            codelab = data[0];

            // Check if already registered in this session
            const savedAttendee = localStorage.getItem(`attendee_${id}`);
            if (savedAttendee) {
                goto(`/codelabs/${id}`);
            }

            if (isSupabaseMode()) {
                cleanup = onAuthChange(async (user) => {
                    if (!user) return;
                    const pending = sessionStorage.getItem(supabaseJoinKey);
                    if (pending !== id) return;
                    sessionStorage.removeItem(supabaseJoinKey);

                    const displayName =
                        user.displayName ||
                        (user.email ? user.email.split("@")[0] : "") ||
                        $t("attendee.anonymous_user");
                    const userCode = user.uid ? user.uid.substring(0, 8) : "";

                    try {
                        const attendee = await registerAttendee(
                            id,
                            displayName,
                            userCode || "supabase",
                            user.email || undefined,
                        );
                        localStorage.setItem(
                            `attendee_${id}`,
                            JSON.stringify(attendee),
                        );
                        goto(`/codelabs/${id}`);
                    } catch (e: any) {
                        error = $t("attendee.error_registration_failed");
                    }
                });
            }
        } catch (e: any) {
            if (e.message === 'PRIVATE_CODELAB') {
                error = $t("attendee.error_private_codelab");
                errorType = "PRIVATE";
            } else {
                error = $t("attendee.codelab_not_found");
                errorType = "NOT_FOUND";
            }
        } finally {
            loading = false;
        }

        return cleanup;
    });

    async function handleSubmit() {
        if (!name || !code) {
            error = $t("attendee.error_fill_fields");
            return;
        }

        submitting = true;
        error = "";
        try {
            const attendee = await registerAttendee(id, name, code, email || undefined);
            localStorage.setItem(`attendee_${id}`, JSON.stringify(attendee));
            goto(`/codelabs/${id}`);
        } catch (e: any) {
            if (e.message === "DUPLICATE_NAME") {
                error = $t("attendee.error_duplicate_name");
            } else {
                error = $t("attendee.error_registration_failed");
            }
        } finally {
            submitting = false;
        }
    }

    async function handleGoogleLogin() {
        submitting = true;
        error = "";
        try {
            if (isSupabaseMode()) {
                sessionStorage.setItem(supabaseJoinKey, id);
                await loginWithGoogle();
                return;
            }
            const { user } = await loginWithGoogle();
            const emailPart = user.email ? user.email.split('@')[0] : "";
            const displayName = user.displayName || emailPart || $t("attendee.anonymous_user");
            const userCode = user.uid.substring(0, 8); // Use part of UID as code
            
            const attendee = await registerAttendee(id, displayName, userCode);
            localStorage.setItem(`attendee_${id}`, JSON.stringify(attendee));
            goto(`/codelabs/${id}`);
        } catch (e: any) {
            if (e.code !== 'auth/popup-closed-by-user') {
                error = $t("attendee.error_registration_failed") + ": " + e.message;
            }
        } finally {
            submitting = false;
        }
    }
</script>

<div
    class="min-h-screen bg-background dark:bg-dark-bg flex flex-col items-center justify-center p-6 font-sans text-foreground dark:text-dark-text"
>
    {#if loading}
        <div in:fade class="flex flex-col items-center gap-4">
            <Loader2 class="w-10 h-10 text-primary animate-spin" />
            <p class="text-muted-foreground dark:text-dark-text-muted font-medium">{$t("common.loading")}</p>
        </div>
    {:else if codelab}
        <div in:fly={{ y: 20, duration: 600 }} class="w-full max-w-md">
            <div
                class="bg-white dark:bg-dark-surface rounded-3xl shadow-xl shadow-primary/5 border border-border dark:border-dark-border overflow-hidden"
            >
                <div class="p-8 pb-4 text-center">
                    <div
                        class="w-16 h-16 bg-primary/10 dark:bg-primary/20 rounded-2xl flex items-center justify-center mx-auto mb-6"
                    >
                        <User class="w-8 h-8 text-primary" />
                    </div>
                    <h1 class="text-2xl font-bold text-foreground dark:text-dark-text mb-2">
                        {$t("attendee.join_title")}
                    </h1>
                    <p class="text-muted-foreground dark:text-dark-text-muted text-sm">
                        {$t("attendee.join_desc")} <br />
                        <span class="font-bold text-foreground dark:text-dark-text"
                            >{codelab.title}</span
                        >
                    </p>
                </div>

                <form
                    onsubmit={(e) => {
                        e.preventDefault();
                        handleSubmit();
                    }}
                    class="p-8 pt-4 space-y-6"
                >
                    {#if error}
                        <div
                            in:fade
                            class="p-3 bg-red-50 dark:bg-red-500/10 text-red-600 dark:text-red-400 text-sm rounded-xl border border-red-100 dark:border-red-500/20 font-medium"
                            role="alert"
                        >
                            {error}
                        </div>
                    {/if}

                    <div class="space-y-2">
                        <label
                            for="name"
                            class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider ml-1"
                        >
                            {$t("attendee.nickname")}
                        </label>
                        <div class="relative group">
                            <div
                                class="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground dark:text-dark-text-muted group-focus-within:text-primary transition-colors"
                            >
                                <User size={18} />
                            </div>
                            <input
                                id="name"
                                type="text"
                                bind:value={name}
                                placeholder={$t("attendee.nickname_placeholder")}
                                class="w-full pl-12 pr-4 py-3.5 bg-background dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary focus:ring-4 focus:ring-primary/10 transition-all text-foreground dark:text-dark-text placeholder-muted-foreground/60 dark:placeholder-dark-text-muted/60"
                                required
                            />
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label
                            for="email"
                            class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider ml-1"
                        >
                            {$t("attendee.email")} <span class="text-muted-foreground dark:text-dark-text-muted font-normal lowercase">({$t("common.optional")})</span>
                        </label>
                        <div class="relative group">
                            <div
                                class="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground dark:text-dark-text-muted group-focus-within:text-primary transition-colors"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="m2 7 8.97 5.7a1.94 1.94 0 0 0 2.06 0L22 7"/></svg>
                            </div>
                            <input
                                id="email"
                                type="email"
                                bind:value={email}
                                placeholder={$t("attendee.email_placeholder")}
                                class="w-full pl-12 pr-4 py-3.5 bg-background dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary focus:ring-4 focus:ring-primary/10 transition-all text-foreground dark:text-dark-text placeholder-muted-foreground/60 dark:placeholder-dark-text-muted/60"
                            />
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label
                            for="code"
                            class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-wider ml-1"
                        >
                            {$t("attendee.unique_code")}
                        </label>
                        <div class="relative group">
                            <div
                                class="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground dark:text-dark-text-muted group-focus-within:text-primary transition-colors"
                            >
                                <KeyRound size={18} />
                            </div>
                            <input
                                id="code"
                                type="text"
                                bind:value={code}
                                placeholder={$t("attendee.unique_code_placeholder")}
                                class="w-full pl-12 pr-4 py-3.5 bg-background dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl outline-none focus:border-primary focus:ring-4 focus:ring-primary/10 transition-all text-foreground dark:text-dark-text placeholder-muted-foreground/60 dark:placeholder-dark-text-muted/60"
                                required
                            />
                        </div>
                    </div>

                    <button
                        type="submit"
                        disabled={submitting}
                        class="w-full bg-primary hover:bg-primary/90 disabled:bg-border text-primary-foreground py-4 rounded-xl font-bold shadow-lg shadow-primary/20 hover:shadow-primary/30 transition-all flex items-center justify-center gap-2 group active:scale-[0.98]"
                    >
                        {#if submitting}
                            <Loader2 class="w-5 h-5 animate-spin" />
                            {$t("attendee.processing")}
                        {:else}
                            {$t("attendee.start_learning")}
                            <ArrowRight
                                class="w-5 h-5 group-hover:translate-x-1 transition-transform"
                            />
                        {/if}
                    </button>

                    {#if isServerlessMode()}
                        <div class="relative py-2">
                            <div class="absolute inset-0 flex items-center">
                                <div class="w-full border-t border-border dark:border-dark-border"></div>
                            </div>
                            <div class="relative flex justify-center text-xs uppercase">
                                <span class="bg-white dark:bg-dark-surface px-4 text-muted-foreground dark:text-dark-text-muted font-bold">{$t("common.or")}</span>
                            </div>
                        </div>

                        <button
                            type="button"
                            onclick={handleGoogleLogin}
                            disabled={submitting}
                            class="w-full bg-white dark:bg-dark-surface hover:bg-accent/60 dark:hover:bg-white/5 text-foreground dark:text-dark-text font-bold py-4 rounded-xl border border-border dark:border-dark-border shadow-sm hover:shadow-md transition-all flex items-center justify-center gap-3 active:scale-[0.98] disabled:opacity-50"
                        >
                            <Chrome size={20} class="text-primary" />
                            <span>{$t("common.google_login")}</span>
                        </button>
                    {/if}
                </form>
            </div>

            <p class="mt-8 text-center text-xs text-muted-foreground dark:text-dark-text-muted font-medium">
                Powered by Open Codelabs
            </p>
        </div>
    {:else}
        <div in:fade class="text-center max-w-sm px-6">
            <div
                class="w-20 h-20 {errorType === 'PRIVATE'
                    ? 'bg-amber-50 dark:bg-amber-500/10 text-amber-500 dark:text-amber-300 border-amber-100 dark:border-amber-500/30'
                    : 'bg-red-50 dark:bg-red-500/10 text-red-500 dark:text-red-400 border-red-100 dark:border-red-500/20'} rounded-full flex items-center justify-center mx-auto mb-6 border shadow-sm"
            >
                {#if errorType === "PRIVATE"}
                    <Lock size={40} />
                {:else}
                    <X size={40} />
                {/if}
            </div>
            <h1
                class="text-xl font-bold text-foreground dark:text-dark-text mb-8 break-keep leading-relaxed"
            >
                {error}
            </h1>
            <a
                href="/codelabs"
                class="inline-flex items-center gap-2 bg-primary text-primary-foreground px-8 py-3 rounded-xl font-bold hover:bg-primary/90 transition-all shadow-lg shadow-primary/20 active:scale-95"
            >
                {$t("attendee.return_home")}
            </a>
        </div>
    {/if}
</div>
