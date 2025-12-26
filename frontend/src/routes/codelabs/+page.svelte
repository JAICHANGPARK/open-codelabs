<script lang="ts">
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { listCodelabs, type Codelab } from "$lib/api";
    import {
        BookOpen,
        User,
        Clock,
        Search,
        ArrowRight,
        Loader2,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";

    let codelabs: Codelab[] = $state([]);
    let loading = $state(true);
    let searchQuery = $state("");

    onMount(async () => {
        try {
            codelabs = await listCodelabs();
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    let filteredCodelabs = $derived(
        codelabs.filter(
            (c) =>
                c.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                c.description.toLowerCase().includes(searchQuery.toLowerCase()),
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
        {:else if filteredCodelabs.length === 0}
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
    </main>

    <footer class="py-12 text-center text-[#9AA0A6] text-sm font-medium">
        <p>&copy; 2025 JAICHANGPARK &bull; Built for Learning</p>
    </footer>
</div>
