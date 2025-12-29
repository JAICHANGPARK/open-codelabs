<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { getCertificate, type CertificateInfo } from "$lib/api";
    import { t } from "svelte-i18n";
    import { fade, scale } from "svelte/transition";
    import { ShieldCheck, XCircle, CheckCircle2, User, Calendar, Bookmark, FileText } from "lucide-svelte";

    let id = page.params.id as string;
    let info = $state<CertificateInfo | null>(null);
    let loading = $state(true);
    let error = $state(false);

    onMount(async () => {
        try {
            info = await getCertificate(id);
        } catch (e) {
            console.error(e);
            error = true;
        } finally {
            loading = false;
        }
    });
</script>

<div class="min-h-screen bg-[#F8F9FA] dark:bg-dark-bg flex flex-col items-center justify-center p-4">
    <div class="w-full max-w-md bg-white dark:bg-dark-surface rounded-3xl shadow-xl border border-[#E8EAED] dark:border-dark-border overflow-hidden">
        <div class="p-8 text-center space-y-6">
            <a href="/" class="flex items-center justify-center gap-2 mb-8 group">
                <div class="w-8 h-8 bg-[#4285F4] rounded flex items-center justify-center text-white font-bold group-hover:scale-110 transition-transform">OC</div>
                <span class="font-bold text-xl dark:text-dark-text">Open-Codelabs</span>
            </a>

            {#if loading}
                <div class="py-12 flex flex-col items-center gap-4">
                    <div class="w-12 h-12 border-4 border-[#4285F4] border-t-transparent rounded-full animate-spin"></div>
                    <p class="text-[#5F6368] dark:text-dark-text-muted font-medium">{$t("common.loading")}</p>
                </div>
            {:else if error || !info}
                <div in:scale={{ duration: 300 }} class="py-8 space-y-6">
                    <div class="w-20 h-20 bg-red-50 dark:bg-red-500/10 rounded-full flex items-center justify-center text-[#EA4335] mx-auto">
                        <XCircle size={48} />
                    </div>
                    <div>
                        <h2 class="text-2xl font-bold text-[#202124] dark:text-dark-text">{$t("certificate.not_found")}</h2>
                        <p class="text-[#5F6368] dark:text-dark-text-muted mt-2">This certificate ID is invalid or has not been issued yet.</p>
                    </div>
                </div>
            {:else}
                <div in:scale={{ duration: 300 }} class="space-y-8">
                    <div class="w-20 h-20 bg-[#E6F4EA] dark:bg-green-500/10 rounded-full flex items-center justify-center text-[#34A853] mx-auto">
                        <ShieldCheck size={48} />
                    </div>
                    
                    <div class="space-y-2">
                        <h2 class="text-2xl font-bold text-[#202124] dark:text-dark-text">{$t("certificate.valid")}</h2>
                        <p class="text-xs text-[#5F6368] dark:text-dark-text-muted uppercase tracking-widest font-bold">{$t("certificate.verify_status")}</p>
                    </div>

                    <div class="bg-[#F8F9FA] dark:bg-white/5 rounded-2xl p-6 text-left space-y-4 border border-[#E8EAED] dark:border-dark-border">
                        <div class="flex items-start gap-3">
                            <User size={18} class="text-[#4285F4] mt-1 shrink-0" />
                            <div>
                                <p class="text-[10px] text-[#5F6368] dark:text-dark-text-muted uppercase font-bold">{$t("certificate.issued_to")}</p>
                                <p class="font-bold text-[#202124] dark:text-dark-text">{info.attendee_name}</p>
                            </div>
                        </div>
                        <div class="flex items-start gap-3">
                            <Bookmark size={18} class="text-[#EA4335] mt-1 shrink-0" />
                            <div>
                                <p class="text-[10px] text-[#5F6368] dark:text-dark-text-muted uppercase font-bold">{$t("certificate.codelab_title")}</p>
                                <p class="font-bold text-[#202124] dark:text-dark-text">{info.codelab_title}</p>
                            </div>
                        </div>
                        <div class="flex items-start gap-3">
                            <Calendar size={18} class="text-[#FBBC04] mt-1 shrink-0" />
                            <div>
                                <p class="text-[10px] text-[#5F6368] dark:text-dark-text-muted uppercase font-bold">{$t("certificate.verified_at")}</p>
                                <p class="font-bold text-[#202124] dark:text-dark-text">{new Date(info.completed_at).toLocaleString()}</p>
                            </div>
                        </div>
                    </div>

                    <a 
                        href="/certificate/{id}" 
                        class="flex items-center justify-center gap-2 w-full bg-[#4285F4] text-white py-4 rounded-xl font-bold hover:bg-[#1A73E8] transition-all shadow-md active:scale-95"
                    >
                        <FileText size={20} />
                        View Full Certificate
                    </a>
                </div>
            {/if}
        </div>
        <div class="bg-[#F8F9FA] dark:bg-white/5 p-4 text-center border-t border-[#E8EAED] dark:border-dark-border">
            <p class="text-[10px] text-[#9AA0A6] uppercase tracking-widest">Digital Verification Service by Open-Codelabs</p>
        </div>
    </div>
    
    <p class="mt-8 text-sm text-[#5F6368] dark:text-dark-text-muted">
        &copy; {new Date().getFullYear()} Open-Codelabs Project
    </p>
</div>
