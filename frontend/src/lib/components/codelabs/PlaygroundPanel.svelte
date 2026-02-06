<script lang="ts">
    import { browser } from "$app/environment";
    import { t } from "svelte-i18n";
    import {
        Check,
        ChevronDown,
        ChevronUp,
        Code,
        Copy,
        ExternalLink,
    } from "lucide-svelte";
    import type { PlaygroundBlock, PlaygroundLanguage } from "$lib/playground";

    const PLAYGROUND_CONFIG: Record<
        PlaygroundLanguage,
        { label: string; embedUrl: string; openUrl: string; embeddable: boolean }
    > = {
        dart: {
            label: "DartPad",
            embedUrl: "https://dartpad.dev/embed-dart.html",
            openUrl: "https://dartpad.dev/",
            embeddable: true,
        },
        go: {
            label: "Go Playground",
            embedUrl: "https://play.golang.org/",
            openUrl: "https://play.golang.org/",
            embeddable: false,
        },
        python: {
            label: "Pyodide",
            embedUrl: "https://pyodide.org/en/stable/console.html",
            openUrl: "https://pyodide.org/en/stable/console.html",
            embeddable: true,
        },
        jupyter: {
            label: "JupyterLite",
            embedUrl: "https://jupyterlite.github.io/demo/lab/index.html",
            openUrl: "https://jupyterlite.github.io/demo/lab/index.html",
            embeddable: true,
        },
    };

    let { playgrounds } = $props<{
        playgrounds: PlaygroundBlock[];
    }>();

    let isOpen = $state(true);
    let copied = $state(false);
    let activeLanguage = $state<PlaygroundLanguage>("python");

    let availableLanguages = $derived(playgrounds.map((block) => block.language));
    let activeBlock = $derived(
        playgrounds.find((block) => block.language === activeLanguage) ||
            playgrounds[0],
    );

    $effect(() => {
        if (!playgrounds.length) return;
        if (!availableLanguages.includes(activeLanguage)) {
            activeLanguage = playgrounds[0].language;
        }
    });

    async function handleCopyCode() {
        const code = activeBlock?.code ?? "";
        if (!browser || !code) return;

        try {
            if (navigator.clipboard && navigator.clipboard.writeText) {
                await navigator.clipboard.writeText(code);
                copied = true;
            } else {
                throw new Error("clipboard API unavailable");
            }
        } catch (e) {
            try {
                const textArea = document.createElement("textarea");
                textArea.value = code;
                textArea.style.position = "fixed";
                textArea.style.left = "-9999px";
                textArea.style.top = "0";
                document.body.appendChild(textArea);
                textArea.focus();
                textArea.select();
                const successful = document.execCommand("copy");
                document.body.removeChild(textArea);
                if (successful) {
                    copied = true;
                }
            } catch (err) {
                console.error("Fallback copy failed", err);
            }
        }

        if (copied) {
            setTimeout(() => (copied = false), 2000);
        }
    }
</script>

<div class="mt-10">
    <section
        class="rounded-3xl border border-border dark:border-dark-border bg-accent/60 dark:bg-dark-surface shadow-sm overflow-hidden"
    >
        <header
            class="flex items-center justify-between gap-4 px-6 py-4 border-b border-border dark:border-dark-border"
        >
            <div class="flex items-center gap-3">
                <div
                    class="w-10 h-10 rounded-2xl bg-primary/10 text-primary flex items-center justify-center"
                >
                    <Code size={18} />
                </div>
                <div>
                    <p
                        class="text-[11px] font-bold uppercase tracking-widest text-muted-foreground dark:text-dark-text-muted"
                    >
                        {$t("playground.title")}
                    </p>
                    <p class="text-xs text-muted-foreground dark:text-dark-text-muted">
                        {$t("playground.subtitle")}
                    </p>
                </div>
            </div>
            <button
                type="button"
                onclick={() => (isOpen = !isOpen)}
                class="p-2 rounded-full hover:bg-white/80 dark:hover:bg-white/10 transition-colors text-muted-foreground dark:text-dark-text-muted"
                aria-label={isOpen ? $t("playground.toggle_close") : $t("playground.toggle_open")}
            >
                {#if isOpen}
                    <ChevronUp size={18} />
                {:else}
                    <ChevronDown size={18} />
                {/if}
            </button>
        </header>

        {#if isOpen}
            <div class="px-6 pb-6 pt-4 space-y-4">
                <div class="flex flex-wrap gap-2">
                    {#each availableLanguages as lang}
                        <button
                            type="button"
                            onclick={() => (activeLanguage = lang)}
                            class="px-4 py-2 rounded-full text-xs font-bold transition-all border {activeLanguage === lang
                                ? 'bg-primary border-primary text-primary-foreground shadow-sm'
                                : 'bg-white dark:bg-dark-bg border-border dark:border-dark-border text-muted-foreground dark:text-dark-text-muted hover:border-primary hover:text-primary'}"
                        >
                            {PLAYGROUND_CONFIG[lang].label}
                        </button>
                    {/each}
                </div>

                {#if activeBlock}
                    <div
                        class="rounded-2xl border border-border dark:border-dark-border overflow-hidden bg-white dark:bg-dark-bg"
                    >
                        <div
                            class="flex flex-wrap items-center justify-between gap-3 px-4 py-3 bg-accent/60 dark:bg-dark-surface border-b border-border dark:border-dark-border"
                        >
                            <span
                                class="text-[11px] font-bold uppercase tracking-widest text-muted-foreground dark:text-dark-text-muted"
                            >
                                {$t("playground.code_label")}
                            </span>
                            <div class="flex items-center gap-2">
                                <button
                                    type="button"
                                    onclick={handleCopyCode}
                                    disabled={!activeBlock.code}
                                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[11px] font-bold border transition-all {activeBlock.code
                                        ? 'bg-white dark:bg-dark-bg border-border dark:border-dark-border text-foreground dark:text-dark-text hover:border-primary hover:text-primary'
                                        : 'bg-white/60 border-border text-muted-foreground cursor-not-allowed'}"
                                >
                                    {#if copied}
                                        <Check size={14} />
                                    {:else}
                                        <Copy size={14} />
                                    {/if}
                                    <span>{copied ? $t("playground.copied") : $t("common.copy")}</span>
                                </button>
                                <a
                                    href={PLAYGROUND_CONFIG[activeLanguage].openUrl}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[11px] font-bold border bg-white dark:bg-dark-bg border-border dark:border-dark-border text-foreground dark:text-dark-text hover:border-primary hover:text-primary transition-all"
                                >
                                    <ExternalLink size={14} />
                                    <span>{$t("common.open_new_tab")}</span>
                                </a>
                            </div>
                        </div>
                        {#if activeBlock.code}
                            <textarea
                                readonly
                                value={activeBlock.code}
                                rows={8}
                                class="w-full resize-y bg-white dark:bg-dark-bg text-xs font-mono text-foreground dark:text-dark-text p-4 outline-none"
                            ></textarea>
                        {:else}
                            <div
                                class="p-4 text-xs text-muted-foreground dark:text-dark-text-muted"
                            >
                                {$t("playground.no_code")}
                            </div>
                        {/if}
                    </div>

                    <div
                        class="rounded-2xl border border-border dark:border-dark-border overflow-hidden bg-white dark:bg-dark-bg"
                    >
                        {#if PLAYGROUND_CONFIG[activeLanguage].embeddable}
                            <iframe
                                title={$t("playground.iframe_title", {
                                    values: { name: PLAYGROUND_CONFIG[activeLanguage].label },
                                })}
                                src={PLAYGROUND_CONFIG[activeLanguage].embedUrl}
                                class="w-full min-h-[480px]"
                                loading="lazy"
                                sandbox="allow-scripts allow-same-origin allow-forms"
                            ></iframe>
                        {:else}
                            <div class="p-6 text-sm text-muted-foreground dark:text-dark-text-muted space-y-2">
                                <p>{$t("playground.embed_unavailable")}</p>
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>
        {/if}
    </section>
</div>
