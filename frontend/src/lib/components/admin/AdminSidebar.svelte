<script lang="ts">
    import { Plus, X, Trash2, Copy, Check } from "lucide-svelte";
    import { t } from "svelte-i18n";
    // @ts-ignore
    import QRCode from "svelte-qrcode";
    import type { Step } from "$lib/api";

    let {
        steps = $bindable(),
        activeStepIndex = $bindable(),
        isSidebarOpen = $bindable(),
        attendeeUrl,
        copySuccess = $bindable(),
        addStep,
        removeStep,
        handleCopyUrl,
    } = $props<{
        steps: Step[];
        activeStepIndex: number;
        isSidebarOpen: boolean;
        attendeeUrl: string;
        copySuccess: boolean;
        addStep: () => void;
        removeStep: (index: number) => void;
        handleCopyUrl: () => void;
    }>();

    let draggedStepIndex = $state<number | null>(null);
    let dragOverIndex = $state<number | null>(null);

    function handleDragStart(e: DragEvent, index: number) {
        draggedStepIndex = index;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = "move";
        }
    }

    function handleDragOver(e: DragEvent, index: number) {
        e.preventDefault();
        dragOverIndex = index;
    }

    function handleDragLeave() {
        dragOverIndex = null;
    }

    function handleDrop(e: DragEvent, index: number) {
        e.preventDefault();
        if (draggedStepIndex === null || draggedStepIndex === index) return;

        const newSteps = [...steps];
        const [removed] = newSteps.splice(draggedStepIndex, 1);
        newSteps.splice(index, 0, removed);

        // Update step numbers
        steps = newSteps.map((step, i) => ({
            ...step,
            step_number: i + 1,
        }));

        if (activeStepIndex === draggedStepIndex) {
            activeStepIndex = index;
        } else if (
            draggedStepIndex < activeStepIndex &&
            index >= activeStepIndex
        ) {
            activeStepIndex--;
        } else if (
            draggedStepIndex > activeStepIndex &&
            index <= activeStepIndex
        ) {
            activeStepIndex++;
        }

        draggedStepIndex = null;
        dragOverIndex = null;
    }

    function handleDragEnd() {
        draggedStepIndex = null;
        dragOverIndex = null;
    }
</script>

<div
    class="fixed inset-0 z-50 lg:z-30 lg:relative lg:inset-auto lg:col-span-3 lg:block transition-all duration-300 {isSidebarOpen
        ? 'translate-x-0 opacity-100'
        : '-translate-x-full opacity-0 lg:translate-x-0 lg:opacity-100 lg:sticky lg:top-28'}"
>
    <!-- Overlay for mobile -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="absolute inset-0 bg-black/50 backdrop-blur-sm lg:hidden"
        onclick={() => (isSidebarOpen = false)}
    ></div>

    <div
        class="relative bg-white dark:bg-dark-surface rounded-2xl border border-border dark:border-dark-border overflow-hidden shadow-xl lg:shadow-sm w-4/5 lg:w-full max-w-sm lg:max-w-none h-[90vh] lg:h-auto m-4 lg:m-0 flex flex-col"
    >
        <div
            class="p-5 border-b border-border dark:border-dark-border bg-muted dark:bg-white/5 flex justify-between items-center"
        >
            <span
                class="text-xs font-bold text-muted-foreground dark:text-dark-text-muted uppercase tracking-widest"
                >{$t("editor.step_navigation")}</span
            >
            <div class="flex items-center gap-2">
                <button
                    type="button"
                    onclick={addStep}
                    class="text-primary hover:bg-accent/70 dark:hover:bg-primary/10 p-1.5 rounded-full transition-colors"
                    title={$t("editor.add_step")}
                    aria-label={$t("editor.add_step")}
                >
                    <Plus size={18} />
                </button>
                <button
                    type="button"
                    onclick={() => (isSidebarOpen = false)}
                    class="lg:hidden p-1.5 hover:bg-accent/70 dark:hover:bg-white/5 rounded-full transition-colors"
                    aria-label={$t("common.close")}
                >
                    <X size={18} />
                </button>
            </div>
        </div>
        <div class="flex-1 overflow-y-auto max-h-[50vh] lg:max-h-[60vh]">
            {#each steps as step, i}
                <div
                    role="listitem"
                    class="group relative {dragOverIndex === i
                        ? 'border-t-4 border-primary'
                        : ''}"
                    draggable="true"
                    ondragstart={(e) => handleDragStart(e, i)}
                    ondragover={(e) => handleDragOver(e, i)}
                    ondragleave={handleDragLeave}
                    ondrop={(e) => handleDrop(e, i)}
                    ondragend={handleDragEnd}
                >
                    <button
                        onclick={() => {
                            activeStepIndex = i;
                            isSidebarOpen = false;
                        }}
                        class="w-full text-left px-5 py-4 hover:bg-muted dark:hover:bg-white/5 transition-all flex items-start gap-4 border-l-4 cursor-pointer {activeStepIndex ===
                        i
                            ? 'border-primary bg-accent/30 dark:bg-primary/10'
                            : 'border-transparent'} {draggedStepIndex === i
                            ? 'opacity-50'
                            : ''}"
                    >
                        <span
                            class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold shrink-0 {activeStepIndex ===
                            i
                                ? 'bg-primary text-white'
                                : 'bg-accent/60 dark:bg-white/10 text-muted-foreground dark:text-dark-text-muted'}"
                            >{i + 1}</span
                        >
                        <span
                            class="text-sm font-bold {activeStepIndex === i
                                ? 'text-primary dark:text-primary'
                                : 'text-muted-foreground dark:text-dark-text-muted'} line-clamp-1 pt-0.5 pr-6"
                            >{step.title}</span
                        >
                    </button>
                    <button
                        type="button"
                        onclick={() => removeStep(i)}
                        class="absolute right-3 top-1/2 -translate-y-1/2 p-1.5 text-muted-foreground/60 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg lg:opacity-0 lg:group-hover:opacity-100 transition-all"
                        title={$t("editor.delete_step")}
                        aria-label={$t("editor.delete_step")}
                    >
                        <Trash2 size={14} />
                    </button>
                </div>
            {/each}
        </div>

        <div
            class="p-6 border-t border-border dark:border-dark-border bg-muted/50 dark:bg-white/5 flex flex-col items-center"
        >
            <div
                class="bg-white p-3 rounded-2xl border border-border dark:border-dark-border shadow-sm mb-4"
            >
                <QRCode value={attendeeUrl} size={140} />
            </div>
            <p
                class="text-[11px] text-muted-foreground dark:text-dark-text-muted text-center uppercase tracking-widest font-bold mb-3"
            >
                {$t("editor.attendee_access")}
            </p>

            <div
                class="w-full flex items-center gap-2 p-2 bg-white dark:bg-dark-bg border border-border dark:border-dark-border rounded-xl shadow-sm overflow-hidden"
            >
                <input
                    type="text"
                    readonly
                    value={attendeeUrl}
                    aria-label={$t("editor.attendee_access")}
                    class="flex-1 bg-transparent border-none text-[10px] text-muted-foreground dark:text-dark-text-muted px-2 outline-none"
                />
                <button
                    type="button"
                    onclick={handleCopyUrl}
                    class="p-2 hover:bg-accent/60 dark:hover:bg-white/10 rounded-lg transition-colors text-primary"
                    title={$t("editor.copy_url")}
                    aria-label={$t("editor.copy_url")}
                >
                    {#if copySuccess}
                        <Check size={14} />
                    {:else}
                        <Copy size={14} />
                    {/if}
                </button>
            </div>
        </div>
    </div>
</div>
