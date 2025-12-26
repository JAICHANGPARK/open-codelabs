<script lang="ts">
    import { onMount } from "svelte";
    import { fly, fade } from "svelte/transition";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { getCodelab, registerAttendee, type Codelab } from "$lib/api";
    import { User, KeyRound, ArrowRight, Loader2 } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let id = page.params.id as string;
    let codelab = $state<Codelab | null>(null);
    let name = $state("");
    let code = $state("");
    let loading = $state(true);
    let submitting = $state(false);
    let error = $state("");

    onMount(async () => {
        try {
            const data = await getCodelab(id);
            codelab = data[0];

            // Check if already registered in this session
            const savedAttendee = localStorage.getItem(`attendee_${id}`);
            if (savedAttendee) {
                goto(`/codelabs/${id}`);
            }
        } catch (e) {
            error = "Codelab not found";
        } finally {
            loading = false;
        }
    });

    async function handleSubmit() {
        if (!name || !code) {
            error = "Please fill in all fields";
            return;
        }

        submitting = true;
        error = "";
        try {
            const attendee = await registerAttendee(id, name, code);
            localStorage.setItem(`attendee_${id}`, JSON.stringify(attendee));
            goto(`/codelabs/${id}`);
        } catch (e: any) {
            if (e.message === "DUPLICATE_NAME") {
                error =
                    "This nickname is already taken in this codelab. Please try another one.";
            } else {
                error =
                    "Registration failed. Please check your connection and try again.";
            }
        } finally {
            submitting = false;
        }
    }
</script>

<div
    class="min-h-screen bg-[#F8F9FA] flex flex-col items-center justify-center p-6 font-sans"
>
    {#if loading}
        <div in:fade class="flex flex-col items-center gap-4">
            <Loader2 class="w-10 h-10 text-[#4285F4] animate-spin" />
            <p class="text-[#5F6368] font-medium">{$t("common.loading")}</p>
        </div>
    {:else if codelab}
        <div in:fly={{ y: 20, duration: 600 }} class="w-full max-w-md">
            <div
                class="bg-white rounded-3xl shadow-xl shadow-[#4285F4]/5 border border-[#DADCE0] overflow-hidden"
            >
                <div class="p-8 pb-4 text-center">
                    <div
                        class="w-16 h-16 bg-[#4285F4]/10 rounded-2xl flex items-center justify-center mx-auto mb-6"
                    >
                        <User class="w-8 h-8 text-[#4285F4]" />
                    </div>
                    <h1 class="text-2xl font-bold text-[#202124] mb-2">
                        Join Codelab
                    </h1>
                    <p class="text-[#5F6368] text-sm">
                        Enter your details to participate in <br />
                        <span class="font-bold text-[#202124]"
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
                            class="p-3 bg-red-50 text-red-600 text-sm rounded-xl border border-red-100 font-medium"
                        >
                            {error}
                        </div>
                    {/if}

                    <div class="space-y-2">
                        <label
                            for="name"
                            class="text-xs font-bold text-[#5F6368] uppercase tracking-wider ml-1"
                        >
                            Your Name / Nickname
                        </label>
                        <div class="relative group">
                            <div
                                class="absolute left-4 top-1/2 -translate-y-1/2 text-[#9AA0A6] group-focus-within:text-[#4285F4] transition-colors"
                            >
                                <User size={18} />
                            </div>
                            <input
                                id="name"
                                type="text"
                                bind:value={name}
                                placeholder="e.g. Alex"
                                class="w-full pl-12 pr-4 py-3.5 bg-[#F8F9FA] border border-[#DADCE0] rounded-xl outline-none focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 transition-all text-[#202124]"
                                required
                            />
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label
                            for="code"
                            class="text-xs font-bold text-[#5F6368] uppercase tracking-wider ml-1"
                        >
                            Unique Code (Phone suffix, etc.)
                        </label>
                        <div class="relative group">
                            <div
                                class="absolute left-4 top-1/2 -translate-y-1/2 text-[#9AA0A6] group-focus-within:text-[#4285F4] transition-colors"
                            >
                                <KeyRound size={18} />
                            </div>
                            <input
                                id="code"
                                type="text"
                                bind:value={code}
                                placeholder="e.g. 1234"
                                class="w-full pl-12 pr-4 py-3.5 bg-[#F8F9FA] border border-[#DADCE0] rounded-xl outline-none focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 transition-all text-[#202124]"
                                required
                            />
                        </div>
                    </div>

                    <button
                        type="submit"
                        disabled={submitting}
                        class="w-full bg-[#4285F4] hover:bg-[#1A73E8] disabled:bg-[#DADCE0] text-white py-4 rounded-xl font-bold shadow-lg shadow-[#4285F4]/20 hover:shadow-[#4285F4]/30 transition-all flex items-center justify-center gap-2 group active:scale-[0.98]"
                    >
                        {#if submitting}
                            <Loader2 class="w-5 h-5 animate-spin" />
                            Processing...
                        {:else}
                            Start Learning
                            <ArrowRight
                                class="w-5 h-5 group-hover:translate-x-1 transition-transform"
                            />
                        {/if}
                    </button>
                </form>
            </div>

            <p class="mt-8 text-center text-xs text-[#9AA0A6] font-medium">
                Powered by AntiGravity Dojo
            </p>
        </div>
    {:else}
        <div in:fade class="text-center">
            <h1 class="text-2xl font-bold text-[#202124] mb-4">{error}</h1>
            <a href="/" class="text-[#4285F4] font-bold hover:underline"
                >Return to Home</a
            >
        </div>
    {/if}
</div>
