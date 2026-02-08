<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { getCertificate, type CertificateInfo } from "$lib/api";
    import { t } from "svelte-i18n";
    import { fade } from "svelte/transition";
    import { CheckCircle2, Download, Printer, ShieldCheck, ExternalLink } from "lucide-svelte";
    // @ts-ignore
    import QRCode from "svelte-qrcode";

    let id = page.params.id as string;
    let info = $state<CertificateInfo | null>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);

    onMount(async () => {
        try {
            info = await getCertificate(id);
        } catch (e: any) {
            console.error(e);
            if (e.message && e.message.includes("REQUIREMENTS_NOT_MET")) {
                error = "REQUIREMENTS_NOT_MET";
            } else {
                error = "NOT_FOUND";
            }
        } finally {
            loading = false;
        }
    });

    function handlePrint() {
        window.print();
    }

    let fullVerificationUrl = $derived(
        info ? `${window.location.origin}${info.verification_url}` : ""
    );
</script>

<div class="min-h-screen bg-background dark:bg-dark-bg p-4 sm:p-8 flex flex-col items-center">
    {#if loading}
        <div class="flex-1 flex flex-col items-center justify-center gap-4">
            <div class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
            <p class="text-muted-foreground dark:text-dark-text-muted font-medium">{$t("common.loading")}</p>
        </div>
    {:else if error}
        <div class="flex-1 flex flex-col items-center justify-center gap-6 text-center max-w-md">
            <div class="w-20 h-20 {error === 'REQUIREMENTS_NOT_MET' ? 'bg-amber-50 dark:bg-amber-500/10 text-amber-500' : 'bg-red-50 dark:bg-red-500/10 text-red-500'} rounded-full flex items-center justify-center">
                <ShieldCheck size={48} />
            </div>
            <div>
                <h1 class="text-2xl font-bold text-foreground dark:text-dark-text mb-2">
                    {error === 'REQUIREMENTS_NOT_MET' ? $t("certificate.not_earned") : $t("certificate.not_found")}
                </h1>
                <p class="text-muted-foreground dark:text-dark-text-muted">
                    {error === 'REQUIREMENTS_NOT_MET' ? $t("certificate.requirements_guide") : $t("attendee.error_registration_failed")}
                </p>
            </div>
            <div class="flex flex-col gap-3 w-full">
                <a href="/" class="text-primary font-bold hover:underline">{$t("attendee.return_home")}</a>
                <button 
                    onclick={() => window.history.back()}
                    class="text-sm text-muted-foreground dark:text-dark-text-muted hover:underline"
                >
                    {$t("editor.back")}
                </button>
            </div>
        </div>
    {:else if info}
        <div class="w-full max-w-4xl space-y-8 no-print" in:fade>
            <div class="flex flex-col sm:flex-row justify-between items-center gap-4">
                <a href="/codelabs/{info.codelab_id}" class="flex items-center gap-2 group">
                    <div class="w-8 h-8 bg-primary rounded flex items-center justify-center text-primary-foreground font-bold group-hover:scale-110 transition-transform">OC</div>
                    <span class="font-bold text-xl dark:text-dark-text">Open-Codelabs</span>
                </a>
                <div class="flex gap-4 items-center">
                    <a href="/codelabs/{info.codelab_id}" class="text-sm font-bold text-primary hover:underline">
                        {$t("certificate.return_to_codelab")}
                    </a>
                    <div class="flex gap-3">
                        <button 
                            onclick={handlePrint}
                            class="flex items-center gap-2 bg-white dark:bg-dark-surface border border-border dark:border-dark-border px-6 py-2.5 rounded-xl font-bold text-foreground dark:text-dark-text hover:bg-accent/60 dark:hover:bg-white/5 shadow-sm transition-all"
                        >
                            <Printer size={18} />
                            {$t("certificate.download_pdf")}
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Certificate Content -->
        <div 
            class="certificate-container mt-12 w-full max-w-[210mm] bg-white text-foreground shadow-2xl relative overflow-hidden transition-colors border-[16px] border-border"
        >
            <!-- Corner Accents -->
            <div class="absolute top-0 left-0 w-32 h-32 border-t-8 border-l-8 border-primary opacity-20"></div>
            <div class="absolute top-0 right-0 w-32 h-32 border-t-8 border-r-8 border-red-500 opacity-20"></div>
            <div class="absolute bottom-0 left-0 w-32 h-32 border-b-8 border-l-8 border-amber-400 opacity-20"></div>
            <div class="absolute bottom-0 right-0 w-32 h-32 border-b-8 border-r-8 border-emerald-500 opacity-20"></div>

            <div class="h-full flex flex-col items-center justify-between p-10 sm:p-14 text-center">
                <div class="space-y-4">
                    <h1 class="text-4xl sm:text-6xl font-serif font-bold tracking-[0.2em] text-foreground">{$t("certificate.title")}</h1>
                    <p class="text-base sm:text-lg font-bold tracking-widest text-muted-foreground uppercase">{$t("certificate.subtitle")}</p>
                </div>

                <div class="w-full space-y-8">
                    <div class="space-y-2">
                        <p class="text-muted-foreground italic text-sm">This is to certify that</p>
                        <h2 class="text-3xl sm:text-5xl font-bold text-foreground border-b-2 border-border inline-block px-12 pb-2">{info.attendee_name}</h2>
                    </div>

                    <div class="space-y-4">
                        <p class="text-muted-foreground text-sm">has successfully completed the hands-on session</p>
                        <h3 class="text-xl sm:text-2xl font-bold text-primary max-w-2xl mx-auto leading-tight">{info.codelab_title}</h3>
                        <p class="text-xs text-muted-foreground">Facilitated by <span class="font-bold text-foreground">{info.author}</span></p>
                    </div>
                </div>

                <div class="w-full flex justify-between items-end mt-4">
                    <div class="text-left space-y-6">
                        <div class="space-y-1">
                            <p class="text-[10px] uppercase tracking-tighter text-muted-foreground">{$t("certificate.completion_date")}</p>
                            <p class="font-bold text-lg">{new Date(info.completed_at).toLocaleDateString()}</p>
                        </div>
                        <div class="flex items-center gap-2 text-emerald-600">
                            <CheckCircle2 size={24} />
                            <span class="font-bold tracking-tighter uppercase text-sm">{$t("certificate.valid")}</span>
                        </div>
                    </div>

                    <div class="flex flex-col items-center gap-2">
                        <div class="bg-white p-2 border border-border">
                            <QRCode value={fullVerificationUrl} size={80} />
                        </div>
                        <p class="text-[8px] text-muted-foreground uppercase tracking-tighter">Verify Authenticity</p>
                    </div>

                    <div class="text-right">
                        <div class="w-48 border-b-2 border-foreground mb-2"></div>
                        <p class="text-xs font-bold text-muted-foreground uppercase tracking-widest">Authorized Signature</p>
                        <p class="text-[10px] text-muted-foreground mt-1">Open-Codelabs Facilitator Team</p>
                    </div>
                </div>
            </div>
        </div>
        
        <p class="mt-8 text-sm text-muted-foreground dark:text-dark-text-muted text-center max-w-lg no-print">
            {$t("certificate.verify_desc")}
        </p>
    {/if}
</div>

<style>
    @media print {
        .no-print {
            display: none !important;
        }
        :global(body) {
            background-color: white !important;
        }
        .certificate-container {
            margin: 0 !important;
            box-shadow: none !important;
            border-width: 12px !important;
            width: 100% !important;
            max-width: none !important;
            position: fixed !important;
            top: 0 !important;
            left: 0 !important;
            aspect-ratio: 1.414 / 1 !important;
        }
    }
    
    .certificate-container {
        font-family: 'Times New Roman', Times, serif;
        aspect-ratio: 1.5 / 1;
        min-height: 760px;
    }

    @media (min-width: 640px) {
        .certificate-container {
            min-height: 820px;
        }
    }
    
    h2, h3, p, span {
        font-family: 'Inter', system-ui, -apple-system, sans-serif;
    }
</style>
