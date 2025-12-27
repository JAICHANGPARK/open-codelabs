<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import { X, Sparkles, Loader2, ArrowRight } from "lucide-svelte";
    import { streamGeminiResponseRobust } from "$lib/gemini";
    import { createCodelab, saveSteps, type Codelab } from "$lib/api";
    import { t } from "svelte-i18n";

    let { apiKey, onClose, onCodelabCreated } = $props<{
        apiKey: string;
        onClose: () => void;
        onCodelabCreated: (codelab: Codelab) => void;
    }>();

    let sourceCode = $state("");
    let loading = $state(false);
    let generationStep = $state<"input" | "generating" | "review">("input");
    let generatedContent = $state("");
    let parsedData = $state<{
        title: string;
        description: string;
        steps: { title: string; content: string }[];
    } | null>(null);

    const SYSTEM_PROMPT = `
You are an expert technical writer and developer advocate. 
Your goal is to convert the provided source code into an engaging, step-by-step generic "Codelab" or tutorial.

Return the response **strictly** as a valid JSON object with the following structure:
{
  "title": "Codelab Title",
  "description": "Brief description of what will be built",
  "steps": [
    {
      "title": "Step Title (e.g., Setting up the Project)",
      "content": "Markdown content for this step. Explain the code clearly. Use code blocks."
    }
  ]
}

- Break down the code into logical steps.
- Explain "why" we are doing this, not just "what".
- Use generic clear markdown.
- Do NOT include any markdown formatting outside the JSON structure (e.g., no \`\`\`json wrappers).
- Ensure the JSON is valid.
`;

    async function handleGenerate() {
        if (!sourceCode.trim() || !apiKey) return;

        loading = true;
        generationStep = "generating";
        generatedContent = "";
        parsedData = null;

        const prompt = `Here is the source code:\n\n${sourceCode}\n\nCreate a codelab from this.`;

        try {
            const stream = streamGeminiResponseRobust(prompt, SYSTEM_PROMPT, {
                apiKey,
            });

            for await (const chunk of stream) {
                generatedContent += chunk;
            }

            // Attempt to parse JSON
            // Clean up any markdown code blocks if the model still adds them
            let jsonStr = generatedContent.trim();
            if (jsonStr.startsWith("```json")) {
                jsonStr = jsonStr.replace(/^```json/, "").replace(/```$/, "");
            } else if (jsonStr.startsWith("```")) {
                jsonStr = jsonStr.replace(/^```/, "").replace(/```$/, "");
            }

            try {
                parsedData = JSON.parse(jsonStr);
                generationStep = "review";
            } catch (e) {
                console.error("JSON Parse Error", e);
                alert("Failed to parse AI response. Please try again.");
                generationStep = "input";
            }
        } catch (e: any) {
            console.error("Generation failed", e);
            alert("Generation failed: " + e.message);
            generationStep = "input";
        } finally {
            loading = false;
        }
    }

    async function handleSave() {
        if (!parsedData) return;
        loading = true;
        try {
            // 1. Create Codelab
            const codelab = await createCodelab({
                title: parsedData.title,
                description: parsedData.description,
                author: "AI Assistant",
            });

            // 2. Save Steps
            const stepsPayload = parsedData.steps.map((s) => ({
                title: s.title,
                content_markdown: s.content,
            }));
            await saveSteps(codelab.id, stepsPayload);

            onCodelabCreated(codelab);
        } catch (e) {
            console.error("Failed to save codelab", e);
            alert("Failed to save codelab.");
        } finally {
            loading = false;
        }
    }
</script>

<div
    class="fixed inset-0 bg-[#202124]/60 backdrop-blur-sm flex items-center justify-center p-4 z-50"
>
    <div
        class="bg-white rounded-3xl shadow-2xl w-full max-w-4xl h-[85vh] flex flex-col overflow-hidden relative"
        in:fly={{ y: 20, duration: 400 }}
    >
        <!-- Header -->
        <div
            class="bg-gradient-to-r from-[#8E24AA] to-[#D81B60] p-6 text-white shrink-0"
        >
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <div class="bg-white/20 p-2 rounded-lg">
                        <Sparkles size={24} />
                    </div>
                    <div>
                        <h2 class="text-2xl font-bold">
                            Create with Gemini AI
                        </h2>
                        <p class="opacity-80 text-sm">
                            Turn your code into an interactive Codelab instantly
                        </p>
                    </div>
                </div>
                <button
                    onclick={onClose}
                    class="p-2 hover:bg-white/10 rounded-full transition-colors"
                >
                    <X size={24} />
                </button>
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-hidden p-6 bg-[#F8F9FA]">
            {#if generationStep === "input"}
                <div class="h-full flex flex-col gap-4" in:fade>
                    <label
                        for="source-code"
                        class="text-[#5F6368] font-bold text-lg"
                        >Paste your source code here</label
                    >
                    <textarea
                        id="source-code"
                        bind:value={sourceCode}
                        placeholder="// Paste your Rust, TypeScript, Python... code here&#10;// Gemini will explain it step-by-step."
                        class="flex-1 w-full border border-[#DADCE0] rounded-xl p-4 font-mono text-sm focus:border-[#8E24AA] focus:ring-4 focus:ring-[#8E24AA]/10 outline-none resize-none shadow-sm transition-all"
                    ></textarea>

                    <div class="flex justify-end pt-2">
                        {#if !apiKey}
                            <p
                                class="text-[#EA4335] font-bold mr-4 self-center"
                            >
                                Please set your Gemini API Key in Settings
                                first.
                            </p>
                            <button
                                disabled
                                class="bg-[#E8EAED] text-[#9AA0A6] px-8 py-3 rounded-full font-bold cursor-not-allowed"
                            >
                                Generate
                            </button>
                        {:else}
                            <button
                                onclick={handleGenerate}
                                disabled={!sourceCode.trim()}
                                class="bg-gradient-to-r from-[#8E24AA] to-[#D81B60] text-white px-8 py-3 rounded-full font-bold hover:shadow-lg hover:scale-105 transition-all text-lg flex items-center gap-2 disabled:opacity-50 disabled:hover:scale-100"
                            >
                                <Sparkles size={20} />
                                Generate Codelab
                            </button>
                        {/if}
                    </div>
                </div>
            {:else if generationStep === "generating"}
                <div
                    class="h-full flex flex-col items-center justify-center gap-6"
                    in:fade
                >
                    <div class="relative">
                        <div
                            class="absolute inset-0 bg-gradient-to-r from-[#8E24AA] to-[#D81B60] rounded-full blur-xl opacity-20 animate-pulse"
                        ></div>
                        <Loader2
                            class="w-16 h-16 text-[#D81B60] animate-spin relative z-10"
                        />
                    </div>
                    <h3 class="text-xl font-bold text-[#3C4043]">
                        Analyzing your code...
                    </h3>
                    <p class="text-[#5F6368]">
                        Gemini is crafting a step-by-step guide<br />from your
                        source code.
                    </p>

                    <!-- Preview of raw stream just to show activity -->
                    <div
                        class="w-full max-w-2xl h-32 overflow-hidden text-xs text-[#9AA0A6] font-mono text-center opacity-50 relative mt-8"
                    >
                        <div
                            class="absolute inset-x-0 bottom-0 h-12 bg-gradient-to-t from-[#F8F9FA] to-transparent"
                        ></div>
                        {generatedContent.slice(-500)}
                    </div>
                </div>
            {:else if generationStep === "review" && parsedData}
                <div class="h-full flex flex-col gap-6" in:fade>
                    <div
                        class="flex items-center justify-between border-b border-[#E8EAED] pb-4"
                    >
                        <div>
                            <h3 class="text-xl font-bold text-[#202124]">
                                Preview
                            </h3>
                            <p class="text-[#5F6368] text-sm">
                                Review the generated codelab structure
                            </p>
                        </div>
                        <div class="flex gap-3">
                            <button
                                onclick={() => (generationStep = "input")}
                                class="px-6 py-2 text-[#5F6368] font-bold hover:bg-[#E8EAED] rounded-full transition-all"
                            >
                                Back
                            </button>
                            <button
                                onclick={handleSave}
                                disabled={loading}
                                class="bg-[#34A853] text-white px-8 py-2 rounded-full font-bold hover:bg-[#1E8E3E] shadow-md transition-all flex items-center gap-2"
                            >
                                {#if loading}
                                    <Loader2 class="animate-spin" size={18} />
                                    Saving...
                                {:else}
                                    <ArrowRight size={18} />
                                    Create Codelab
                                {/if}
                            </button>
                        </div>
                    </div>

                    <div
                        class="flex-1 overflow-y-auto bg-white rounded-xl border border-[#E8EAED] p-8 shadow-sm"
                    >
                        <h1 class="text-3xl font-bold text-[#202124] mb-4">
                            {parsedData.title}
                        </h1>
                        <p class="text-lg text-[#5F6368] mb-8">
                            {parsedData.description}
                        </p>

                        <div class="space-y-8">
                            {#each parsedData.steps as step, i}
                                <div
                                    class="border border-[#F1F3F4] rounded-lg p-6 hover:shadow-sm transition-shadow"
                                >
                                    <h4
                                        class="font-bold text-lg text-[#202124] mb-2"
                                    >
                                        {i + 1}. {step.title}
                                    </h4>
                                    <div
                                        class="text-[#3C4043] text-sm line-clamp-3 opacity-80"
                                    >
                                        {step.content}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                </div>
            {/if}
        </div>
    </div>
</div>
