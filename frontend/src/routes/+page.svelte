<script>
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { LayoutDashboard, BookOpen } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let visible = $state(false);
    onMount(() => {
        visible = true;
    });
</script>

<div
    class="min-h-screen bg-background dark:bg-dark-bg flex flex-col items-center justify-center p-6 font-sans transition-colors"
>
    {#if visible}
        <div in:fly={{ y: 20, duration: 800 }} class="text-center max-w-2xl">
            <h1
                class="text-5xl font-extrabold text-foreground dark:text-dark-text mb-4 tracking-tight"
            >
                Open<span class="text-primary">Codelabs</span>
            </h1>
            <p class="text-xl text-muted-foreground dark:text-dark-text-muted mb-12">
                {$t("home.subtitle")}
            </p>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full">
                <!-- Facilitator Access -->
                <a
                    href="/admin"
                    class="group p-8 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-sm hover:shadow-md hover:border-primary transition-all duration-300 flex flex-col items-center text-center"
                >
                    <div
                        class="p-4 bg-primary/10 rounded-full mb-4 group-hover:bg-primary transition-colors duration-300"
                    >
                        <LayoutDashboard
                            class="w-8 h-8 text-primary group-hover:text-primary-foreground"
                        />
                    </div>
                    <h3 class="text-xl font-bold text-foreground dark:text-dark-text mb-2">
                        {$t("common.facilitator")}
                    </h3>
                    <p class="text-muted-foreground dark:text-dark-text-muted text-sm">
                        {$t("home.facilitator_desc")}
                    </p>
                </a>

                <!-- Attendee Access -->
                <a
                    href="/codelabs"
                    class="group p-8 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-2xl shadow-sm hover:shadow-md hover:border-primary transition-all duration-300 flex flex-col items-center text-center"
                >
                    <div
                        class="p-4 bg-primary/10 rounded-full mb-4 group-hover:bg-primary transition-colors duration-300"
                    >
                        <BookOpen
                            class="w-8 h-8 text-primary group-hover:text-primary-foreground"
                        />
                    </div>
                    <h3 class="text-xl font-bold text-foreground dark:text-dark-text mb-2">
                        {$t("common.attendee")}
                    </h3>
                    <p class="text-muted-foreground dark:text-dark-text-muted text-sm">
                        {$t("home.attendee_desc")}
                    </p>
                </a>
            </div>
        </div>
    {/if}
</div>

<style>
    :global(body) {
        margin: 0;
        background-color: var(--background);
    }
</style>
