<script lang="ts">
    import {
        Images,
        Download,
        ExternalLink,
        User,
        Calendar,
        Search,
        Grid3X3,
        LayoutGrid,
        X,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { ASSET_URL, type SubmissionWithAttendee } from "$lib/api";
    import { fade, fly } from "svelte/transition";

    let { submissions = [] } = $props<{
        submissions: SubmissionWithAttendee[];
    }>();

    let searchTerm = $state("");
    let selectedImage = $state<SubmissionWithAttendee | null>(null);
    let viewMode = $state<"grid" | "masonry">("grid");

    // Filter only image submissions
    let imageSubmissions = $derived(
        submissions.filter((sub: SubmissionWithAttendee) =>
            isImage(sub.file_name, sub.submission_type),
        ),
    );

    let filteredImages = $derived(
        imageSubmissions.filter(
            (s: SubmissionWithAttendee) =>
                s.attendee_name
                    .toLowerCase()
                    .includes(searchTerm.toLowerCase()) ||
                s.file_name.toLowerCase().includes(searchTerm.toLowerCase()),
        ),
    );

    function isImage(fileName: string, type?: string) {
        if (type === "link") return false;
        const ext = fileName.split(".").pop()?.toLowerCase();
        return [
            "jpg",
            "jpeg",
            "png",
            "gif",
            "webp",
            "svg",
            "bmp",
            "ico",
            "heic",
            "heif",
        ].includes(ext || "");
    }

    function isHeic(fileName: string) {
        const ext = fileName.split(".").pop()?.toLowerCase();
        return ext === "heic" || ext === "heif";
    }

    function getSubmissionUrl(sub: SubmissionWithAttendee) {
        if (sub.file_path.startsWith("http")) return sub.file_path;
        return `${ASSET_URL}${sub.file_path}`;
    }

    function formatDate(dateStr?: string) {
        if (!dateStr) return "";
        try {
            return new Date(dateStr).toLocaleString();
        } catch (e) {
            return dateStr;
        }
    }

    function openLightbox(sub: SubmissionWithAttendee) {
        selectedImage = sub;
    }

    function closeLightbox() {
        selectedImage = null;
    }

    function downloadImage(sub: SubmissionWithAttendee) {
        const link = document.createElement("a");
        link.href = getSubmissionUrl(sub);
        link.download = sub.file_name;
        link.target = "_blank";
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    }
</script>

<div class="space-y-6" in:fade>
    <div
        class="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-accent/60 dark:bg-white/5 p-6 rounded-2xl border border-border dark:border-dark-border"
    >
        <div>
            <h2
                class="text-xl font-bold text-foreground dark:text-dark-text flex items-center gap-2"
            >
                <Images size={24} class="text-primary" />
                {$t("gallery.title")}
            </h2>
            <p
                class="text-sm text-muted-foreground dark:text-dark-text-muted mt-1"
            >
                {imageSubmissions.length}
                {$t("gallery.image_count")}
            </p>
        </div>

        <div class="flex items-center gap-3">
            <!-- View Mode Toggle -->
            <div
                class="flex items-center bg-white dark:bg-dark-surface rounded-full border border-border dark:border-dark-border p-1"
            >
                <button
                    onclick={() => (viewMode = "grid")}
                    class="p-2 rounded-full transition-all {viewMode === 'grid'
                        ? 'bg-primary text-white'
                        : 'text-muted-foreground hover:text-foreground'}"
                    title={$t("gallery.grid_view")}
                >
                    <LayoutGrid size={16} />
                </button>
                <button
                    onclick={() => (viewMode = "masonry")}
                    class="p-2 rounded-full transition-all {viewMode ===
                    'masonry'
                        ? 'bg-primary text-white'
                        : 'text-muted-foreground hover:text-foreground'}"
                    title={$t("gallery.masonry_view")}
                >
                    <Grid3X3 size={16} />
                </button>
            </div>

            <div class="relative max-w-xs w-full">
                <Search
                    size={18}
                    class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                />
                <input
                    type="text"
                    bind:value={searchTerm}
                    placeholder={$t("gallery.search_placeholder")}
                    aria-label={$t("gallery.search_placeholder")}
                    class="w-full pl-10 pr-4 py-2 bg-white dark:bg-dark-surface border border-border dark:border-dark-border rounded-full text-sm outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all"
                />
            </div>
        </div>
    </div>

    {#if filteredImages.length === 0}
        <div
            class="flex flex-col items-center justify-center py-20 text-center bg-white dark:bg-dark-surface rounded-3xl border-2 border-dashed border-border dark:border-dark-border"
        >
            <div
                class="w-20 h-20 bg-accent/60 dark:bg-white/5 rounded-full flex items-center justify-center mb-6"
            >
                <Images size={40} class="text-muted-foreground/70" />
            </div>
            <h3 class="text-lg font-bold text-foreground dark:text-dark-text">
                {$t("gallery.no_images")}
            </h3>
            <p
                class="text-sm text-muted-foreground dark:text-dark-text-muted mt-2"
            >
                {searchTerm
                    ? $t("gallery.no_search_results")
                    : $t("gallery.waiting_images")}
            </p>
        </div>
    {:else if viewMode === "grid"}
        <!-- Grid View -->
        <div
            class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4"
        >
            {#each filteredImages as img, i}
                {#if isHeic(img.file_name)}
                    <!-- HEIC files - show download card since browsers can't display them -->
                    <a
                        href={getSubmissionUrl(img)}
                        download={img.file_name}
                        class="group relative aspect-square rounded-2xl overflow-hidden bg-accent/60 dark:bg-white/5 border border-border dark:border-dark-border hover:shadow-lg transition-all cursor-pointer flex flex-col items-center justify-center p-4"
                        in:fly={{ y: 20, delay: i * 30 }}
                    >
                        <div
                            class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mb-3"
                        >
                            <Download size={32} class="text-primary" />
                        </div>
                        <p
                            class="text-xs font-bold text-foreground dark:text-dark-text text-center truncate w-full"
                        >
                            {img.attendee_name}
                        </p>
                        <p
                            class="text-[10px] text-muted-foreground dark:text-dark-text-muted text-center truncate w-full mt-1"
                        >
                            {img.file_name}
                        </p>
                        <p
                            class="text-[9px] text-primary font-medium mt-2 bg-primary/10 px-2 py-1 rounded-full"
                        >
                            HEIC - {$t("common.download")}
                        </p>
                    </a>
                {:else}
                    <button
                        onclick={() => openLightbox(img)}
                        class="group relative aspect-square rounded-2xl overflow-hidden bg-accent/60 dark:bg-white/5 border border-border dark:border-dark-border hover:shadow-lg transition-all cursor-pointer"
                        in:fly={{ y: 20, delay: i * 30 }}
                    >
                        <img
                            src={getSubmissionUrl(img)}
                            alt={img.file_name}
                            class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                            style="image-orientation: from-image;"
                            loading="lazy"
                        />
                        <div
                            class="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity"
                        >
                            <div
                                class="absolute bottom-0 left-0 right-0 p-3 text-white"
                            >
                                <p class="text-xs font-bold truncate">
                                    {img.attendee_name}
                                </p>
                                <p class="text-[10px] opacity-80 truncate">
                                    {img.file_name}
                                </p>
                            </div>
                        </div>
                    </button>
                {/if}
            {/each}
        </div>
    {:else}
        <!-- Masonry View -->
        <div
            class="columns-2 sm:columns-3 md:columns-4 lg:columns-5 xl:columns-6 gap-4 space-y-4"
        >
            {#each filteredImages as img, i}
                {#if isHeic(img.file_name)}
                    <!-- HEIC files - show download card since browsers can't display them -->
                    <a
                        href={getSubmissionUrl(img)}
                        download={img.file_name}
                        class="group relative break-inside-avoid rounded-2xl overflow-hidden bg-accent/60 dark:bg-white/5 border border-border dark:border-dark-border hover:shadow-lg transition-all cursor-pointer block w-full p-4 mb-4"
                        in:fly={{ y: 20, delay: i * 30 }}
                    >
                        <div
                            class="aspect-[3/4] flex flex-col items-center justify-center"
                        >
                            <div
                                class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mb-3"
                            >
                                <Download size={32} class="text-primary" />
                            </div>
                            <p
                                class="text-xs font-bold text-foreground dark:text-dark-text text-center truncate w-full"
                            >
                                {img.attendee_name}
                            </p>
                            <p
                                class="text-[10px] text-muted-foreground dark:text-dark-text-muted text-center truncate w-full mt-1"
                            >
                                {img.file_name}
                            </p>
                            <p
                                class="text-[9px] text-primary font-medium mt-2 bg-primary/10 px-2 py-1 rounded-full"
                            >
                                HEIC - {$t("common.download")}
                            </p>
                        </div>
                    </a>
                {:else}
                    <button
                        onclick={() => openLightbox(img)}
                        class="group relative break-inside-avoid rounded-2xl overflow-hidden bg-accent/60 dark:bg-white/5 border border-border dark:border-dark-border hover:shadow-lg transition-all cursor-pointer block w-full"
                        in:fly={{ y: 20, delay: i * 30 }}
                    >
                        <img
                            src={getSubmissionUrl(img)}
                            alt={img.file_name}
                            class="w-full h-auto object-cover transition-transform duration-500 group-hover:scale-105"
                            style="image-orientation: from-image;"
                            loading="lazy"
                        />
                        <div
                            class="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity"
                        >
                            <div
                                class="absolute bottom-0 left-0 right-0 p-3 text-white"
                            >
                                <p class="text-xs font-bold truncate">
                                    {img.attendee_name}
                                </p>
                                <p class="text-[10px] opacity-80 truncate">
                                    {img.file_name}
                                </p>
                            </div>
                        </div>
                    </button>
                {/if}
            {/each}
        </div>
    {/if}
</div>

<!-- Lightbox -->
{#if selectedImage}
    <div
        class="fixed inset-0 z-50 bg-black/95 flex items-center justify-center p-4"
        role="dialog"
        tabindex="-1"
        aria-modal="true"
        onclick={closeLightbox}
        onkeydown={(e) => e.key === "Escape" && closeLightbox()}
        transition:fade={{ duration: 200 }}
    >
        <button
            onclick={closeLightbox}
            class="absolute top-4 right-4 p-3 text-white/80 hover:text-white hover:bg-white/10 rounded-full transition-all z-10"
            aria-label={$t("common.close")}
        >
            <X size={24} />
        </button>

        <div
            class="flex flex-col items-center max-w-5xl max-h-[90vh] w-full"
            role="none"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
        >
            {#if selectedImage && isHeic(selectedImage.file_name)}
                <!-- HEIC file - show download prompt -->
                <div
                    class="bg-white/10 backdrop-blur-sm rounded-2xl p-12 text-white text-center"
                >
                    <div
                        class="w-24 h-24 bg-primary/20 rounded-full flex items-center justify-center mx-auto mb-6"
                    >
                        <Download size={48} class="text-primary" />
                    </div>
                    <h3 class="text-xl font-bold mb-2">HEIC Image</h3>
                    <p class="text-sm opacity-70 mb-6">
                        This image format is not supported by browsers.<br
                        />Please download to view.
                    </p>
                    <a
                        href={getSubmissionUrl(selectedImage)}
                        download={selectedImage.file_name}
                        class="inline-flex items-center gap-2 bg-primary hover:bg-primary/90 text-white px-6 py-3 rounded-full font-bold transition-all"
                    >
                        <Download size={20} />
                        {$t("common.download")}
                    </a>
                </div>
            {:else}
                <img
                    src={getSubmissionUrl(selectedImage)}
                    alt={selectedImage.file_name}
                    class="max-w-full max-h-[70vh] object-contain rounded-lg shadow-2xl"
                    style="image-orientation: from-image;"
                />
            {/if}

            <div
                class="mt-6 bg-white/10 backdrop-blur-sm rounded-2xl p-4 text-white max-w-2xl w-full"
            >
                <div class="flex items-center justify-between gap-4">
                    <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2 text-sm mb-1">
                            <User size={14} />
                            <span class="font-bold"
                                >{selectedImage.attendee_name}</span
                            >
                        </div>
                        <p class="text-xs opacity-70 truncate">
                            {selectedImage.file_name}
                        </p>
                        <p class="text-[10px] opacity-50 mt-1">
                            <Calendar size={10} class="inline mr-1" />
                            {formatDate(selectedImage.created_at)}
                        </p>
                    </div>
                    <div class="flex items-center gap-2 shrink-0">
                        <button
                            onclick={() =>
                                selectedImage && downloadImage(selectedImage)}
                            class="p-3 bg-white/20 hover:bg-white/30 rounded-full transition-all"
                            title={$t("common.download")}
                        >
                            <Download size={20} />
                        </button>
                        <a
                            href={getSubmissionUrl(selectedImage)}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="p-3 bg-white/20 hover:bg-white/30 rounded-full transition-all"
                            title={$t("common.open_new_tab")}
                        >
                            <ExternalLink size={20} />
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}
