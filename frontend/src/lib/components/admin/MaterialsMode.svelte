<script lang="ts">
    import { 
        Plus, 
        Trash2, 
        ExternalLink, 
        Download, 
        CheckCircle2 
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import type { Material } from "$lib/api";

    let {
        materials,
        newMaterial = $bindable(),
        materialFileInput = $bindable(),
        handleMaterialFileSelect,
        handleAddMaterial,
        handleDeleteMaterial
    } = $props<{
        materials: Material[];
        newMaterial: {
            title: string;
            material_type: "link" | "file";
            link_url: string;
            file_path: string;
        };
        materialFileInput: HTMLInputElement | undefined;
        handleMaterialFileSelect: (e: Event) => void;
        handleAddMaterial: () => void;
        handleDeleteMaterial: (id: string) => void;
    }>();
</script>

<div
    class="flex-1 flex flex-col p-6 sm:p-8 space-y-8 overflow-y-auto max-h-[75vh]"
>
    <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4"
    >
        <div>
            <h2
                class="text-2xl font-bold text-[#202124] dark:text-dark-text mb-1"
            >
                {$t("editor.materials_title")}
            </h2>
        </div>
    </div>

    <!-- Material Form -->
    <div
        class="bg-[#F8F9FA] dark:bg-white/5 p-6 rounded-2xl border border-[#E8EAED] dark:border-dark-border space-y-4 shadow-sm"
    >
        <div
            class="grid grid-cols-1 md:grid-cols-2 gap-4"
        >
            <div class="space-y-2">
                <label
                    for="mat-name"
                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                    >{$t(
                        "editor.material_name",
                    )}</label
                >
                <input
                    id="mat-name"
                    type="text"
                    bind:value={newMaterial.title}
                    placeholder={$t(
                        "editor.material_placeholder_name",
                    )}
                    class="w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-3 outline-none focus:ring-2 focus:ring-[#4285F4] transition-all dark:text-dark-text shadow-sm"
                />
            </div>
            <div class="space-y-2">
                <label
                    for="mat-type"
                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                    >{$t(
                        "editor.material_type",
                    )}</label
                >
                <select
                    id="mat-type"
                    bind:value={newMaterial.material_type}
                    class="w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-3 outline-none focus:ring-2 focus:ring-[#4285F4] transition-all dark:text-dark-text shadow-sm"
                >
                    <option value="link">Link</option>
                    <option value="file">File</option>
                </select>
            </div>
        </div>

        {#if newMaterial.material_type === "link"}
            <div class="space-y-2">
                <label
                    for="mat-link"
                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                    >{$t(
                        "editor.material_link",
                    )}</label
                >
                <input
                    id="mat-link"
                    type="text"
                    bind:value={newMaterial.link_url}
                    placeholder={$t(
                        "editor.material_placeholder_link",
                    )}
                    class="w-full bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl p-3 outline-none focus:ring-2 focus:ring-[#4285F4] transition-all dark:text-dark-text shadow-sm"
                />
            </div>
        {:else}
            <div class="space-y-2">
                <label
                    for="mat-file-upload"
                    class="text-xs font-bold text-[#5F6368] dark:text-dark-text-muted uppercase tracking-wider"
                    >{$t(
                        "editor.material_file",
                    )}</label
                >
                <div
                    class="flex items-center gap-4"
                >
                    <button
                        id="mat-file-upload"
                        onclick={() =>
                            materialFileInput?.click()}
                        class="flex items-center gap-2 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border hover:bg-[#F1F3F4] dark:hover:bg-white/10 px-4 py-2.5 rounded-xl transition-all shadow-sm text-sm"
                    >
                        <Plus size={18} />
                        <span
                            >{$t(
                                "editor.upload_file",
                            )}</span
                        >
                    </button>
                    {#if newMaterial.file_path}
                        <span
                            class="text-sm text-[#1E8E3E] flex items-center gap-1"
                            ><CheckCircle2
                                size={14}
                            /> Uploaded</span
                        >
                    {/if}
                </div>
                <input
                    type="file"
                    bind:this={materialFileInput}
                    onchange={handleMaterialFileSelect}
                    class="hidden"
                />
            </div>
        {/if}

        <div class="flex justify-end pt-2">
            <button
                onclick={handleAddMaterial}
                disabled={!newMaterial.title ||
                    (newMaterial.material_type ===
                    "link"
                        ? !newMaterial.link_url
                        : !newMaterial.file_path)}
                class="bg-[#4285F4] hover:bg-[#1A73E8] disabled:opacity-50 text-white px-8 py-3 rounded-xl font-bold transition-all shadow-md active:scale-95 flex items-center gap-2"
            >
                <Plus size={18} />
                {$t("editor.add_material")}
            </button>
        </div>
    </div>

    <!-- Material List -->
    <div class="space-y-4">
        {#if materials.length > 0}
            <div
                class="grid grid-cols-1 md:grid-cols-2 gap-4"
            >
                {#each materials as mat}
                    <div
                        class="flex items-center justify-between p-4 bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-2xl shadow-sm hover:shadow-md transition-all group"
                    >
                        <div
                            class="flex items-center gap-3 min-w-0"
                        >
                            <div
                                class="p-2.5 bg-[#F1F3F4] dark:bg-white/10 rounded-xl text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4] transition-colors shrink-0"
                            >
                                {#if mat.material_type === "link"}
                                    <ExternalLink
                                        size={20}
                                    />
                                {:else}
                                    <Download
                                        size={20}
                                    />
                                {/if}
                            </div>
                            <div class="min-w-0">
                                <h4
                                    class="font-bold text-[#202124] dark:text-dark-text truncate"
                                >
                                    {mat.title}
                                </h4>
                                <p
                                    class="text-xs text-[#5F6368] dark:text-dark-text-muted truncate"
                                >
                                    {mat.material_type ===
                                    "link"
                                        ? mat.link_url
                                        : mat.file_path
                                              ?.split("/")
                                              .pop()}
                                </p>
                            </div>
                        </div>
                        <button
                            type="button"
                            onclick={() =>
                                handleDeleteMaterial(
                                    mat.id,
                                )}
                            class="p-2 text-[#5F6368] dark:text-dark-text-muted hover:text-[#EA4335] hover:bg-[#FCE8E6] dark:hover:bg-[#EA4335]/10 rounded-lg transition-all opacity-0 group-hover:opacity-100 shrink-0"
                            aria-label={$t("common.delete")}
                        >
                            <Trash2 size={18} />
                        </button>
                    </div>
                {/each}
            </div>
        {:else}
            <p
                class="text-center py-12 text-[#9AA0A6] dark:text-dark-text-muted"
            >
                {$t("editor.no_materials")}
            </p>
        {/if}
    </div>
</div>
