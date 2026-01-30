<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { listCodelabs, getJoinedCodelabs, onAuthChange, type Codelab, isServerlessMode } from "$lib/api";
    import {
        BookOpen,
        User,
        Clock,
        Search,
        ArrowRight,
        Loader2,
        Star,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";

    let codelabs: Codelab[] = $state([]);
    let joinedCodelabs: Codelab[] = $state([]);
    let loading = $state(true);
    let searchQuery = $state("");
    let user = $state<any>(null);

    onMount(async () => {
        onAuthChange((u) => {
            user = u;
            if (isServerlessMode() && u) {
                loadJoinedCodelabs();
            }
        });

        try {
            codelabs = await listCodelabs();
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    async function loadJoinedCodelabs() {
        try {
            joinedCodelabs = await getJoinedCodelabs();
        } catch (e) {
            console.error("Failed to load joined codelabs", e);
        }
    }

    let filteredCodelabs = $derived(
        codelabs.filter(
            (c) =>
                c.is_public &&
                (c.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                    c.description
                        .toLowerCase()
                        .includes(searchQuery.toLowerCase())),
        ),
    );
</script>

<div class="min-h-screen bg-[#F8F9FA] flex flex-col font-sans text-[#3C4043]">
    <header
        class="bg-white border-b border-[#E8EAED] py-6 px-8 sticky top-0 z-30 shadow-sm"
    >
        <div
            class="max-w-6xl mx-auto flex flex-col md:flex-row justify-between items-center gap-6"
        >
            <div class="flex items-center gap-4">
                <a href="/" class="flex items-center gap-3">
                    <div
                        class="w-10 h-10 bg-[#34A853] rounded-lg flex items-center justify-center text-white shadow-sm"
                    >
                        <BookOpen size={24} />
                    </div>
                    <div>
                        <h1 class="text-2xl font-bold text-[#202124]">
                            Open-Codelabs <span class="text-[#34A853]"
                                >Dojo</span
                            >
                        </h1>
                        <p
                            class="text-xs text-[#5F6368] font-bold uppercase tracking-widest"
                        >
                            Attendee Portal
                        </p>
                    </div>
                </a>
            </div>

            <div class="relative w-full max-w-md">
                <Search
                    class="absolute left-4 top-1/2 -translate-y-1/2 text-[#9AA0A6]"
                    size={20}
                />
                <input
                    type="text"
                    bind:value={searchQuery}
                    placeholder="Search codelabs..."
                    class="w-full pl-12 pr-4 py-3 bg-[#F1F3F4] border-transparent rounded-full outline-none focus:bg-white focus:ring-4 focus:ring-[#34A853]/10 transition-all text-[#202124]"
                />
            </div>
        </div>
    </header>

    <main class="max-w-6xl mx-auto w-full p-8 lg:p-12 flex-1">
        {#if loading}
            <div class="flex flex-col items-center justify-center py-32 gap-4">
                <Loader2 class="w-10 h-10 text-[#34A853] animate-spin" />
                <p class="text-[#5F6368] font-medium">{$t("common.loading")}</p>
            </div>
        {:else}
            {#if isServerlessMode() && user && joinedCodelabs.length > 0}
                <section class="mb-16">
                    <div class="flex items-center gap-2 mb-8">
                        <Star class="text-[#FBBC04] fill-[#FBBC04]" size={24} />
                        <h2 class="text-2xl font-bold text-[#202124]">Joined Codelabs</h2>
                        <span class="bg-[#FBBC04]/10 text-[#FBBC04] px-3 py-1 rounded-full text-xs font-bold">
                            {joinedCodelabs.length}
                        </span>
                    </div>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                        {#each joinedCodelabs as codelab}
                            <div in:fade>
                                <a
                                    href="/codelabs/{codelab.id}"
                                    class="group block bg-[#FBBC04]/5 border border-[#FBBC04]/20 rounded-3xl p-8 hover:shadow-xl transition-all duration-300 hover:border-[#FBBC04] h-full flex flex-col"
                                >
                                    <h3 class="text-xl font-bold text-[#202124] group-hover:text-[#FBBC04] mb-3 line-clamp-2">
                                        {codelab.title}
                                    </h3>
                                    <p class="text-[#5F6368] text-sm line-clamp-2 mb-6">
                                        {codelab.description}
                                    </p>
                                    <div class="mt-auto flex items-center justify-between">
                                        <span class="text-xs font-bold text-[#5F6368]">{codelab.author}</span>
                                        <div class="text-[#FBBC04] font-bold text-sm flex items-center gap-1">
                                            Continue <ArrowRight size={14} />
                                        </div>
                                    </div>
                                </a>
                            </div>
                        {/each}
                    </div>
                </section>
                <div class="h-px bg-[#E8EAED] w-full mb-16"></div>
            {/if}

            {#if filteredCodelabs.length === 0}
            <div
                class="bg-white border border-[#E8EAED] rounded-3xl p-20 text-center shadow-sm"
                in:fade
            >
                <div
                    class="bg-[#F8F9FA] w-24 h-24 rounded-full flex items-center justify-center mx-auto mb-8"
                >
                    <Search size={40} class="text-[#BDC1C6]" />
                </div>
                <h3 class="text-2xl font-bold text-[#202124]">
                    No codelabs found
                </h3>
                <p class="text-[#5F6368] mt-4 text-lg max-w-md mx-auto">
                    We couldn't find any codelabs matching your search. Try a
                    different keyword or check back later!
                </p>
                <button
                    onclick={() => (searchQuery = "")}
                    class="mt-8 text-[#34A853] font-bold hover:underline"
                >
                    Clear Search
                </button>
            </div>
        {:else}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                {#each filteredCodelabs as codelab, i}
                    <div in:fly={{ y: 20, delay: i * 50, duration: 500 }}>
                        <a
                            href="/codelabs/{codelab.id}/entry"
                            class="group block bg-white border border-[#E8EAED] rounded-3xl p-8 hover:shadow-2xl transition-all duration-500 hover:border-[#34A853] relative overflow-hidden h-full flex flex-col"
                        >
                            <div class="flex-1">
                                <h3
                                    class="text-2xl font-bold text-[#202124] group-hover:text-[#34A853] transition-colors mb-4 line-clamp-2 leading-tight"
                                >
                                    {codelab.title}
                                </h3>
                                <p
                                    class="text-[#5F6368] text-base line-clamp-3 mb-8 leading-relaxed"
                                >
                                    {codelab.description}
                                </p>
                            </div>

                            <div
                                class="flex items-center justify-between border-t border-[#F1F3F4] pt-6 mt-auto"
                            >
                                <div
                                    class="flex items-center gap-3 text-[#5F6368] text-sm font-bold"
                                >
                                    <div
                                        class="w-8 h-8 rounded-full bg-[#E8F5E9] flex items-center justify-center text-[#34A853]"
                                    >
                                        <User size={16} />
                                    </div>
                                    {codelab.author}
                                </div>
                                <div
                                    class="flex items-center gap-2 text-[#34A853] font-bold text-sm group-hover:translate-x-1 transition-transform"
                                >
                                    Join <ArrowRight size={18} />
                                </div>
                            </div>
                        </a>
                    </div>
                {/each}
            </div>
        {/if}
    {/if}
</main>

    <footer class="py-12 text-center text-[#9AA0A6] text-sm font-medium">
        <p>&copy; 2025 JAICHANGPARK &bull; Built for Learning</p>
    </footer>
</div>
