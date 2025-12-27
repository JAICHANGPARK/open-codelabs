<script lang="ts">
    import { login } from "$lib/api";
    import { goto } from "$app/navigation";
    import { Lock, User, LogIn, AlertCircle } from "lucide-svelte";
    import { fade, fly } from "svelte/transition";
    import { t } from "svelte-i18n";

    let admin_id = $state("");
    let admin_pw = $state("");
    let error = $state("");
    let loading = $state(false);

    async function handleLogin() {
        if (!admin_id || !admin_pw) {
            error = $t("login.error_fields");
            return;
        }
        loading = true;
        error = "";
        try {
            const { token } = await login(admin_id, admin_pw);
            localStorage.setItem("adminToken", token);
            goto("/admin");
        } catch (e) {
            error = $t("login.error_credentials");
        } finally {
            loading = false;
        }
    }
</script>

<div class="min-h-screen bg-[#F8F9FA] flex items-center justify-center p-6">
    <div class="w-full max-w-md" in:fly={{ y: 20, duration: 600 }}>
        <div
            class="bg-white rounded-[2rem] shadow-2xl overflow-hidden border border-[#E8EAED]"
        >
            <div class="bg-[#4285F4] p-10 text-white text-center">
                <div
                    class="w-16 h-16 bg-white/20 rounded-2xl flex items-center justify-center mx-auto mb-6 backdrop-blur-sm"
                >
                    <LogIn size={32} />
                </div>
                <h1 class="text-3xl font-bold mb-2">{$t("login.title")}</h1>
                <p class="text-white/80 font-medium">{$t("login.subtitle")}</p>
            </div>

            <div class="p-10 space-y-8">
                {#if error}
                    <div
                        class="bg-red-50 text-red-600 p-4 rounded-xl flex items-center gap-3 border border-red-100"
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
                            class="block text-xs font-bold text-[#5F6368] uppercase tracking-widest mb-2 px-1"
                            >{$t("login.admin_id")}</label
                        >
                        <div class="relative group">
                            <div
                                class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-[#BDC1C6] group-focus-within:text-[#4285F4] transition-colors"
                            >
                                <User size={20} />
                            </div>
                            <input
                                id="admin_id"
                                type="text"
                                bind:value={admin_id}
                                placeholder={$t("login.placeholder_id")}
                                class="w-full bg-[#FAFBFF] border-2 border-[#F1F3F4] rounded-2xl pl-12 pr-4 py-4 focus:border-[#4285F4] outline-none transition-all placeholder-[#BDC1C6] font-medium"
                                onkeydown={(e) =>
                                    e.key === "Enter" && handleLogin()}
                            />
                        </div>
                    </div>

                    <div>
                        <label
                            for="admin_pw"
                            class="block text-xs font-bold text-[#5F6368] uppercase tracking-widest mb-2 px-1"
                            >{$t("login.password")}</label
                        >
                        <div class="relative group">
                            <div
                                class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-[#BDC1C6] group-focus-within:text-[#4285F4] transition-colors"
                            >
                                <Lock size={20} />
                            </div>
                            <input
                                id="admin_pw"
                                type="password"
                                bind:value={admin_pw}
                                placeholder="••••••••"
                                class="w-full bg-[#FAFBFF] border-2 border-[#F1F3F4] rounded-2xl pl-12 pr-4 py-4 focus:border-[#4285F4] outline-none transition-all placeholder-[#BDC1C6] font-medium"
                                onkeydown={(e) =>
                                    e.key === "Enter" && handleLogin()}
                            />
                        </div>
                    </div>
                </div>

                <button
                    onclick={handleLogin}
                    disabled={loading}
                    class="w-full bg-[#4285F4] hover:bg-[#1A73E8] text-white font-bold py-5 rounded-2xl shadow-lg hover:shadow-xl transition-all active:scale-[0.98] disabled:opacity-50 flex items-center justify-center gap-3 text-lg"
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

                <div class="pt-4 text-center">
                    <button
                        class="text-sm font-bold text-[#5F6368] hover:text-[#4285F4] transition-colors"
                    >
                        {$t("login.trouble")}
                    </button>
                </div>
            </div>
        </div>

        <p class="mt-8 text-center text-[#9AA0A6] text-sm font-medium">
            {$t("common.title")} &copy; 2025
        </p>
    </div>
</div>
