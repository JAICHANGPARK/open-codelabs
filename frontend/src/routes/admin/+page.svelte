<script lang="ts">
    import { onMount } from "svelte";
    import { fade, slide, fly } from "svelte/transition";
    import {
        listCodelabs,
        createCodelab,
        importCodelab,
        deleteCodelab,
        type Codelab,
    } from "$lib/api";
    import {
        Plus,
        BookOpen,
        User,
        Clock,
        LayoutDashboard,
        Download,
        FileUp,
        Trash2,
        Share2,
        Check,
        Settings,
        Sparkles,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";
    import { encrypt, decrypt } from "$lib/crypto";
    import AiCodelabGenerator from "$lib/components/AiCodelabGenerator.svelte";

    let codelabs: Codelab[] = $state([]);
    let loading = $state(true);
    let showCreateModal = $state(false);
    let newCodelab = $state({ title: "", description: "", author: "" });
    let copyTarget: string | null = $state(null);

    // AI & Settings State
    let showSettingsModal = $state(false);
    let showAiGenerator = $state(false);
    let geminiApiKey = $state("");
    let apiKeySaved = $state(false);

    onMount(async () => {
        try {
            codelabs = await listCodelabs();

            // Load API Key
            const encryptedKey = localStorage.getItem("gemini_api_key");
            if (encryptedKey) {
                const decrypted = decrypt(encryptedKey);
                if (decrypted) {
                    geminiApiKey = decrypted;
                    apiKeySaved = true;
                }
            }
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    function saveSettings() {
        if (geminiApiKey.trim()) {
            const encrypted = encrypt(geminiApiKey.trim());
            localStorage.setItem("gemini_api_key", encrypted);
            apiKeySaved = true;
            showSettingsModal = false;
        } else {
            localStorage.removeItem("gemini_api_key");
            apiKeySaved = false;
        }
    }

    async function handleCreate() {
        if (!newCodelab.title) return;
        try {
            const created = await createCodelab(newCodelab);
            codelabs = [created, ...codelabs];
            showCreateModal = false;
            newCodelab = { title: "", description: "", author: "" };
        } catch (e) {
            console.error(e);
        }
    }

    let fileInput: HTMLInputElement;

    async function handleImport(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            loading = true;
            try {
                const imported = await importCodelab(target.files[0]);
                codelabs = [imported, ...codelabs];
            } catch (e) {
                alert("Import failed: " + e);
            } finally {
                loading = false;
                target.value = "";
            }
        }
    }

    async function handleDelete(id: string) {
        if (
            !confirm(
                $t("dashboard.confirm_delete") ||
                    "Are you sure you want to delete this codelab?",
            )
        )
            return;
        try {
            await deleteCodelab(id);
            codelabs = codelabs.filter((c) => c.id !== id);
        } catch (e) {
            console.error(e);
            alert("Delete failed: " + e);
        }
    }

    async function copyLink(id: string) {
        const url = `${window.location.origin}/codelabs/${id}`;
        try {
            // Check if clipboard API is available
            if (navigator.clipboard && navigator.clipboard.writeText) {
                await navigator.clipboard.writeText(url);
            } else {
                // Fallback for browsers without clipboard API
                const input = document.createElement("input");
                input.value = url;
                input.style.position = "fixed";
                input.style.opacity = "0";
                document.body.appendChild(input);
                input.select();
                document.execCommand("copy");
                document.body.removeChild(input);
            }

            copyTarget = id;
            setTimeout(() => {
                if (copyTarget === id) copyTarget = null;
            }, 2000);
        } catch (err) {
            console.error("Failed to copy!", err);
            alert("Failed to copy link. Please try again.");
        }
    }
</script>

<div class="min-h-screen bg-[#F8F9FA]">
    <div class="max-w-6xl mx-auto p-8 lg:p-12">
        <header
            class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-6 mb-12"
        >
            <div>
                <div class="flex items-center gap-3 mb-2">
                    <div
                        class="w-10 h-10 bg-[#4285F4] rounded-lg flex items-center justify-center text-white shadow-sm"
                    >
                        <LayoutDashboard size={24} />
                    </div>
                    <h1 class="text-3xl font-bold text-[#202124]">
                        {$t("dashboard.title")}
                    </h1>
                </div>
                <p class="text-[#5F6368] text-lg">
                    {$t("dashboard.subtitle")}
                </p>
            </div>
            <div class="flex items-center gap-4">
                <button
                    onclick={() => (showSettingsModal = true)}
                    class="p-2.5 hover:bg-[#F8F9FA] rounded-full text-[#5F6368] transition-all border border-transparent hover:border-[#E8EAED]"
                    title="Settings"
                >
                    <Settings
                        size={20}
                        class={apiKeySaved ? "text-[#34A853]" : ""}
                    />
                </button>
                <div class="h-6 w-px bg-[#E8EAED]"></div>

                <input
                    type="file"
                    accept=".zip"
                    bind:this={fileInput}
                    onchange={handleImport}
                    class="hidden"
                />
                <button
                    onclick={() => fileInput.click()}
                    class="bg-white hover:bg-[#F8F9FA] text-[#5F6368] px-4 py-2.5 rounded-full flex items-center gap-2 transition-all border border-[#DADCE0] font-bold text-sm"
                >
                    <FileUp size={18} />
                    {$t("common.import")}
                </button>
                <button
                    onclick={() => (showAiGenerator = true)}
                    class="bg-white hover:bg-[#F8F9FA] text-[#4285F4] px-4 py-2.5 rounded-full flex items-center gap-2 transition-all border border-[#DADCE0] font-bold text-sm"
                >
                    <Sparkles size={18} />
                    Create with AI
                </button>
                <button
                    onclick={() => (showCreateModal = true)}
                    class="bg-[#4285F4] hover:bg-[#1A73E8] text-white px-5 py-2.5 rounded-full flex items-center gap-2 transition-all shadow-md hover:shadow-lg font-bold text-sm"
                >
                    <Plus size={20} />
                    {$t("dashboard.new_codelab")}
                </button>
            </div>
        </header>

        {#if loading}
            <div class="flex justify-center items-center py-20" in:fade>
                <div
                    class="animate-spin rounded-full h-12 w-12 border-4 border-[#E8EAED] border-t-[#4285F4]"
                ></div>
            </div>
        {:else if codelabs.length === 0}
            <div
                class="bg-white border border-[#E8EAED] rounded-2xl p-16 text-center shadow-sm"
                in:fly={{ y: 20, duration: 500 }}
            >
                <div
                    class="bg-[#F8F9FA] w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-6"
                >
                    <BookOpen size={40} class="text-[#BDC1C6]" />
                </div>
                <h3 class="text-xl font-bold text-[#202124]">
                    {$t("dashboard.no_codelabs")}
                </h3>
                <p class="text-[#5F6368] mt-2 text-lg">
                    {$t("dashboard.get_started")}
                </p>
                <button
                    onclick={() => (showCreateModal = true)}
                    class="mt-8 text-[#4285F4] font-bold hover:text-[#1A73E8] flex items-center gap-2 mx-auto px-6 py-2 rounded-full border border-[#DADCE0] hover:bg-[#E8F0FE] transition-all"
                >
                    {$t("dashboard.create_first")}
                    <Plus size={18} />
                </button>
            </div>
        {:else}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                {#each codelabs as codelab, i}
                    <div in:fly={{ y: 20, delay: i * 100, duration: 500 }}>
                        <a
                            href="/admin/{codelab.id}"
                            class="group block bg-white border border-[#E8EAED] rounded-2xl p-8 hover:shadow-xl transition-all duration-300 hover:border-[#4285F4] relative overflow-hidden"
                        >
                            <div
                                class="absolute top-4 right-4 flex items-center gap-2 z-20"
                            >
                                <button
                                    onclick={(e) => {
                                        e.preventDefault();
                                        copyLink(codelab.id);
                                    }}
                                    class="p-2 transition-all rounded-full {copyTarget ===
                                    codelab.id
                                        ? 'bg-[#E6F4EA] text-[#1E8E3E]'
                                        : 'bg-[#F8F9FA] text-[#5F6368] hover:bg-[#E8F0FE] hover:text-[#4285F4]'}"
                                    title="Share Participant Link"
                                >
                                    {#if copyTarget === codelab.id}
                                        <Check size={18} />
                                    {:else}
                                        <Share2 size={18} />
                                    {/if}
                                </button>
                                <button
                                    onclick={(e) => {
                                        e.preventDefault();
                                        handleDelete(codelab.id);
                                    }}
                                    class="p-2 bg-[#F8F9FA] text-[#5F6368] hover:text-[#EA4335] hover:bg-[#FEECEB] rounded-full transition-all"
                                    title={$t("common.delete") || "Delete"}
                                >
                                    <Trash2 size={18} />
                                </button>
                            </div>

                            <h3
                                class="text-xl font-bold text-[#202124] group-hover:text-[#4285F4] transition-colors mb-3 line-clamp-1"
                            >
                                {codelab.title}
                            </h3>
                            <p
                                class="text-[#5F6368] text-base line-clamp-2 mb-8 h-12"
                            >
                                {codelab.description}
                            </p>
                            <div
                                class="flex items-center justify-between border-t border-[#F1F3F4] pt-6"
                            >
                                <div
                                    class="flex items-center gap-2 text-[#5F6368] text-sm font-medium"
                                >
                                    <div
                                        class="w-6 h-6 rounded-full bg-[#F1F3F4] flex items-center justify-center"
                                    >
                                        <User size={14} />
                                    </div>
                                    {codelab.author}
                                </div>
                                <div
                                    class="flex items-center gap-1.5 text-[#9AA0A6] text-xs font-medium uppercase tracking-wider"
                                >
                                    <Clock size={14} />
                                    {new Date(
                                        codelab.created_at || "",
                                    ).toLocaleDateString($locale || "en")}
                                </div>
                            </div>
                        </a>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>

{#if showCreateModal}
    <div
        class="fixed inset-0 bg-[#202124]/60 backdrop-blur-sm flex items-center justify-center p-4 z-50"
        transition:fade={{ duration: 200 }}
    >
        <div
            class="bg-white rounded-3xl shadow-2xl w-full max-w-lg overflow-hidden"
            in:fly={{ y: 40, duration: 400 }}
        >
            <div class="bg-[#4285F4] p-8 text-white">
                <h2 class="text-2xl font-bold mb-2">
                    {$t("dashboard.create_new_title")}
                </h2>
                <p class="opacity-80">{$t("dashboard.design_experience")}</p>
            </div>

            <div class="p-8 space-y-6">
                <div>
                    <label
                        for="new-title"
                        class="block text-sm font-bold text-[#5F6368] mb-2 uppercase tracking-wide"
                        >{$t("dashboard.codelab_title")}</label
                    >
                    <input
                        id="new-title"
                        type="text"
                        bind:value={newCodelab.title}
                        placeholder={$t("dashboard.placeholder_title")}
                        class="w-full border-2 border-[#F1F3F4] rounded-xl px-4 py-3 focus:border-[#4285F4] outline-none transition-all placeholder-[#BDC1C6]"
                    />
                </div>
                <div>
                    <label
                        for="new-desc"
                        class="block text-sm font-bold text-[#5F6368] mb-2 uppercase tracking-wide"
                        >{$t("dashboard.codelab_desc")}</label
                    >
                    <textarea
                        id="new-desc"
                        bind:value={newCodelab.description}
                        placeholder={$t("dashboard.placeholder_desc")}
                        class="w-full border-2 border-[#F1F3F4] rounded-xl px-4 py-3 focus:border-[#4285F4] outline-none h-32 resize-none transition-all placeholder-[#BDC1C6]"
                    ></textarea>
                </div>
                <div>
                    <label
                        for="new-author"
                        class="block text-sm font-bold text-[#5F6368] mb-2 uppercase tracking-wide"
                        >{$t("dashboard.codelab_author")}</label
                    >
                    <input
                        id="new-author"
                        type="text"
                        bind:value={newCodelab.author}
                        placeholder={$t("dashboard.placeholder_author")}
                        class="w-full border-2 border-[#F1F3F4] rounded-xl px-4 py-3 focus:border-[#4285F4] outline-none transition-all placeholder-[#BDC1C6]"
                    />
                </div>
            </div>

            <div class="px-8 pb-8 flex justify-end gap-4">
                <button
                    onclick={() => (showCreateModal = false)}
                    class="px-6 py-2.5 text-[#5F6368] font-bold hover:bg-[#F8F9FA] rounded-full transition-all"
                >
                    {$t("common.cancel")}
                </button>
                <button
                    onclick={handleCreate}
                    class="px-8 py-2.5 bg-[#4285F4] text-white rounded-full font-bold hover:bg-[#1A73E8] shadow-md transition-all active:scale-95"
                >
                    {$t("common.create")}
                </button>
            </div>
        </div>
    </div>
{/if}

{#if showSettingsModal}
    <div
        class="fixed inset-0 bg-[#202124]/60 backdrop-blur-sm flex items-center justify-center p-4 z-50"
        transition:fade={{ duration: 200 }}
    >
        <div
            class="bg-white rounded-2xl shadow-2xl w-full max-w-md overflow-hidden"
            in:fly={{ y: 20, duration: 300 }}
        >
            <div
                class="px-6 py-4 border-b border-[#F1F3F4] flex items-center justify-between bg-[#F8F9FA]"
            >
                <h3 class="font-bold text-[#202124]">Settings</h3>
                <button
                    onclick={() => (showSettingsModal = false)}
                    class="text-[#5F6368] hover:bg-[#E8EAED] p-1 rounded-full"
                    ><Settings size={18} /></button
                >
            </div>
            <div class="p-6 space-y-4">
                <div>
                    <label
                        for="api-key"
                        class="block text-sm font-bold text-[#5F6368] mb-2"
                        >Gemini API Key</label
                    >
                    <input
                        id="api-key"
                        type="password"
                        bind:value={geminiApiKey}
                        placeholder="Enter your Gemini API Key"
                        class="w-full border-2 border-[#F1F3F4] rounded-lg px-4 py-2.5 focus:border-[#4285F4] outline-none transition-all placeholder-[#BDC1C6] text-sm font-mono"
                    />
                    <p class="text-xs text-[#9AA0A6] mt-2">
                        Your key is stored locally in your browser and
                        encrypted. It is never sent to our servers.
                    </p>
                </div>
            </div>
            <div
                class="px-6 py-4 border-t border-[#F1F3F4] flex justify-end gap-3 bg-[#F8F9FA]"
            >
                <button
                    onclick={() => (showSettingsModal = false)}
                    class="px-4 py-2 text-[#5F6368] font-bold hover:bg-[#E8EAED] rounded-lg text-sm transition-all"
                >
                    Cancel
                </button>
                <button
                    onclick={saveSettings}
                    class="px-6 py-2 bg-[#4285F4] text-white rounded-lg font-bold hover:bg-[#1A73E8] text-sm transition-all shadow-sm"
                >
                    Save Settings
                </button>
            </div>
        </div>
    </div>
{/if}

{#if showAiGenerator}
    <AiCodelabGenerator
        apiKey={geminiApiKey}
        onClose={() => (showAiGenerator = false)}
        onCodelabCreated={(codelab) => {
            codelabs = [codelab, ...codelabs];
            showAiGenerator = false;
        }}
    />
{/if}
