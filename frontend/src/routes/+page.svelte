<script>
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { LayoutDashboard, BookOpen } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let visible = false;
    onMount(() => {
        visible = true;
    });
</script>

<div
    class="min-h-screen bg-[#F8F9FA] flex flex-col items-center justify-center p-6 font-sans"
>
    {#if visible}
        <div in:fly={{ y: 20, duration: 800 }} class="text-center max-w-2xl">
            <h1
                class="text-5xl font-extrabold text-[#3C4043] mb-4 tracking-tight"
            >
                AntiGravity <span class="text-[#4285F4]">Dojo</span>
            </h1>
            <p class="text-xl text-[#5F6368] mb-12">
                {$t("home.subtitle")}
            </p>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full">
                <!-- Facilitator Access -->
                <a
                    href="/admin"
                    class="group p-8 bg-white border border-[#DADCE0] rounded-2xl shadow-sm hover:shadow-md hover:border-[#4285F4] transition-all duration-300 flex flex-col items-center text-center"
                >
                    <div
                        class="p-4 bg-[#4285F4]/10 rounded-full mb-4 group-hover:bg-[#4285F4] transition-colors duration-300"
                    >
                        <LayoutDashboard
                            class="w-8 h-8 text-[#4285F4] group-hover:text-white"
                        />
                    </div>
                    <h3 class="text-xl font-bold text-[#3C4043] mb-2">
                        {$t("common.facilitator")}
                    </h3>
                    <p class="text-[#5F6368] text-sm">
                        {$t("home.facilitator_desc")}
                    </p>
                </a>

                <!-- Attendee Access -->
                <a
                    href="/codelabs"
                    class="group p-8 bg-white border border-[#DADCE0] rounded-2xl shadow-sm hover:shadow-md hover:border-[#4285F4] transition-all duration-300 flex flex-col items-center text-center"
                >
                    <div
                        class="p-4 bg-[#34A853]/10 rounded-full mb-4 group-hover:bg-[#34A853] transition-colors duration-300"
                    >
                        <BookOpen
                            class="w-8 h-8 text-[#34A853] group-hover:text-white"
                        />
                    </div>
                    <h3 class="text-xl font-bold text-[#3C4043] mb-2">
                        {$t("common.attendee")}
                    </h3>
                    <p class="text-[#5F6368] text-sm">
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
        background-color: #f8f9fa;
    }
</style>
