<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import { X, Sparkles, Loader2, ArrowRight, Info } from "lucide-svelte";
    import {
        streamGeminiStructuredOutput,
        type GeminiStructuredConfig,
    } from "$lib/gemini";
    import { createCodelab, saveSteps, type Codelab } from "$lib/api";
    import { t, locale } from "svelte-i18n";

    let { apiKey, onClose, onCodelabCreated } = $props<{
        apiKey: string;
        onClose: () => void;
        onCodelabCreated: (codelab: Codelab) => void;
    }>();

    let sourceCode = $state("");
    let loading = $state(false);
    let generationStep = $state<"input" | "generating" | "review">("input");
    let generatedContent = $state("");
    let thinkingContent = $state("");
    let showThinking = $state(true);
    let useGoogleSearch = $state(false);
    let useUrlContext = $state(false);
    let parsedData = $state<{
        title: string;
        description: string;
        steps: { title: string; content: string }[];
    } | null>(null);

    //     const SYSTEM_PROMPT = `
    // You are an expert technical writer and developer advocate.
    // Your goal is to convert the provided source code into an engaging, step-by-step generic "Codelab" or tutorial.

    // - Break down the code into logical steps.
    // - Explain "why" we are doing this, not just "what".
    // - Use clear markdown with code blocks.
    // - Create comprehensive, educational content.
    // `;

    const SYSTEM_PROMPT = `
You are a world-class Technical Content Engineer and Developer Advocate. 
Your mission is to transform raw source code into a high-quality, professional "Hands-on Codelab" that ensures a seamless developer experience.

Follow these strict guidelines to create the content:

1. STRUCTURE & HIERARCHY:
- Title: Engaging and clear.
- Overview: What will be built and what are the key learning objectives?
- Prerequisites: Detailed system requirements (Language versions, CLI tools).
- Environment Setup: 
    * System configurations (Environment variables, OS-specific notes).
    * IDE Recommendation & Configuration (VS Code, IntelliJ, etc.).
    * Required/Recommended Plugins/Extensions (e.g., Prettier, ESLint, Language-specific plugins).
- Step-by-Step Implementation: Logical progression from boilerplate to advanced logic.
- Verification: How to test if each step was successful.
- Conclusion & Next Steps: Summary and challenge for the reader.

2. DEPTH OF CONTENT:
- "The Why before the How": Explain the architectural decisions or why a specific configuration is needed.
- IDE Integration: Don't just show code; tell the user how the IDE can help (e.g., "Use 'Cmd+Shift+P' to run this command").
- Error Prevention: Add "Pro-tips" or "Note" boxes for common pitfalls in system setup.

3. TECHNICAL PRECISION:
- Use clear Markdown headings and syntax highlighting.
- Provide shell commands for installation (e.g., npm install, brew install).
- If specific IDE settings (settings.json) or plugin IDs are relevant, include them.

4. TONE & STYLE:
- Professional, encouraging, and action-oriented.
- Use the "Instruction -> Code -> Explanation -> Verification" loop for every step.
`;

    async function handleGenerate() {
        if (!sourceCode.trim() || !apiKey) return;

        loading = true;
        generationStep = "generating";
        generatedContent = "";
        thinkingContent = "";
        parsedData = null;

        // Detect user language
        const userLanguage = $locale || "en";
        const languageNames: Record<string, string> = {
            ko: "Korean",
            en: "English",
            zh: "Chinese",
            ja: "Japanese",
        };
        const targetLanguage = languageNames[userLanguage] || "English";

        // Define JSON Schema for the codelab structure
        const codelabSchema = {
            type: "object",
            properties: {
                title: {
                    type: "string",
                    description: `The name of the codelab in ${targetLanguage}`,
                },
                description: {
                    type: "string",
                    description: `Brief description in ${targetLanguage} of what will be built`,
                },
                steps: {
                    type: "array",
                    items: {
                        type: "object",
                        properties: {
                            title: {
                                type: "string",
                                description: `Step title in ${targetLanguage} (e.g., Setting up the Project)`,
                            },
                            content: {
                                type: "string",
                                description: `Markdown content in ${targetLanguage} for this step. Explain the code clearly. Use code blocks.`,
                            },
                        },
                        required: ["title", "content"],
                    },
                },
            },
            required: ["title", "description", "steps"],
        };

        const prompt = `Create a codelab tutorial from the following source code. Write ALL content in ${targetLanguage}.\n\nSource code:\n${sourceCode}`;

        // Build tools array
        const tools: GeminiStructuredConfig["tools"] = [];
        if (useGoogleSearch) {
            tools.push({ googleSearch: {} });
        }
        if (useUrlContext) {
            tools.push({ urlContext: {} });
        }

        try {
            const stream = streamGeminiStructuredOutput(
                prompt,
                SYSTEM_PROMPT,
                codelabSchema,
                {
                    apiKey,
                    model: "gemini-3-flash-preview",
                    tools: tools.length > 0 ? tools : undefined,
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );

            for await (const chunk of stream) {
                if (chunk.thinking) {
                    thinkingContent += chunk.thinking;
                }
                if (chunk.content) {
                    generatedContent += chunk.content;
                }
            }

            // With structured outputs, we get guaranteed valid JSON
            try {
                parsedData = JSON.parse(generatedContent);
                generationStep = "review";
            } catch (e) {
                console.error(
                    "JSON Parse Error (should not happen with structured outputs)",
                    e,
                );
                console.error("Response:", generatedContent);
                alert(
                    "Unexpected error parsing AI response. Please try again.",
                );
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
            class="bg-[#8E24AA] p-6 text-white shrink-0"
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

                    <!-- Advanced Options -->
                    <div class="flex flex-wrap gap-4 mb-4">
                        <label
                            class="flex items-center gap-2 cursor-pointer group"
                        >
                            <input
                                type="checkbox"
                                bind:checked={useGoogleSearch}
                                class="w-5 h-5 rounded border-gray-300 text-[#8E24AA] focus:ring-[#8E24AA]"
                            />
                            <span
                                class="text-sm font-medium text-[#5F6368] group-hover:text-[#8E24AA]"
                            >
                                Google Search (Real-time data)
                            </span>
                        </label>

                        <label
                            class="flex items-center gap-2 cursor-pointer group"
                        >
                            <input
                                type="checkbox"
                                bind:checked={useUrlContext}
                                class="w-5 h-5 rounded border-gray-300 text-[#8E24AA] focus:ring-[#8E24AA]"
                            />
                            <span
                                class="text-sm font-medium text-[#5F6368] group-hover:text-[#8E24AA]"
                            >
                                URL Context
                            </span>
                        </label>

                        <label
                            class="flex items-center gap-2 cursor-pointer group"
                        >
                            <input
                                type="checkbox"
                                bind:checked={showThinking}
                                class="w-5 h-5 rounded border-gray-300 text-[#8E24AA] focus:ring-[#8E24AA]"
                            />
                            <span
                                class="text-sm font-medium text-[#5F6368] group-hover:text-[#8E24AA]"
                            >
                                Show AI Thinking
                            </span>
                        </label>
                    </div>

                    {#if useGoogleSearch || useUrlContext}
                        <div
                            class="flex items-start gap-2 p-3 bg-[#FEF7E0] border border-[#F9AB00]/30 rounded-lg mb-4"
                        >
                            <Info
                                size={16}
                                class="text-[#F9AB00] mt-0.5 shrink-0"
                            />
                            <p class="text-xs text-[#3C4043]">
                                <strong>Billing Notice:</strong> Google Search and
                                URL Context tools may incur additional charges starting
                                January 5, 2026.
                            </p>
                        </div>
                    {/if}

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
                                class="bg-[#8E24AA] text-white px-8 py-3 rounded-full font-bold hover:shadow-lg hover:scale-105 transition-all text-lg flex items-center gap-2 disabled:opacity-50 disabled:hover:scale-100"
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
                            class="absolute inset-0 bg-[#8E24AA] rounded-full blur-xl opacity-20 animate-pulse"
                        ></div>
                        <Loader2
                            class="w-16 h-16 text-[#8E24AA] animate-spin relative z-10"
                        />
                    </div>
                    <h3 class="text-xl font-bold text-[#3C4043]">
                        Analyzing your code...
                    </h3>
                    <p class="text-[#5F6368]">
                        Gemini is crafting a step-by-step guide<br />from your
                        source code.
                    </p>

                    <!-- Thinking Display -->
                    {#if showThinking && thinkingContent}
                        <div class="w-full max-w-2xl mt-6">
                            <details
                                open
                                class="bg-white rounded-xl border border-[#E8EAED] shadow-sm overflow-hidden"
                            >
                                <summary
                                    class="px-4 py-3 cursor-pointer hover:bg-[#F8F9FA] flex items-center gap-2 font-medium text-[#5F6368]"
                                >
                                    <Sparkles
                                        size={16}
                                        class="text-[#8E24AA]"
                                    />
                                    AI Thinking Process
                                </summary>
                                <div
                                    class="px-4 py-3 text-xs text-[#5F6368] font-mono bg-[#F8F9FA] max-h-48 overflow-y-auto border-t border-[#E8EAED]"
                                >
                                    {thinkingContent}
                                </div>
                            </details>
                        </div>
                    {/if}

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
