<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import {
        X,
        Sparkles,
        Loader2,
        ArrowRight,
        Info,
        Clock,
        FileCode,
        Upload,
        Trash2,
        FileText,
    } from "lucide-svelte";
    import {
        streamGeminiStructuredOutput,
        type GeminiStructuredConfig,
    } from "$lib/gemini";
    import { createCodelab, saveSteps, type Codelab } from "$lib/api";
    import { adminMarked as marked } from "$lib/markdown";
    import DOMPurify from "dompurify";
    import { browser } from "$app/environment";
    import { t, locale } from "svelte-i18n";
    import JSZip from "jszip";
    import UploadedFileList from "$lib/components/admin/UploadedFileList.svelte";

    let { apiKey, onClose, onCodelabCreated } = $props<{
        apiKey: string;
        onClose: () => void;
        onCodelabCreated: (codelab: Codelab) => void;
    }>();

    let sourceCode = $state("");
    let uploadedFiles = $state<{ name: string; content: string }[]>([]);
    let loading = $state(false);
    let generationStep = $state<"input" | "generating" | "review">("input");
    let generatedContent = $state("");
    let thinkingContent = $state("");
    let showThinking = $state(true);
    let useGoogleSearch = $state(false);
    let useUrlContext = $state(false);
    let handsOnDuration = $state("60");
    let customDuration = $state("");
    let enableCodeServer = $state(false);
    let workspaceStructureType = $state<"branch" | "folder">("branch");

    // Token usage tracking
    let totalInputTokens = $state(0);
    let totalOutputTokens = $state(0);
    let totalCost = $state(0);

    // Gemini 3 Flash Preview pricing (per 1M tokens)
    const INPUT_PRICE_PER_1M = 0.50;  // $0.50
    const OUTPUT_PRICE_PER_1M = 3.00; // $3.00

    function calculateCost(inputTokens: number, outputTokens: number): number {
        const inputCost = (inputTokens / 1_000_000) * INPUT_PRICE_PER_1M;
        const outputCost = (outputTokens / 1_000_000) * OUTPUT_PRICE_PER_1M;
        return inputCost + outputCost;
    }

    function addTokenUsage(inputTokens: number, outputTokens: number) {
        totalInputTokens += inputTokens;
        totalOutputTokens += outputTokens;
        totalCost = calculateCost(totalInputTokens, totalOutputTokens);
    }
    let parsedData = $state<{
        title: string;
        description: string;
        steps: { title: string; content: string }[];
    } | null>(null);
    type CodelabDraft = {
        title: string;
        description: string;
        steps: { title: string; content: string }[];
    };
    type PlanData = {
        title: string;
        description: string;
        audience: string;
        learning_objectives: string[];
        prerequisites: string[];
        environment_setup: {
            os_requirements: string[];
            tools: string[];
            env_vars: string[];
            ide: string;
            ide_plugins: string[];
        };
        steps: {
            title: string;
            goal: string;
            files: string[];
            verification: string;
        }[];
        search_terms: string[];
    };
    type ReviewData = {
        summary: string;
        issues: {
            severity: string;
            issue: string;
            recommendation: string;
        }[];
        missing_items: string[];
        improvements: string[];
    };
    type PlanComment = {
        id: string;
        quote: string;
        comment: string;
    };
    type GenerationMode = "basic" | "advanced";
    type AdvancedStep =
        | "input"
        | "planning"
        | "plan"
        | "drafting"
        | "draft"
        | "reviewing"
        | "revising"
        | "final";

    let fileInput = $state<HTMLInputElement>();
    const MAX_FILES = 10;
    const MAX_ZIP_FILES = 10;
    const MAX_FILE_SIZE_BYTES = 1024 * 1024 * 1024; // 1GB per file/zip

    const SYSTEM_PROMPT = `
You are a world-class Technical Content Engineer and Developer Advocate. 
Your mission is to transform raw source code into a high-quality, professional "Hands-on Codelab" that ensures a seamless developer experience.

analyzing two types of inputs:
1. [Reference Codelab]: An existing codelab document used as a structural and stylistic template.
2. [Source Code/New Task]: The actual technical content or code that needs to be converted into a new codelab.

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
- For every code block: include inline comments for each logical line AND add a numbered line-by-line explanation right after the block in the same language as the content.
- Before each code block, state the filename/path being edited.

4. TONE & STYLE:
- Professional, encouraging, and action-oriented.
- Use the "Instruction -> Code -> Explanation -> Verification" loop for every step.

5. ANALYZE & REPLICATE STRUCTURE:
- Use the [Reference Codelab] as a template for tone, formatting, and flow (e.g., Summary, Duration, Step numbering, "What you'll learn" sections).
- Maintain the "Introduction -> Setup -> Step-by-Step implementation -> Verification -> Conclusion" sequence.

6. MANDATORY ENVIRONMENT & IDE SETUP (Crucial):
- Create a dedicated "Environment Setup" section even if the source code doesn't explicitly mention it.
- IDE Recommendations: Suggest the best IDE for the project (e.g., VS Code, IntelliJ).
- Required Plugins: List specific extensions/plugins that will help the learner (e.g., "Install the 'ESLint' and 'Prettier' extensions in VS Code for code quality").
- System Config: Include OS-specific requirements, Node.js/Python versions, and Environment Variables (.env setup).

7. STEP-BY-STEP CONTENT GENERATION:
- Each step must follow this loop:
    a. Step Title & Estimated Duration.
    b. Concept: Why are we doing this? (The logic).
    c. Action: Clear instructions on which file to open/create.
    d. Code Block: Provide the exact code with comments like "<!-- CODELAB: Add this here -->".
    e. Deep Dive/Detour: Explain specific APIs or DevTools features used in this step (referencing the "DevTools Detour" style in the example).

8. VERIFICATION & AUDIT:
- Include a "Verify your changes" or "Audit" section for every major milestone.
- Tell the user exactly what to look for in the browser console, terminal, or UI to ensure they are on the right track.

9. FORMATTING:
- Use clear Markdown.
- Use callout boxes (Note, Caution, Tip) to highlight important information.
- Always specify the filename above the code blocks.
- After each code block, add a short list like "1) line -> explanation" with concise reasons (not just restating the code).

`;
    const PLAN_SYSTEM_PROMPT = `
You are a senior curriculum designer. Create a codelab plan from source code and context.
Return JSON that matches the schema exactly.
Priorities:
1) Learning objectives and target audience.
2) Prerequisites and environment setup.
3) Step outline with verification checkpoints.
4) Search terms for latest information (short English queries).
Keep the plan concise, practical, and aligned to the target duration.
`;
    const REVIEW_SYSTEM_PROMPT = `
You are a third-party facilitator and technical reviewer.
Review the draft codelab against the plan and source context.
Be critical and specific. Provide actionable improvements.
Return JSON that matches the schema exactly.
`;

    let generationMode = $state<GenerationMode>("basic");
    let advancedStep = $state<AdvancedStep>("input");
    let advancedLoading = $state(false);
    let advancedStreamContent = $state("");
    let advancedThinkingContent = $state("");
    let advancedPlanData = $state<PlanData | null>(null);
    let advancedDraftData = $state<CodelabDraft | null>(null);
    let advancedReviewData = $state<ReviewData | null>(null);
    let advancedRevisedData = $state<CodelabDraft | null>(null);
    let advancedDraftView = $state<"markdown" | "raw">("markdown");
    let advancedRevisedView = $state<"markdown" | "raw">("markdown");
    let advancedDiffView = $state<"unified" | "split">("unified");
    let advancedUseGoogleSearch = $state(true);
    let advancedUseUrlContext = $state(false);
    let advancedSourceContext = $state("");
    let advancedTargetLanguage = $state("English");
    let planComments = $state<PlanComment[]>([]);
    let planSelection = $state<{
        text: string;
        top: number;
        left: number;
    } | null>(null);
    let planCommentDraft = $state("");
    let planContainerRef = $state<HTMLDivElement | null>(null);
    let planCommentInputRef = $state<HTMLTextAreaElement | null>(null);

    import {
        getBlocklists,
        isEnvFile,
        isMediaFile,
        isBlockedByPath,
        isBlockedByExt,
        shouldSkipFile,
    } from "$lib/uploadFilters";

    async function handleFileUpload(event: Event) {
        const target = event.target as HTMLInputElement;
        if (!target.files) return;

        const pendingFiles: { name: string; content: string }[] = [];
        const remainingSlots = () => MAX_FILES - (uploadedFiles.length + pendingFiles.length);

        if (remainingSlots() <= 0) {
            alert("You can upload up to 10 files per prompt.");
            return;
        }

        loading = true;
        try {
            for (const file of Array.from(target.files)) {
                if (remainingSlots() <= 0) {
                    alert("You can upload up to 10 files per prompt.");
                    break;
                }

                if (file.size > MAX_FILE_SIZE_BYTES) {
                    alert(`${file.name} exceeds the 1GB file size limit.`);
                    continue;
                }

                if (shouldSkipFile(file.name)) {
                    alert(`${file.name} is skipped (env/media/build/binary artifact).`);
                    continue;
                }

                if (file.name.endsWith(".zip")) {
                    const { added, truncated } = await extractCodeFromZip(
                        file,
                        remainingSlots(),
                    );
                    pendingFiles.push(...added);
                    if (truncated) {
                        alert(
                            `ZIP upload limit reached (max ${MAX_ZIP_FILES} files per zip and ${MAX_FILES} per prompt). Extra files were skipped.`,
                        );
                    }
                } else {
                    const content = await file.text();
                    pendingFiles.push({ name: file.name, content });
                }
            }

            if (pendingFiles.length > 0) {
                uploadedFiles = [
                    ...uploadedFiles,
                    ...pendingFiles,
                ].slice(0, MAX_FILES);
            }
        } catch (e) {
            console.error("File upload failed", e);
            alert($t("ai_generator.error_upload") || "Failed to upload files");
        } finally {
            loading = false;
            target.value = "";
        }
    }

    async function extractCodeFromZip(
        file: File,
        allowedCount: number,
    ): Promise<{ added: { name: string; content: string }[]; truncated: boolean }> {
        const zip = new JSZip();
        const content = await zip.loadAsync(file);
        const added: { name: string; content: string }[] = [];
        let truncated = false;

        for (const [path, zipEntry] of Object.entries(content.files)) {
            if (zipEntry.dir) continue;
            const lowerPath = path.toLowerCase();

            // Filter out binary and ignored files
            const isIgnored = shouldSkipFile(lowerPath);
            if (isIgnored) continue;

            if (added.length >= Math.min(MAX_ZIP_FILES, allowedCount)) {
                truncated = true;
                break;
            }

            if (!isIgnored) {
                const text = await zipEntry.async("text");
                added.push({ name: path, content: text });
            }
        }

        return { added, truncated };
    }

    function removeFile(index: number) {
        uploadedFiles = uploadedFiles.filter((_, i) => i !== index);
    }

    function resolveTargetLanguage() {
        const userLanguage = $locale || "en";
        const languageNames: Record<string, string> = {
            ko: "Korean",
            en: "English",
            zh: "Chinese",
            ja: "Japanese",
        };
        return languageNames[userLanguage] || "English";
    }

    function buildDurationText() {
        const durationValue =
            handsOnDuration === "custom" ? customDuration : handsOnDuration;
        return durationValue
            ? `The target duration for this hands-on session is approximately ${durationValue} minutes. Please adjust the depth and number of steps to fit this timeframe.`
            : "";
    }

    function parseStructuredJson<T>(raw: string): T | null {
        const trimmed = raw.trim();
        if (!trimmed) return null;
        const firstBrace = trimmed.indexOf("{");
        const lastBrace = trimmed.lastIndexOf("}");
        if (firstBrace === -1 || lastBrace === -1) return null;
        const jsonText = trimmed.substring(firstBrace, lastBrace + 1);
        try {
            return JSON.parse(jsonText) as T;
        } catch {
            return null;
        }
    }

    function buildCodelabSchema(targetLanguage: string) {
        return {
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
                                description: `Markdown content in ${targetLanguage} for this step. Explain the code clearly. Use code blocks with inline comments and add a numbered line-by-line explanation list after each block.`,
                            },
                        },
                        required: ["title", "content"],
                    },
                },
            },
            required: ["title", "description", "steps"],
        };
    }

    function buildCodelabMarkdown(data: CodelabDraft | null) {
        if (!data) return "";
        const sections: string[] = [];
        if (data.title) {
            sections.push(`# ${data.title}`);
        }
        if (data.description) {
            sections.push("", data.description);
        }
        data.steps.forEach((step, index) => {
            const heading = `## ${index + 1}. ${step.title}`;
            sections.push("", heading, "", step.content || "");
        });
        return sections.join("\n").trim();
    }

    function renderMarkdown(markdown: string) {
        if (!markdown) return "";
        try {
            const html = marked.parse(markdown) as string;
            if (browser) {
                return DOMPurify.sanitize(html);
            }
            return html;
        } catch (e) {
            console.error("Markdown parse error", e);
            return $t("ai_generator.error_parse");
        }
    }

    type DiffLine = {
        type: "equal" | "add" | "remove";
        text: string;
    };

    type DiffRow = {
        leftText: string;
        rightText: string;
        leftType: "equal" | "remove" | "empty";
        rightType: "equal" | "add" | "empty";
    };

    const MAX_DIFF_CELLS = 2_000_000;

    function buildLineDiff(
        sourceText: string,
        targetText: string,
    ): { lines: DiffLine[]; truncated: boolean } {
        const source = sourceText ? sourceText.split("\n") : [];
        const target = targetText ? targetText.split("\n") : [];
        const rows = source.length;
        const cols = target.length;

        if (!rows && !cols) {
            return { lines: [], truncated: false };
        }

        if (rows * cols > MAX_DIFF_CELLS) {
            return { lines: [], truncated: true };
        }

        const width = cols + 1;
        const dp = new Uint32Array((rows + 1) * width);

        for (let i = 1; i <= rows; i += 1) {
            for (let j = 1; j <= cols; j += 1) {
                const idx = i * width + j;
                if (source[i - 1] === target[j - 1]) {
                    dp[idx] = dp[(i - 1) * width + (j - 1)] + 1;
                } else {
                    const top = dp[(i - 1) * width + j];
                    const left = dp[i * width + (j - 1)];
                    dp[idx] = top > left ? top : left;
                }
            }
        }

        const lines: DiffLine[] = [];
        let i = rows;
        let j = cols;
        while (i > 0 && j > 0) {
            if (source[i - 1] === target[j - 1]) {
                lines.push({ type: "equal", text: source[i - 1] });
                i -= 1;
                j -= 1;
            } else {
                const top = dp[(i - 1) * width + j];
                const left = dp[i * width + (j - 1)];
                if (top >= left) {
                    lines.push({ type: "remove", text: source[i - 1] });
                    i -= 1;
                } else {
                    lines.push({ type: "add", text: target[j - 1] });
                    j -= 1;
                }
            }
        }
        while (i > 0) {
            lines.push({ type: "remove", text: source[i - 1] });
            i -= 1;
        }
        while (j > 0) {
            lines.push({ type: "add", text: target[j - 1] });
            j -= 1;
        }

        lines.reverse();
        return { lines, truncated: false };
    }

    let advancedDraftMarkdown = $derived.by(() =>
        buildCodelabMarkdown(advancedDraftData),
    );
    let advancedRevisedMarkdown = $derived.by(() =>
        buildCodelabMarkdown(advancedRevisedData),
    );
    let advancedDraftHtml = $derived.by(() =>
        renderMarkdown(advancedDraftMarkdown),
    );
    let advancedRevisedHtml = $derived.by(() =>
        renderMarkdown(advancedRevisedMarkdown),
    );
    let advancedDiff = $derived.by(() =>
        buildLineDiff(advancedDraftMarkdown, advancedRevisedMarkdown),
    );
    let advancedDiffRows = $derived.by(() => {
        if (advancedDiff.truncated) return [];
        return advancedDiff.lines.map<DiffRow>((line) => {
            if (line.type === "equal") {
                return {
                    leftText: line.text,
                    rightText: line.text,
                    leftType: "equal",
                    rightType: "equal",
                };
            }
            if (line.type === "remove") {
                return {
                    leftText: line.text,
                    rightText: "",
                    leftType: "remove",
                    rightType: "empty",
                };
            }
            return {
                leftText: "",
                rightText: line.text,
                leftType: "empty",
                rightType: "add",
            };
        });
    });

    async function handleGenerate() {
        // Combine manually entered code and uploaded files
        let fullContext = sourceCode.trim();
        if (uploadedFiles.length > 0) {
            const filesContext = uploadedFiles
                .map((f) => `File: ${f.name}\n\`\`\`\n${f.content}\n\`\`\``)
                .join("\n\n");
            fullContext = fullContext
                ? `${fullContext}\n\nUploaded Files:\n${filesContext}`
                : filesContext;
        }

        if (!fullContext || !apiKey) return;

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
                                description: `Markdown content in ${targetLanguage} for this step. Explain the code clearly. Use code blocks with inline comments and add a numbered line-by-line explanation list after each block.`,
                            },
                        },
                        required: ["title", "content"],
                    },
                },
            },
            required: ["title", "description", "steps"],
        };

        const durationValue =
            handsOnDuration === "custom" ? customDuration : handsOnDuration;
        const durationText = durationValue
            ? `The target duration for this hands-on session is approximately ${durationValue} minutes. Please adjust the depth and number of steps to fit this timeframe.`
            : "";

        const prompt = `Create a codelab tutorial from the following source code and context. ${durationText} Write ALL content in ${targetLanguage}. For every code block, include inline comments on each logical line, specify the filename before the block, and append a numbered line-by-line explanation list immediately after the block (same language).\n\nSource code/Context:\n${fullContext}`;

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
                // Find first '{' and last '}' to extract JSON
                let cleanContent = generatedContent.trim();
                const firstBrace = cleanContent.indexOf("{");
                const lastBrace = cleanContent.lastIndexOf("}");

                if (firstBrace !== -1 && lastBrace !== -1) {
                    cleanContent = cleanContent.substring(
                        firstBrace,
                        lastBrace + 1,
                    );
                }

                parsedData = JSON.parse(cleanContent);
                generationStep = "review";
            } catch (e) {
                console.error("JSON Parse Error:", e);
                console.error("Raw Response:", generatedContent);
                alert($t("ai_generator.error_parse"));
                generationStep = "input";
            }
        } catch (e: any) {
            console.error("Generation failed", e);
            alert($t("ai_generator.error_generate") + ": " + e.message);
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
                author: $t("common.ai_assistant"),
            });

            // 2. Save Steps
            const stepsPayload = parsedData.steps.map((s) => ({
                title: s.title,
                content_markdown: s.content,
            }));
            await saveSteps(codelab.id, stepsPayload);

            // 3. Create Code Server if enabled
            if (enableCodeServer && uploadedFiles.length > 0) {
                try {
                    const { createCodeServer, createCodeServerBranch } = await import('$lib/api');
                    const workspaceFiles = uploadedFiles.map(f => ({
                        path: f.name,
                        content: f.content
                    }));

                    await createCodeServer(codelab.id, workspaceFiles);

                    // Create branches for each step
                    for (let i = 0; i < parsedData.steps.length; i++) {
                        await createCodeServerBranch(codelab.id, i + 1, 'start');
                        await createCodeServerBranch(codelab.id, i + 1, 'end');
                    }
                } catch (e) {
                    console.error('Failed to create code server', e);
                    // Don't fail the whole process if code server creation fails
                }
            }

            onCodelabCreated(codelab);
        } catch (e) {
            console.error("Failed to save codelab", e);
            alert($t("ai_generator.error_save"));
        } finally {
            loading = false;
        }
    }

    function setGenerationMode(mode: GenerationMode) {
        generationMode = mode;
    }

    function clearPlanSelection() {
        planSelection = null;
        planCommentDraft = "";
    }

    function handlePlanSelection() {
        if (!planContainerRef) return;

        const selection = window.getSelection();
        if (!selection || selection.isCollapsed) {
            clearPlanSelection();
            return;
        }

        if (!selection.rangeCount) {
            clearPlanSelection();
            return;
        }

        const anchorNode = selection.anchorNode;
        const focusNode = selection.focusNode;
        if (!anchorNode || !focusNode) {
            clearPlanSelection();
            return;
        }

        if (
            !planContainerRef.contains(anchorNode) ||
            !planContainerRef.contains(focusNode)
        ) {
            clearPlanSelection();
            return;
        }

        const text = selection.toString().trim();
        if (!text) {
            clearPlanSelection();
            return;
        }

        const range = selection.getRangeAt(0);
        const rect = range.getBoundingClientRect();
        if (!rect || (!rect.width && !rect.height)) {
            clearPlanSelection();
            return;
        }

        const padding = 12;
        const popupWidth = 320;
        const popupHeight = 220;
        const maxLeft = Math.max(
            padding,
            window.innerWidth - popupWidth - padding,
        );
        const maxTop = Math.max(
            padding,
            window.innerHeight - popupHeight - padding,
        );
        const left = Math.min(Math.max(rect.left, padding), maxLeft);
        const top = Math.min(rect.bottom + 8, maxTop);

        planSelection = { text, top, left };
        planCommentDraft = "";
        setTimeout(() => planCommentInputRef?.focus(), 0);
    }

    function addPlanComment() {
        if (!planSelection) return;
        const commentText = planCommentDraft.trim();
        if (!commentText) return;

        const id = `${Date.now()}-${Math.random().toString(16).slice(2)}`;
        planComments = [
            ...planComments,
            {
                id,
                quote: planSelection.text,
                comment: commentText,
            },
        ];
        window.getSelection()?.removeAllRanges();
        clearPlanSelection();
    }

    function removePlanComment(id: string) {
        planComments = planComments.filter((comment) => comment.id !== id);
    }

    async function handleAdvancedPlan() {
        let fullContext = sourceCode.trim();
        if (uploadedFiles.length > 0) {
            const filesContext = uploadedFiles
                .map((f) => `File: ${f.name}\n\`\`\`\n${f.content}\n\`\`\``)
                .join("\n\n");
            fullContext = fullContext
                ? `${fullContext}\n\nUploaded Files:\n${filesContext}`
                : filesContext;
        }

        if (!fullContext || !apiKey) return;

        advancedLoading = true;
        advancedStep = "planning";
        advancedStreamContent = "";
        advancedThinkingContent = "";
        advancedPlanData = null;
        advancedDraftData = null;
        advancedReviewData = null;
        advancedRevisedData = null;
        advancedDraftView = "markdown";
        advancedRevisedView = "markdown";
        advancedDiffView = "unified";
        planComments = [];
        clearPlanSelection();
        advancedSourceContext = fullContext;
        advancedTargetLanguage = resolveTargetLanguage();

        const planSchema = {
            type: "object",
            properties: {
                title: {
                    type: "string",
                    description: `Plan title in ${advancedTargetLanguage}`,
                },
                description: {
                    type: "string",
                    description: `Plan summary in ${advancedTargetLanguage}`,
                },
                audience: {
                    type: "string",
                    description: `Target audience in ${advancedTargetLanguage}`,
                },
                learning_objectives: {
                    type: "array",
                    items: {
                        type: "string",
                        description: `Learning objective in ${advancedTargetLanguage}`,
                    },
                },
                prerequisites: {
                    type: "array",
                    items: {
                        type: "string",
                        description: `Prerequisite in ${advancedTargetLanguage}`,
                    },
                },
                environment_setup: {
                    type: "object",
                    properties: {
                        os_requirements: {
                            type: "array",
                            items: {
                                type: "string",
                                description: `OS requirement in ${advancedTargetLanguage}`,
                            },
                        },
                        tools: {
                            type: "array",
                            items: {
                                type: "string",
                                description: `Required tool in ${advancedTargetLanguage}`,
                            },
                        },
                        env_vars: {
                            type: "array",
                            items: {
                                type: "string",
                                description: `Environment variable in ${advancedTargetLanguage}`,
                            },
                        },
                        ide: {
                            type: "string",
                            description: `Recommended IDE in ${advancedTargetLanguage}`,
                        },
                        ide_plugins: {
                            type: "array",
                            items: {
                                type: "string",
                                description: `IDE plugin in ${advancedTargetLanguage}`,
                            },
                        },
                    },
                    required: [
                        "os_requirements",
                        "tools",
                        "env_vars",
                        "ide",
                        "ide_plugins",
                    ],
                },
                steps: {
                    type: "array",
                    items: {
                        type: "object",
                        properties: {
                            title: {
                                type: "string",
                                description: `Step title in ${advancedTargetLanguage}`,
                            },
                            goal: {
                                type: "string",
                                description: `Step goal in ${advancedTargetLanguage}`,
                            },
                            files: {
                                type: "array",
                                items: {
                                    type: "string",
                                    description:
                                        "File paths touched in this step",
                                },
                            },
                            verification: {
                                type: "string",
                                description: `Verification checklist in ${advancedTargetLanguage}`,
                            },
                        },
                        required: ["title", "goal", "files", "verification"],
                    },
                },
                search_terms: {
                    type: "array",
                    items: {
                        type: "string",
                        description:
                            "Short English search query for the latest info",
                    },
                },
            },
            required: [
                "title",
                "description",
                "audience",
                "learning_objectives",
                "prerequisites",
                "environment_setup",
                "steps",
                "search_terms",
            ],
        };

        const durationText = buildDurationText();
        const planPrompt = `Design a codelab plan from the following source code and context. ${durationText} Write all content in ${advancedTargetLanguage}. For "search_terms", use short English queries to find the latest versions, commands, or best practices (3-8 items). Keep step count aligned with the target duration. If something is unknown, return empty arrays.\n\nSource code/Context:\n${fullContext}`;

        try {
            const stream = streamGeminiStructuredOutput(
                planPrompt,
                PLAN_SYSTEM_PROMPT,
                planSchema,
                {
                    apiKey,
                    model: "gemini-3-flash-preview",
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );

            for await (const chunk of stream) {
                if (chunk.thinking) {
                    advancedThinkingContent += chunk.thinking;
                }
                if (chunk.content) {
                    advancedStreamContent += chunk.content;
                }
            }

            const parsed = parseStructuredJson<PlanData>(advancedStreamContent);
            if (!parsed) {
                alert($t("ai_generator.error_parse"));
                advancedStep = "input";
                return;
            }

            advancedPlanData = parsed;
            advancedStep = "plan";
        } catch (e: any) {
            console.error("Plan generation failed", e);
            alert($t("ai_generator.error_generate") + ": " + e.message);
            advancedStep = "input";
        } finally {
            advancedLoading = false;
        }
    }

    async function handleAdvancedDraft() {
        if (!advancedPlanData || !apiKey) return;

        advancedLoading = true;
        advancedStep = "drafting";
        advancedStreamContent = "";
        advancedThinkingContent = "";
        advancedDraftData = null;
        clearPlanSelection();

        const durationText = buildDurationText();
        const searchTerms = advancedPlanData.search_terms || [];
        const searchHint = searchTerms.length
            ? `Use the Google Search tool to verify the latest information for these queries: ${searchTerms.join(
                  ", ",
              )}.`
            : "Use the Google Search tool if any versions, commands, or APIs need verification.";

        const facilitatorComments = planComments.length
            ? `Facilitator comments (address these in the draft):\n${JSON.stringify(
                  planComments.map((comment) => ({
                      selection: comment.quote,
                      comment: comment.comment,
                  })),
                  null,
                  2,
              )}\n\n`
            : "";
        const draftPrompt = `Create a codelab using the plan and source context. ${durationText} Write ALL content in ${advancedTargetLanguage}. ${searchHint}\n\nPlan JSON:\n${JSON.stringify(
            advancedPlanData,
            null,
            2,
        )}\n\n${facilitatorComments}Source code/Context:\n${advancedSourceContext}`;

        const tools: GeminiStructuredConfig["tools"] = [];
        if (advancedUseGoogleSearch) {
            tools.push({ googleSearch: {} });
        }
        if (advancedUseUrlContext) {
            tools.push({ urlContext: {} });
        }

        try {
            const stream = streamGeminiStructuredOutput(
                draftPrompt,
                SYSTEM_PROMPT,
                buildCodelabSchema(advancedTargetLanguage),
                {
                    apiKey,
                    model: "gemini-3-flash-preview",
                    tools: tools.length > 0 ? tools : undefined,
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );

            for await (const chunk of stream) {
                if (chunk.thinking) {
                    advancedThinkingContent += chunk.thinking;
                }
                if (chunk.content) {
                    advancedStreamContent += chunk.content;
                }
            }

            const parsed = parseStructuredJson<CodelabDraft>(
                advancedStreamContent,
            );
            if (!parsed) {
                alert($t("ai_generator.error_parse"));
                advancedStep = "plan";
                return;
            }

            advancedDraftData = parsed;
            advancedStep = "draft";
        } catch (e: any) {
            console.error("Draft generation failed", e);
            alert($t("ai_generator.error_generate") + ": " + e.message);
            advancedStep = "plan";
        } finally {
            advancedLoading = false;
        }
    }

    async function handleAdvancedReviewAndRevise() {
        if (!advancedPlanData || !advancedDraftData || !apiKey) return;

        advancedLoading = true;
        advancedStep = "reviewing";
        advancedStreamContent = "";
        advancedThinkingContent = "";
        advancedReviewData = null;
        advancedRevisedData = null;

        const reviewSchema = {
            type: "object",
            properties: {
                summary: {
                    type: "string",
                    description: `Review summary in ${advancedTargetLanguage}`,
                },
                issues: {
                    type: "array",
                    items: {
                        type: "object",
                        properties: {
                            severity: {
                                type: "string",
                                description: `Severity in ${advancedTargetLanguage}`,
                            },
                            issue: {
                                type: "string",
                                description: `Issue description in ${advancedTargetLanguage}`,
                            },
                            recommendation: {
                                type: "string",
                                description: `Fix recommendation in ${advancedTargetLanguage}`,
                            },
                        },
                        required: ["severity", "issue", "recommendation"],
                    },
                },
                missing_items: {
                    type: "array",
                    items: {
                        type: "string",
                        description: `Missing item in ${advancedTargetLanguage}`,
                    },
                },
                improvements: {
                    type: "array",
                    items: {
                        type: "string",
                        description: `Improvement suggestion in ${advancedTargetLanguage}`,
                    },
                },
            },
            required: ["summary", "issues", "missing_items", "improvements"],
        };

        const reviewPrompt = `Review the draft codelab as a third-party facilitator expert. Use the plan to verify structure and completeness. Write ALL content in ${advancedTargetLanguage}.\n\nPlan JSON:\n${JSON.stringify(
            advancedPlanData,
            null,
            2,
        )}\n\nDraft JSON:\n${JSON.stringify(
            advancedDraftData,
            null,
            2,
        )}\n\nSource code/Context:\n${advancedSourceContext}`;

        try {
            const reviewStream = streamGeminiStructuredOutput(
                reviewPrompt,
                REVIEW_SYSTEM_PROMPT,
                reviewSchema,
                {
                    apiKey,
                    model: "gemini-3-flash-preview",
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );

            for await (const chunk of reviewStream) {
                if (chunk.thinking) {
                    advancedThinkingContent += chunk.thinking;
                }
                if (chunk.content) {
                    advancedStreamContent += chunk.content;
                }
            }

            const parsedReview = parseStructuredJson<ReviewData>(
                advancedStreamContent,
            );
            if (!parsedReview) {
                alert($t("ai_generator.error_parse"));
                advancedStep = "draft";
                advancedLoading = false;
                return;
            }

            advancedReviewData = parsedReview;
        } catch (e: any) {
            console.error("Review failed", e);
            alert($t("ai_generator.error_generate") + ": " + e.message);
            advancedStep = "draft";
            advancedLoading = false;
            return;
        }

        advancedStep = "revising";
        advancedStreamContent = "";
        advancedThinkingContent = "";

        const durationText = buildDurationText();
        const searchTerms = advancedPlanData.search_terms || [];
        const searchHint = searchTerms.length
            ? `Use the Google Search tool to verify the latest information for these queries: ${searchTerms.join(
                  ", ",
              )}.`
            : "Use the Google Search tool if any versions, commands, or APIs need verification.";

        const revisePrompt = `Revise the draft codelab based on the expert review. ${durationText} Write ALL content in ${advancedTargetLanguage}. ${searchHint}\n\nPlan JSON:\n${JSON.stringify(
            advancedPlanData,
            null,
            2,
        )}\n\nDraft JSON:\n${JSON.stringify(
            advancedDraftData,
            null,
            2,
        )}\n\nReview JSON:\n${JSON.stringify(
            advancedReviewData,
            null,
            2,
        )}\n\nSource code/Context:\n${advancedSourceContext}`;

        const tools: GeminiStructuredConfig["tools"] = [];
        if (advancedUseGoogleSearch) {
            tools.push({ googleSearch: {} });
        }
        if (advancedUseUrlContext) {
            tools.push({ urlContext: {} });
        }

        try {
            const reviseStream = streamGeminiStructuredOutput(
                revisePrompt,
                SYSTEM_PROMPT,
                buildCodelabSchema(advancedTargetLanguage),
                {
                    apiKey,
                    model: "gemini-3-flash-preview",
                    tools: tools.length > 0 ? tools : undefined,
                    thinkingConfig: { thinkingLevel: "high" },
                },
            );

            for await (const chunk of reviseStream) {
                if (chunk.thinking) {
                    advancedThinkingContent += chunk.thinking;
                }
                if (chunk.content) {
                    advancedStreamContent += chunk.content;
                }
            }

            const parsed = parseStructuredJson<CodelabDraft>(
                advancedStreamContent,
            );
            if (!parsed) {
                alert($t("ai_generator.error_parse"));
                advancedStep = "draft";
                return;
            }

            advancedRevisedData = parsed;
            advancedStep = "final";
        } catch (e: any) {
            console.error("Revision failed", e);
            alert($t("ai_generator.error_generate") + ": " + e.message);
            advancedStep = "draft";
        } finally {
            advancedLoading = false;
        }
    }

    async function handleSaveAdvanced() {
        if (!advancedRevisedData) return;
        advancedLoading = true;
        try {
            const codelab = await createCodelab({
                title: advancedRevisedData.title,
                description: advancedRevisedData.description,
                author: $t("common.ai_assistant"),
            });

            const stepsPayload = advancedRevisedData.steps.map((s) => ({
                title: s.title,
                content_markdown: s.content,
            }));
            await saveSteps(codelab.id, stepsPayload);

            onCodelabCreated(codelab);
        } catch (e) {
            console.error("Failed to save codelab", e);
            alert($t("ai_generator.error_save"));
        } finally {
            advancedLoading = false;
        }
    }
</script>

<div
    class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
>
    <div
        class="bg-white dark:bg-dark-surface rounded-3xl shadow-2xl w-full max-w-4xl h-[85vh] flex flex-col overflow-hidden relative border dark:border-dark-border"
        in:fly={{ y: 20, duration: 400 }}
    >
        <!-- Header -->
        <div class="bg-[#4285F4] p-6 text-white shrink-0">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <div class="bg-white/20 p-2 rounded-lg" aria-hidden="true">
                        <Sparkles size={24} />
                    </div>
                    <div>
                        <h2 id="modal-title" class="text-2xl font-bold">
                            {$t("ai_generator.title")}
                        </h2>
                        <p class="opacity-80 text-sm">
                            {$t("ai_generator.subtitle")}
                        </p>
                    </div>
                </div>
                <button
                    onclick={onClose}
                    class="p-2 hover:bg-white/10 rounded-full transition-colors"
                    aria-label={$t("common.close") || "Close"}
                >
                    <X size={24} />
                </button>
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 flex flex-col overflow-hidden p-6 bg-[#F8F9FA] dark:bg-dark-bg">
            <div
                class="mb-4 bg-white dark:bg-dark-surface/50 border border-[#DADCE0] dark:border-dark-border rounded-2xl p-3 shadow-sm"
            >
                <div class="flex flex-wrap items-center justify-between gap-3">
                    <span
                        class="text-sm font-bold text-[#5F6368] dark:text-dark-text-muted"
                    >
                        {$t("ai_generator.mode_label")}
                    </span>
                    <div class="flex items-center gap-2">
                        <button
                            onclick={() => setGenerationMode("basic")}
                            disabled={loading || advancedLoading}
                            class="px-4 py-2 rounded-xl text-xs font-bold transition-all border {generationMode ===
                            'basic'
                                ? 'bg-[#4285F4] text-white border-[#4285F4] shadow-md'
                                : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border hover:border-[#4285F4]'}"
                        >
                            {$t("ai_generator.mode_basic")}
                        </button>
                        <button
                            onclick={() => setGenerationMode("advanced")}
                            disabled={loading || advancedLoading}
                            class="px-4 py-2 rounded-xl text-xs font-bold transition-all border {generationMode ===
                            'advanced'
                                ? 'bg-[#4285F4] text-white border-[#4285F4] shadow-md'
                                : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border hover:border-[#4285F4]'}"
                        >
                            {$t("ai_generator.mode_advanced")}
                        </button>
                    </div>
                </div>
                <div class="mt-3 space-y-3">
                    <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                        {generationMode === "basic"
                            ? $t("ai_generator.mode_basic_desc")
                            : $t("ai_generator.mode_advanced_desc")}
                    </p>
                    {#if generationMode === "advanced"}
                        <div
                            class="flex flex-col gap-3 rounded-2xl border border-[#DADCE0] dark:border-dark-border bg-white dark:bg-dark-surface/50 p-4 shadow-sm"
                        >
                            <p class="text-sm text-[#3C4043] dark:text-dark-text">
                                {$t("ai_generator.mode_advanced_star_message")}
                            </p>
                            <a
                                class="inline-flex items-center justify-center gap-2 rounded-xl bg-[#202124] text-white px-4 py-2 text-xs font-bold hover:bg-black transition-colors"
                                href="https://github.com/JAICHANGPARK/open-codelabs"
                                target="_blank"
                                rel="noreferrer"
                            >
                                {$t("ai_generator.mode_advanced_star_button")}
                            </a>
                        </div>
                    {/if}
                </div>
            </div>

            <div class="flex-1 min-h-0 overflow-y-auto">
            {#if generationMode === "basic"}
                {#if generationStep === "input"}
                <div class="min-h-full flex flex-col gap-4" in:fade>
                    <div class="flex items-center justify-between">
                        <label
                            for="source-code"
                            class="text-[#5F6368] dark:text-dark-text-muted font-bold text-lg"
                            >{$t("ai_generator.input_label")}</label
                        >

                        <div class="flex items-center gap-2">
                            <input
                                type="file"
                                multiple
                                accept=".zip,.js,.ts,.py,.rs,.go,.mod,.sum,.kt,.kts,.java,.c,.cpp,.h,.html,.css,.xml,.gradle,.json,.yml,.yaml,.toml,.proto,.md,.ipynb,.dart,.lock"
                                bind:this={fileInput}
                                onchange={handleFileUpload}
                                class="hidden"
                            />
                            <button
                                onclick={() => fileInput.click()}
                                class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-dark-surface border border-[#DADCE0] dark:border-dark-border rounded-xl text-sm font-bold text-[#4285F4] hover:bg-[#4285F4]/5 transition-all shadow-sm"
                            >
                                <Upload size={18} />
                                {$t("ai_generator.upload_files") ||
                                    "Upload Files / Zip"}
                            </button>
                        </div>
                    </div>
                    <p class="text-xs text-[#5F6368] dark:text-dark-text-muted">
                        Limit: up to 10 files per prompt (zip may contain max 10 files), max 1GB each, no audio/video files.
                    </p>

                    <!-- Uploaded Files List -->
                    <UploadedFileList files={uploadedFiles} onRemove={removeFile} />

                    <!-- Advanced Options -->
                    <div
                        class="flex flex-col gap-4 mb-4 bg-white dark:bg-dark-surface/50 p-4 rounded-2xl border border-[#DADCE0] dark:border-dark-border shadow-sm"
                    >
                        <!-- Duration Selection -->
                        <div class="flex flex-col gap-3">
                            <span
                                class="text-sm font-bold text-[#5F6368] dark:text-dark-text-muted flex items-center gap-2"
                            >
                                <Clock size={16} class="text-[#4285F4]" />
                                {$t("ai_generator.duration_label")}
                            </span>
                            <div class="flex flex-wrap gap-2">
                                {#each ["60", "90", "120", "150", "180", "custom"] as d}
                                    <button
                                        onclick={() => (handsOnDuration = d)}
                                        class="px-4 py-2 rounded-xl text-xs font-bold transition-all border {handsOnDuration ===
                                        d
                                            ? 'bg-[#4285F4] text-white border-[#4285F4] shadow-md'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border hover:border-[#4285F4]'}"
                                    >
                                        {d === "custom"
                                            ? $t("ai_generator.duration_custom")
                                            : $t("ai_generator.duration_mins", {
                                                  values: { mins: d },
                                              })}
                                    </button>
                                {/each}
                                {#if handsOnDuration === "custom"}
                                    <div
                                        class="flex items-center gap-2 ml-2"
                                        in:fade
                                    >
                                        <input
                                            type="number"
                                            bind:value={customDuration}
                                            placeholder="10"
                                            class="w-20 bg-white dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-lg px-3 py-2 text-xs outline-none focus:border-[#4285F4] focus:ring-2 focus:ring-[#4285F4]/10"
                                        />
                                        <span
                                            class="text-xs font-medium text-[#5F6368] dark:text-dark-text-muted"
                                            >{$t("ai_generator.mins")}</span
                                        >
                                    </div>
                                {/if}
                            </div>
                        </div>

                        <div
                            class="h-px bg-[#F1F3F4] dark:bg-dark-border w-full"
                        ></div>

                        <div class="flex flex-wrap gap-6">
                            <label
                                class="flex items-center gap-2 cursor-pointer group"
                            >
                                <input
                                    type="checkbox"
                                    bind:checked={useGoogleSearch}
                                    class="w-5 h-5 rounded border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-[#4285F4] focus:ring-[#4285F4]"
                                />
                                <span
                                    class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4]"
                                >
                                    {$t("ai_generator.google_search")}
                                </span>
                            </label>

                            <label
                                class="flex items-center gap-2 cursor-pointer group"
                            >
                                <input
                                    type="checkbox"
                                    bind:checked={useUrlContext}
                                    class="w-5 h-5 rounded border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-[#4285F4] focus:ring-[#4285F4]"
                                />
                                <span
                                    class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4]"
                                >
                                    {$t("ai_generator.url_context")}
                                </span>
                            </label>

                            <label
                                class="flex items-center gap-2 cursor-pointer group"
                            >
                                <input
                                    type="checkbox"
                                    bind:checked={showThinking}
                                    class="w-5 h-5 rounded border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-[#4285F4] focus:ring-[#4285F4]"
                                />
                                <span
                                    class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4]"
                                >
                                    {$t("ai_generator.show_thinking")}
                                </span>
                            </label>
                        </div>
                    </div>

                    {#if useGoogleSearch || useUrlContext}
                        <div
                            class="flex items-start gap-2 p-3 bg-[#FEF7E0] dark:bg-[#F9AB00]/10 border border-[#F9AB00]/30 rounded-lg mb-4"
                        >
                            <Info
                                size={16}
                                class="text-[#F9AB00] mt-0.5 shrink-0"
                            />
                            <p
                                class="text-xs text-[#3C4043] dark:text-dark-text"
                            >
                                <strong
                                    >{$t("ai_generator.billing_notice")}</strong
                                >
                                {$t("ai_generator.billing_desc")}
                            </p>
                        </div>
                    {/if}

                    <textarea
                        id="source-code"
                        bind:value={sourceCode}
                        placeholder={$t("ai_generator.placeholder")}
                        class="flex-1 w-full bg-white dark:bg-dark-surface text-[#3C4043] dark:text-dark-text border border-[#DADCE0] dark:border-dark-border rounded-xl p-4 font-mono text-sm focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 outline-none resize-none shadow-sm transition-all"
                    ></textarea>

                    <div class="flex justify-end pt-2">
                        {#if !apiKey}
                            <p
                                class="text-[#EA4335] font-bold mr-4 self-center"
                            >
                                {$t("ai_generator.api_key_required")}
                            </p>
                            <button
                                disabled
                                class="bg-[#E8EAED] dark:bg-dark-border text-[#9AA0A6] dark:text-dark-text-muted px-8 py-3 rounded-full font-bold cursor-not-allowed"
                            >
                                {$t("common.create")}
                            </button>
                        {:else}
                            <button
                                onclick={handleGenerate}
                                disabled={!sourceCode.trim() &&
                                    uploadedFiles.length === 0}
                                class="bg-[#4285F4] text-white px-8 py-3 rounded-full font-bold hover:bg-[#1A73E8] hover:shadow-lg transition-colors text-lg flex items-center gap-2 disabled:opacity-50"
                            >
                                <Sparkles size={20} />
                                {$t("ai_generator.generate_button")}
                            </button>
                        {/if}
                    </div>
                </div>
            {:else if generationStep === "generating"}
                <div
                    class="min-h-full flex flex-col items-center justify-center gap-6"
                    in:fade
                    aria-live="polite"
                >
                    <div class="relative">
                        <div
                            class="absolute inset-0 bg-[#4285F4] rounded-full blur-xl opacity-20 animate-pulse"
                            aria-hidden="true"
                        ></div>
                        <Loader2
                            class="w-16 h-16 text-[#4285F4] animate-spin relative z-10"
                            aria-hidden="true"
                        />
                    </div>
                    <h3
                        class="text-xl font-bold text-[#3C4043] dark:text-dark-text"
                    >
                        {$t("ai_generator.analyzing")}
                    </h3>
                    <p
                        class="text-[#5F6368] dark:text-dark-text-muted text-center"
                    >
                        {@html $t("ai_generator.analyzing_desc").replace(
                            "\n",
                            "<br />",
                        )}
                    </p>

                    <!-- Thinking Display -->
                    {#if showThinking && thinkingContent}
                        <div class="w-full max-w-2xl mt-6">
                            <details
                                open
                                class="bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden"
                            >
                                <summary
                                    class="px-4 py-3 cursor-pointer hover:bg-[#F8F9FA] dark:hover:bg-white/5 flex items-center gap-2 font-medium text-[#5F6368] dark:text-dark-text-muted"
                                >
                                    <Sparkles
                                        size={16}
                                        class="text-[#4285F4]"
                                    />
                                    {$t("ai_generator.thinking_process")}
                                </summary>
                                <div
                                    class="px-4 py-3 text-xs text-[#5F6368] dark:text-dark-text-muted font-mono bg-[#F8F9FA] dark:bg-dark-bg/50 max-h-48 overflow-y-auto border-t border-[#E8EAED] dark:border-dark-border"
                                >
                                    {thinkingContent}
                                </div>
                            </details>
                        </div>
                    {/if}

                    <!-- Preview of raw stream just to show activity -->
                    <div
                        class="w-full max-w-2xl h-32 overflow-hidden text-xs text-[#9AA0A6] dark:text-dark-text-muted font-mono text-center opacity-50 relative mt-8"
                    >
                        <div
                            class="absolute inset-x-0 bottom-0 h-12 bg-gradient-to-t from-[#F8F9FA] dark:from-dark-bg to-transparent"
                        ></div>
                        {generatedContent.slice(-500)}
                    </div>
                </div>
            {:else if generationStep === "review" && parsedData}
                <div class="min-h-full flex flex-col gap-6" in:fade>
                    <div
                        class="flex items-center justify-between border-b border-[#E8EAED] dark:border-dark-border pb-4"
                    >
                        <div>
                            <h3
                                class="text-xl font-bold text-[#202124] dark:text-dark-text"
                            >
                                {$t("ai_generator.preview_title")}
                            </h3>
                            <p
                                class="text-[#5F6368] dark:text-dark-text-muted text-sm"
                            >
                                {$t("ai_generator.preview_subtitle")}
                            </p>
                        </div>
                        <div class="flex gap-3">
                            <button
                                onclick={() => (generationStep = "input")}
                                class="px-6 py-2 text-[#5F6368] dark:text-dark-text-muted font-bold hover:bg-[#E8EAED] dark:hover:bg-dark-border rounded-full transition-all"
                            >
                                {$t("ai_generator.back")}
                            </button>
                            <button
                                onclick={handleSave}
                                disabled={loading}
                                class="bg-[#34A853] text-white px-8 py-2 rounded-full font-bold hover:bg-[#1E8E3E] shadow-md transition-all flex items-center gap-2"
                            >
                                {#if loading}
                                    <Loader2 class="animate-spin" size={18} />
                                    {$t("ai_generator.saving")}
                                {:else}
                                    <ArrowRight size={18} />
                                    {$t("ai_generator.create_button")}
                                {/if}
                            </button>
                        </div>
                    </div>

                    <div
                        class="flex-1 overflow-y-auto bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-8 shadow-sm"
                    >
                        <h1
                            class="text-3xl font-bold text-[#202124] dark:text-dark-text mb-4"
                        >
                            {parsedData.title}
                        </h1>
                        <p
                            class="text-lg text-[#5F6368] dark:text-dark-text-muted mb-8"
                        >
                            {parsedData.description}
                        </p>

                        <div class="space-y-8">
                            {#each parsedData.steps as step, i}
                                <div
                                    class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6 hover:shadow-sm transition-shadow"
                                >
                                    <h4
                                        class="font-bold text-lg text-[#202124] dark:text-dark-text mb-2"
                                    >
                                        {i + 1}. {step.title}
                                    </h4>
                                    <div
                                        class="text-[#3C4043] dark:text-dark-text-muted text-sm line-clamp-3 opacity-80"
                                    >
                                        {step.content}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                </div>
            {/if}
        {:else}
            {#if advancedStep === "input"}
                <div class="min-h-full flex flex-col gap-4" in:fade>
                    <div class="flex items-center justify-between">
                        <label
                            for="source-code"
                            class="text-[#5F6368] dark:text-dark-text-muted font-bold text-lg"
                            >{$t("ai_generator.input_label")}</label
                        >

                        <div class="flex items-center gap-2">
                            <input
                                type="file"
                                multiple
                                accept=".zip,.js,.ts,.py,.rs,.go,.mod,.sum,.kt,.kts,.java,.c,.cpp,.h,.html,.css,.xml,.gradle,.json,.yml,.yaml,.toml,.proto,.md,.ipynb,.dart,.lock"
                                bind:this={fileInput}
                                onchange={handleFileUpload}
                                class="hidden"
                            />
                            <button
                                onclick={() => fileInput.click()}
                                class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-dark-surface border border-[#DADCE0] dark:border-dark-border rounded-xl text-sm font-bold text-[#4285F4] hover:bg-[#4285F4]/5 transition-all shadow-sm"
                            >
                                <Upload size={18} />
                                {$t("ai_generator.upload_files") ||
                                    "Upload Files / Zip"}
                            </button>
                        </div>
                    </div>
                    <p class="text-xs text-[#5F6368] dark:text-dark-text-muted">
                        Limit: up to 10 files per prompt (zip may contain max 10 files), max 1GB each, no audio/video files.
                    </p>

                    <!-- Uploaded Files List -->
                    <UploadedFileList
                        files={uploadedFiles}
                        onRemove={removeFile}
                    />

                    <!-- Advanced Options -->
                    <div
                        class="flex flex-col gap-4 mb-4 bg-white dark:bg-dark-surface/50 p-4 rounded-2xl border border-[#DADCE0] dark:border-dark-border shadow-sm"
                    >
                        <!-- Duration Selection -->
                        <div class="flex flex-col gap-3">
                            <span
                                class="text-sm font-bold text-[#5F6368] dark:text-dark-text-muted flex items-center gap-2"
                            >
                                <Clock size={16} class="text-[#4285F4]" />
                                {$t("ai_generator.duration_label")}
                            </span>
                            <div class="flex flex-wrap gap-2">
                                {#each ["60", "90", "120", "150", "180", "custom"] as d}
                                    <button
                                        onclick={() => (handsOnDuration = d)}
                                        class="px-4 py-2 rounded-xl text-xs font-bold transition-all border {handsOnDuration ===
                                        d
                                            ? 'bg-[#4285F4] text-white border-[#4285F4] shadow-md'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border hover:border-[#4285F4]'}"
                                    >
                                        {d === "custom"
                                            ? $t("ai_generator.duration_custom")
                                            : $t("ai_generator.duration_mins", {
                                                  values: { mins: d },
                                              })}
                                    </button>
                                {/each}
                                {#if handsOnDuration === "custom"}
                                    <div
                                        class="flex items-center gap-2 ml-2"
                                        in:fade
                                    >
                                        <input
                                            type="number"
                                            bind:value={customDuration}
                                            placeholder="10"
                                            class="w-20 bg-white dark:bg-dark-bg border border-[#DADCE0] dark:border-dark-border rounded-lg px-3 py-2 text-xs outline-none focus:border-[#4285F4] focus:ring-2 focus:ring-[#4285F4]/10"
                                        />
                                        <span
                                            class="text-xs font-medium text-[#5F6368] dark:text-dark-text-muted"
                                            >{$t("ai_generator.mins")}</span
                                        >
                                    </div>
                                {/if}
                            </div>
                        </div>

                        <div
                            class="h-px bg-[#F1F3F4] dark:bg-dark-border w-full"
                        ></div>

                        <div class="flex flex-wrap gap-6">
                            <label
                                class="flex items-center gap-2 cursor-pointer group"
                            >
                                <input
                                    type="checkbox"
                                    bind:checked={advancedUseGoogleSearch}
                                    class="w-5 h-5 rounded border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-[#4285F4] focus:ring-[#4285F4]"
                                />
                                <span
                                    class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4]"
                                >
                                    {$t("ai_generator.google_search")}
                                </span>
                            </label>

                            <label
                                class="flex items-center gap-2 cursor-pointer group"
                            >
                                <input
                                    type="checkbox"
                                    bind:checked={advancedUseUrlContext}
                                    class="w-5 h-5 rounded border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-[#4285F4] focus:ring-[#4285F4]"
                                />
                                <span
                                    class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4]"
                                >
                                    {$t("ai_generator.url_context")}
                                </span>
                            </label>

                            <label
                                class="flex items-center gap-2 cursor-pointer group"
                            >
                                <input
                                    type="checkbox"
                                    bind:checked={enableCodeServer}
                                    disabled={uploadedFiles.length === 0}
                                    class="w-5 h-5 rounded border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-[#4285F4] focus:ring-[#4285F4] disabled:opacity-50 disabled:cursor-not-allowed"
                                />
                                <span
                                    class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4] {uploadedFiles.length === 0 ? 'opacity-50' : ''}"
                                >
                                    Create Code Server Workspace
                                </span>
                            </label>

                            <label
                                class="flex items-center gap-2 cursor-pointer group"
                            >
                                <input
                                    type="checkbox"
                                    bind:checked={showThinking}
                                    class="w-5 h-5 rounded border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-[#4285F4] focus:ring-[#4285F4]"
                                />
                                <span
                                    class="text-sm font-medium text-[#5F6368] dark:text-dark-text-muted group-hover:text-[#4285F4]"
                                >
                                    {$t("ai_generator.show_thinking")}
                                </span>
                            </label>
                        </div>
                    </div>

                    {#if advancedUseGoogleSearch || advancedUseUrlContext}
                        <div
                            class="flex items-start gap-2 p-3 bg-[#FEF7E0] dark:bg-[#F9AB00]/10 border border-[#F9AB00]/30 rounded-lg mb-4"
                        >
                            <Info
                                size={16}
                                class="text-[#F9AB00] mt-0.5 shrink-0"
                            />
                            <p
                                class="text-xs text-[#3C4043] dark:text-dark-text"
                            >
                                <strong
                                    >{$t("ai_generator.billing_notice")}</strong
                                >
                                {$t("ai_generator.billing_desc")}
                            </p>
                        </div>
                    {/if}

                    <textarea
                        id="source-code"
                        bind:value={sourceCode}
                        placeholder={$t("ai_generator.placeholder")}
                        class="flex-1 min-h-[320px] lg:min-h-[420px] w-full bg-white dark:bg-dark-surface text-[#3C4043] dark:text-dark-text border border-[#DADCE0] dark:border-dark-border rounded-xl p-4 font-mono text-sm focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 outline-none resize-none shadow-sm transition-all"
                    ></textarea>

                    <div class="flex justify-end pt-2">
                        {#if !apiKey}
                            <p
                                class="text-[#EA4335] font-bold mr-4 self-center"
                            >
                                {$t("ai_generator.api_key_required")}
                            </p>
                            <button
                                disabled
                                class="bg-[#E8EAED] dark:bg-dark-border text-[#9AA0A6] dark:text-dark-text-muted px-8 py-3 rounded-full font-bold cursor-not-allowed"
                            >
                                {$t("common.create")}
                            </button>
                        {:else}
                            <button
                                onclick={handleAdvancedPlan}
                                disabled={advancedLoading ||
                                    (!sourceCode.trim() &&
                                        uploadedFiles.length === 0)}
                                class="bg-[#4285F4] text-white px-8 py-3 rounded-full font-bold hover:bg-[#1A73E8] hover:shadow-lg transition-colors text-lg flex items-center gap-2 disabled:opacity-50"
                            >
                                <Sparkles size={20} />
                                {$t("ai_generator.plan_button")}
                            </button>
                        {/if}
                    </div>
                </div>
            {:else if advancedStep === "planning" || advancedStep === "drafting" || advancedStep === "reviewing" || advancedStep === "revising"}
                <div
                    class="min-h-full flex flex-col items-center justify-center gap-6"
                    in:fade
                    aria-live="polite"
                >
                    <div class="relative">
                        <div
                            class="absolute inset-0 bg-[#4285F4] rounded-full blur-xl opacity-20 animate-pulse"
                            aria-hidden="true"
                        ></div>
                        <Loader2
                            class="w-16 h-16 text-[#4285F4] animate-spin relative z-10"
                            aria-hidden="true"
                        />
                    </div>
                    <h3
                        class="text-xl font-bold text-[#3C4043] dark:text-dark-text"
                    >
                        {#if advancedStep === "planning"}
                            {$t("ai_generator.plan_loading")}
                        {:else if advancedStep === "drafting"}
                            {$t("ai_generator.draft_loading")}
                        {:else if advancedStep === "reviewing"}
                            {$t("ai_generator.review_loading")}
                        {:else}
                            {$t("ai_generator.revise_loading")}
                        {/if}
                    </h3>
                    <p
                        class="text-[#5F6368] dark:text-dark-text-muted text-center"
                    >
                        {#if advancedStep === "planning"}
                            {$t("ai_generator.plan_loading_desc")}
                        {:else if advancedStep === "drafting"}
                            {$t("ai_generator.draft_loading_desc")}
                        {:else if advancedStep === "reviewing"}
                            {$t("ai_generator.review_loading_desc")}
                        {:else}
                            {$t("ai_generator.revise_loading_desc")}
                        {/if}
                    </p>

                    {#if showThinking && advancedThinkingContent}
                        <div class="w-full max-w-2xl mt-6">
                            <details
                                open
                                class="bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border shadow-sm overflow-hidden"
                            >
                                <summary
                                    class="px-4 py-3 cursor-pointer hover:bg-[#F8F9FA] dark:hover:bg-white/5 flex items-center gap-2 font-medium text-[#5F6368] dark:text-dark-text-muted"
                                >
                                    <Sparkles
                                        size={16}
                                        class="text-[#4285F4]"
                                    />
                                    {$t("ai_generator.thinking_process")}
                                </summary>
                                <div
                                    class="px-4 py-3 text-xs text-[#5F6368] dark:text-dark-text-muted font-mono bg-[#F8F9FA] dark:bg-dark-bg/50 max-h-48 overflow-y-auto border-t border-[#E8EAED] dark:border-dark-border"
                                >
                                    {advancedThinkingContent}
                                </div>
                            </details>
                        </div>
                    {/if}

                    <div
                        class="w-full max-w-2xl h-32 overflow-hidden text-xs text-[#9AA0A6] dark:text-dark-text-muted font-mono text-center opacity-50 relative mt-8"
                    >
                        <div
                            class="absolute inset-x-0 bottom-0 h-12 bg-gradient-to-t from-[#F8F9FA] dark:from-dark-bg to-transparent"
                        ></div>
                        {advancedStreamContent.slice(-500)}
                    </div>
                </div>
            {:else if advancedStep === "plan" && advancedPlanData}
                <div class="min-h-full flex flex-col gap-6" in:fade>
                    <div
                        class="flex items-center justify-between border-b border-[#E8EAED] dark:border-dark-border pb-4"
                    >
                        <div>
                            <h3
                                class="text-xl font-bold text-[#202124] dark:text-dark-text"
                            >
                                {$t("ai_generator.plan_title")}
                            </h3>
                            <p
                                class="text-[#5F6368] dark:text-dark-text-muted text-sm"
                            >
                                {$t("ai_generator.plan_subtitle")}
                            </p>
                        </div>
                        <div class="flex gap-3">
                            <button
                                onclick={() => (advancedStep = "input")}
                                class="px-6 py-2 text-[#5F6368] dark:text-dark-text-muted font-bold hover:bg-[#E8EAED] dark:hover:bg-dark-border rounded-full transition-all"
                            >
                                {$t("ai_generator.back")}
                            </button>
                            <button
                                onclick={handleAdvancedDraft}
                                disabled={advancedLoading}
                                class="bg-[#4285F4] text-white px-8 py-2 rounded-full font-bold hover:shadow-md transition-all flex items-center gap-2"
                            >
                                <ArrowRight size={18} />
                                {$t("ai_generator.draft_button")}
                            </button>
                        </div>
                    </div>

                    <div
                        bind:this={planContainerRef}
                        onmouseup={handlePlanSelection}
                        onkeyup={handlePlanSelection}
                        onscroll={clearPlanSelection}
                        class="flex-1 overflow-y-auto bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-8 shadow-sm"
                    >
                        <div class="space-y-6">
                            <div
                                class="flex items-start gap-2 p-3 bg-[#F8F9FA] dark:bg-dark-bg border border-[#E8EAED] dark:border-dark-border rounded-lg"
                            >
                                <Info
                                    size={16}
                                    class="text-[#4285F4] mt-0.5 shrink-0"
                                />
                                <p
                                    class="text-xs text-[#3C4043] dark:text-dark-text"
                                >
                                    {$t("ai_generator.plan_comment_hint")}
                                </p>
                            </div>
                            <div class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6">
                                <h1
                                    class="text-2xl font-bold text-[#202124] dark:text-dark-text mb-2"
                                >
                                    {advancedPlanData.title}
                                </h1>
                                <p
                                    class="text-sm text-[#5F6368] dark:text-dark-text-muted"
                                >
                                    {advancedPlanData.description}
                                </p>
                                {#if advancedPlanData.audience}
                                    <p
                                        class="mt-3 text-sm text-[#3C4043] dark:text-dark-text-muted"
                                    >
                                        <span
                                            class="font-semibold text-[#202124] dark:text-dark-text"
                                            >{$t(
                                                "ai_generator.plan_audience_label",
                                            )}</span
                                        >
                                        {advancedPlanData.audience}
                                    </p>
                                {/if}
                            </div>

                            {#if advancedPlanData.learning_objectives.length}
                                <div class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6">
                                    <h4
                                        class="font-bold text-[#202124] dark:text-dark-text mb-3"
                                    >
                                        {$t(
                                            "ai_generator.plan_objectives_label",
                                        )}
                                    </h4>
                                    <ul
                                        class="list-disc ml-5 text-sm text-[#3C4043] dark:text-dark-text-muted"
                                    >
                                        {#each advancedPlanData.learning_objectives as objective}
                                            <li>{objective}</li>
                                        {/each}
                                    </ul>
                                </div>
                            {/if}

                            {#if advancedPlanData.prerequisites.length}
                                <div class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6">
                                    <h4
                                        class="font-bold text-[#202124] dark:text-dark-text mb-3"
                                    >
                                        {$t(
                                            "ai_generator.plan_prerequisites_label",
                                        )}
                                    </h4>
                                    <ul
                                        class="list-disc ml-5 text-sm text-[#3C4043] dark:text-dark-text-muted"
                                    >
                                        {#each advancedPlanData.prerequisites as item}
                                            <li>{item}</li>
                                        {/each}
                                    </ul>
                                </div>
                            {/if}

                            <div class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6">
                                <h4
                                    class="font-bold text-[#202124] dark:text-dark-text mb-3"
                                >
                                    {$t(
                                        "ai_generator.plan_environment_label",
                                    )}
                                </h4>
                                {#if advancedPlanData.environment_setup.os_requirements.length}
                                    <p
                                        class="text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted"
                                    >
                                        {$t(
                                            "ai_generator.plan_environment_os",
                                        )}
                                    </p>
                                    <p
                                        class="text-sm text-[#3C4043] dark:text-dark-text-muted mb-3"
                                    >
                                        {advancedPlanData.environment_setup.os_requirements.join(
                                            ", ",
                                        )}
                                    </p>
                                {/if}
                                {#if advancedPlanData.environment_setup.tools.length}
                                    <p
                                        class="text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted"
                                    >
                                        {$t(
                                            "ai_generator.plan_environment_tools",
                                        )}
                                    </p>
                                    <p
                                        class="text-sm text-[#3C4043] dark:text-dark-text-muted mb-3"
                                    >
                                        {advancedPlanData.environment_setup.tools.join(
                                            ", ",
                                        )}
                                    </p>
                                {/if}
                                {#if advancedPlanData.environment_setup.env_vars.length}
                                    <p
                                        class="text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted"
                                    >
                                        {$t(
                                            "ai_generator.plan_environment_envvars",
                                        )}
                                    </p>
                                    <p
                                        class="text-sm text-[#3C4043] dark:text-dark-text-muted mb-3"
                                    >
                                        {advancedPlanData.environment_setup.env_vars.join(
                                            ", ",
                                        )}
                                    </p>
                                {/if}
                                {#if advancedPlanData.environment_setup.ide}
                                    <p
                                        class="text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted"
                                    >
                                        {$t(
                                            "ai_generator.plan_environment_ide",
                                        )}
                                    </p>
                                    <p
                                        class="text-sm text-[#3C4043] dark:text-dark-text-muted mb-3"
                                    >
                                        {advancedPlanData.environment_setup.ide}
                                    </p>
                                {/if}
                                {#if advancedPlanData.environment_setup.ide_plugins.length}
                                    <p
                                        class="text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted"
                                    >
                                        {$t(
                                            "ai_generator.plan_environment_plugins",
                                        )}
                                    </p>
                                    <p
                                        class="text-sm text-[#3C4043] dark:text-dark-text-muted"
                                    >
                                        {advancedPlanData.environment_setup.ide_plugins.join(
                                            ", ",
                                        )}
                                    </p>
                                {/if}
                            </div>

                            {#if advancedPlanData.search_terms.length}
                                <div class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6">
                                    <h4
                                        class="font-bold text-[#202124] dark:text-dark-text mb-3"
                                    >
                                        {$t(
                                            "ai_generator.plan_search_terms_label",
                                        )}
                                    </h4>
                                    <div class="flex flex-wrap gap-2">
                                        {#each advancedPlanData.search_terms as term}
                                            <span
                                                class="px-3 py-1 rounded-full bg-[#E8EAED] dark:bg-dark-border text-xs font-semibold text-[#3C4043] dark:text-dark-text"
                                            >
                                                {term}
                                            </span>
                                        {/each}
                                    </div>
                                </div>
                            {/if}

                            {#if advancedPlanData.steps.length}
                                <div class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6">
                                    <h4
                                        class="font-bold text-[#202124] dark:text-dark-text mb-3"
                                    >
                                        {$t(
                                            "ai_generator.plan_steps_label",
                                        )}
                                    </h4>
                                    <div class="space-y-4">
                                        {#each advancedPlanData.steps as step, i}
                                            <div
                                                class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-4"
                                            >
                                                <h5
                                                    class="font-bold text-[#202124] dark:text-dark-text"
                                                >
                                                    {i + 1}. {step.title}
                                                </h5>
                                                <p
                                                    class="text-sm text-[#3C4043] dark:text-dark-text-muted mt-2"
                                                >
                                                    {step.goal}
                                                </p>
                                                {#if step.files.length}
                                                    <p
                                                        class="text-xs text-[#5F6368] dark:text-dark-text-muted mt-2"
                                                    >
                                                        <span
                                                            class="font-semibold"
                                                            >{$t(
                                                                "ai_generator.plan_files_label",
                                                            )}</span
                                                        >
                                                        {step.files.join(
                                                            ", ",
                                                        )}
                                                    </p>
                                                {/if}
                                                <p
                                                    class="text-xs text-[#5F6368] dark:text-dark-text-muted mt-1"
                                                >
                                                    <span
                                                        class="font-semibold"
                                                        >{$t(
                                                            "ai_generator.plan_verification_label",
                                                        )}</span
                                                    >
                                                    {step.verification}
                                                </p>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {/if}
                            <div class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6">
                                <h4
                                    class="font-bold text-[#202124] dark:text-dark-text mb-3"
                                >
                                    {$t("ai_generator.plan_comment_label")}
                                </h4>
                                {#if planComments.length}
                                    <div class="space-y-3">
                                        {#each planComments as comment}
                                            <div
                                                class="border border-[#E8EAED] dark:border-dark-border rounded-lg p-4"
                                            >
                                                <p
                                                    class="text-[11px] font-semibold uppercase text-[#5F6368] dark:text-dark-text-muted"
                                                >
                                                    {$t(
                                                        "ai_generator.plan_comment_selection_label",
                                                    )}
                                                </p>
                                                <p
                                                    class="text-xs text-[#3C4043] dark:text-dark-text font-mono whitespace-pre-wrap line-clamp-3 mt-1"
                                                >
                                                    {comment.quote}
                                                </p>
                                                <p
                                                    class="text-sm text-[#202124] dark:text-dark-text mt-2"
                                                >
                                                    {comment.comment}
                                                </p>
                                                <button
                                                    onclick={() =>
                                                        removePlanComment(
                                                            comment.id,
                                                        )}
                                                    class="mt-3 text-xs font-semibold text-[#EA4335] hover:underline"
                                                >
                                                    {$t(
                                                        "ai_generator.plan_comment_remove",
                                                    )}
                                                </button>
                                            </div>
                                        {/each}
                                    </div>
                                {:else}
                                    <p
                                        class="text-xs text-[#9AA0A6] dark:text-dark-text-muted"
                                    >
                                        {$t("ai_generator.plan_comment_empty")}
                                    </p>
                                {/if}
                            </div>
                        </div>
                    </div>
                    {#if planSelection}
                        <div
                            class="fixed z-[60] w-80 max-w-[calc(100%-1.5rem)] bg-white dark:bg-dark-surface border border-[#E8EAED] dark:border-dark-border rounded-xl shadow-xl p-4"
                            style="top: {planSelection.top}px; left: {planSelection.left}px;"
                            role="dialog"
                            aria-label={$t("ai_generator.plan_comment_title")}
                        >
                            <p
                                class="text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted"
                            >
                                {$t("ai_generator.plan_comment_title")}
                            </p>
                            <p
                                class="text-xs text-[#3C4043] dark:text-dark-text font-mono whitespace-pre-wrap line-clamp-3 mt-2"
                            >
                                {planSelection.text}
                            </p>
                            <textarea
                                bind:this={planCommentInputRef}
                                bind:value={planCommentDraft}
                                placeholder={$t(
                                    "ai_generator.plan_comment_placeholder",
                                )}
                                onkeydown={(event) => {
                                    if (event.key === "Escape") {
                                        event.preventDefault();
                                        clearPlanSelection();
                                    }
                                }}
                                class="mt-3 w-full h-24 resize-none rounded-lg border border-[#DADCE0] dark:border-dark-border bg-white dark:bg-dark-surface text-sm text-[#202124] dark:text-dark-text px-3 py-2 outline-none focus:border-[#4285F4] focus:ring-2 focus:ring-[#4285F4]/10"
                            ></textarea>
                            <div class="mt-3 flex justify-end gap-2">
                                <button
                                    onclick={clearPlanSelection}
                                    class="px-3 py-1.5 text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted hover:text-[#202124] dark:hover:text-dark-text"
                                >
                                    {$t("ai_generator.plan_comment_cancel")}
                                </button>
                                <button
                                    onclick={addPlanComment}
                                    disabled={!planCommentDraft.trim()}
                                    class="px-4 py-1.5 rounded-lg bg-[#4285F4] text-white text-xs font-semibold disabled:opacity-50"
                                >
                                    {$t("ai_generator.plan_comment_add")}
                                </button>
                            </div>
                        </div>
                    {/if}
                </div>
            {:else if advancedStep === "draft" && advancedDraftData}
                <div class="min-h-full flex flex-col gap-6" in:fade>
                    <div
                        class="flex items-center justify-between border-b border-[#E8EAED] dark:border-dark-border pb-4"
                    >
                        <div>
                            <h3
                                class="text-xl font-bold text-[#202124] dark:text-dark-text"
                            >
                                {$t("ai_generator.draft_title")}
                            </h3>
                            <p
                                class="text-[#5F6368] dark:text-dark-text-muted text-sm"
                            >
                                {$t("ai_generator.draft_subtitle")}
                            </p>
                        </div>
                        <div class="flex gap-3">
                            <button
                                onclick={() => (advancedStep = "plan")}
                                class="px-6 py-2 text-[#5F6368] dark:text-dark-text-muted font-bold hover:bg-[#E8EAED] dark:hover:bg-dark-border rounded-full transition-all"
                            >
                                {$t("ai_generator.back")}
                            </button>
                            <button
                                onclick={handleAdvancedReviewAndRevise}
                                disabled={advancedLoading}
                                class="bg-[#34A853] text-white px-8 py-2 rounded-full font-bold hover:bg-[#1E8E3E] shadow-md transition-all flex items-center gap-2"
                            >
                                <ArrowRight size={18} />
                                {$t("ai_generator.review_button")}
                            </button>
                        </div>
                    </div>

                    <div
                        class="flex-1 overflow-y-auto bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-8 shadow-sm"
                    >
                        <h1
                            class="text-3xl font-bold text-[#202124] dark:text-dark-text mb-4"
                        >
                            {advancedDraftData.title}
                        </h1>
                        <p
                            class="text-lg text-[#5F6368] dark:text-dark-text-muted mb-8"
                        >
                            {advancedDraftData.description}
                        </p>

                        <div class="space-y-8">
                            {#each advancedDraftData.steps as step, i}
                                <div
                                    class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6 hover:shadow-sm transition-shadow"
                                >
                                    <h4
                                        class="font-bold text-lg text-[#202124] dark:text-dark-text mb-2"
                                    >
                                        {i + 1}. {step.title}
                                    </h4>
                                    <div
                                        class="text-[#3C4043] dark:text-dark-text-muted text-sm line-clamp-3 opacity-80"
                                    >
                                        {step.content}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                </div>
            {:else if advancedStep === "final" && advancedRevisedData}
                <div class="min-h-full flex flex-col gap-6" in:fade>
                    <div
                        class="flex items-center justify-between border-b border-[#E8EAED] dark:border-dark-border pb-4"
                    >
                        <div>
                            <h3
                                class="text-xl font-bold text-[#202124] dark:text-dark-text"
                            >
                                {$t("ai_generator.final_title")}
                            </h3>
                            <p
                                class="text-[#5F6368] dark:text-dark-text-muted text-sm"
                            >
                                {$t("ai_generator.final_subtitle")}
                            </p>
                        </div>
                        <div class="flex gap-3">
                            <button
                                onclick={() => (advancedStep = "draft")}
                                class="px-6 py-2 text-[#5F6368] dark:text-dark-text-muted font-bold hover:bg-[#E8EAED] dark:hover:bg-dark-border rounded-full transition-all"
                            >
                                {$t("ai_generator.back")}
                            </button>
                            <button
                                onclick={handleSaveAdvanced}
                                disabled={advancedLoading}
                                class="bg-[#34A853] text-white px-8 py-2 rounded-full font-bold hover:bg-[#1E8E3E] shadow-md transition-all flex items-center gap-2"
                            >
                                {#if advancedLoading}
                                    <Loader2 class="animate-spin" size={18} />
                                    {$t("ai_generator.saving")}
                                {:else}
                                    <ArrowRight size={18} />
                                    {$t("ai_generator.create_button")}
                                {/if}
                            </button>
                        </div>
                    </div>

                    <div class="flex-1 overflow-y-auto space-y-6">
                        {#if advancedReviewData}
                            <div
                                class="bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-6 shadow-sm"
                            >
                                <h4
                                    class="text-lg font-bold text-[#202124] dark:text-dark-text mb-2"
                                >
                                    {$t("ai_generator.review_title")}
                                </h4>
                                <p
                                    class="text-sm text-[#5F6368] dark:text-dark-text-muted"
                                >
                                    {advancedReviewData.summary}
                                </p>

                                {#if advancedReviewData.issues.length}
                                    <div class="mt-4">
                                        <h5
                                            class="text-sm font-bold text-[#202124] dark:text-dark-text mb-2"
                                        >
                                            {$t(
                                                "ai_generator.review_issues_label",
                                            )}
                                        </h5>
                                        <div class="space-y-3">
                                            {#each advancedReviewData.issues as issue}
                                                <div
                                                    class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-3"
                                                >
                                                    <div
                                                        class="flex items-center gap-2 text-xs font-semibold text-[#5F6368] dark:text-dark-text-muted"
                                                    >
                                                        <span
                                                            class="px-2 py-0.5 rounded-full bg-[#E8EAED] dark:bg-dark-border text-[#3C4043] dark:text-dark-text"
                                                            >{issue.severity}</span
                                                        >
                                                        <span>{issue.issue}</span>
                                                    </div>
                                                    <p
                                                        class="text-xs text-[#5F6368] dark:text-dark-text-muted mt-2"
                                                    >
                                                        {issue.recommendation}
                                                    </p>
                                                </div>
                                            {/each}
                                        </div>
                                    </div>
                                {/if}

                                {#if advancedReviewData.missing_items.length}
                                    <div class="mt-4">
                                        <h5
                                            class="text-sm font-bold text-[#202124] dark:text-dark-text mb-2"
                                        >
                                            {$t(
                                                "ai_generator.review_missing_label",
                                            )}
                                        </h5>
                                        <ul
                                            class="list-disc ml-5 text-sm text-[#3C4043] dark:text-dark-text-muted"
                                        >
                                            {#each advancedReviewData.missing_items as item}
                                                <li>{item}</li>
                                            {/each}
                                        </ul>
                                    </div>
                                {/if}

                                {#if advancedReviewData.improvements.length}
                                    <div class="mt-4">
                                        <h5
                                            class="text-sm font-bold text-[#202124] dark:text-dark-text mb-2"
                                        >
                                            {$t(
                                                "ai_generator.review_suggestions_label",
                                            )}
                                        </h5>
                                        <ul
                                            class="list-disc ml-5 text-sm text-[#3C4043] dark:text-dark-text-muted"
                                        >
                                            {#each advancedReviewData.improvements as item}
                                                <li>{item}</li>
                                            {/each}
                                        </ul>
                                    </div>
                                {/if}
                            </div>
                        {/if}

                        <div
                            class="bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-6 shadow-sm"
                        >
                            <div class="flex flex-wrap items-center justify-between gap-3 mb-3">
                                <div>
                                    <h4
                                        class="text-lg font-bold text-[#202124] dark:text-dark-text mb-1"
                                    >
                                        {$t("ai_generator.diff_title")}
                                    </h4>
                                    <p class="text-sm text-[#5F6368] dark:text-dark-text-muted">
                                        {$t("ai_generator.diff_desc")}
                                    </p>
                                </div>
                                <div class="flex items-center gap-1">
                                    <button
                                        onclick={() => (advancedDiffView = "unified")}
                                        class="px-2 py-1 rounded-full text-[10px] font-bold border {advancedDiffView === 'unified'
                                            ? 'bg-[#1A73E8] text-white border-[#1A73E8]'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border'}"
                                    >
                                        {$t("ai_generator.diff_view_unified")}
                                    </button>
                                    <button
                                        onclick={() => (advancedDiffView = "split")}
                                        class="px-2 py-1 rounded-full text-[10px] font-bold border {advancedDiffView === 'split'
                                            ? 'bg-[#1A73E8] text-white border-[#1A73E8]'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border'}"
                                    >
                                        {$t("ai_generator.diff_view_split")}
                                    </button>
                                </div>
                            </div>

                            {#if !advancedDraftMarkdown || !advancedRevisedMarkdown}
                                <p class="text-sm text-[#9AA0A6] dark:text-dark-text-muted">
                                    {$t("ai_generator.diff_empty")}
                                </p>
                            {:else if advancedDiff.truncated}
                                <p class="text-sm text-[#9AA0A6] dark:text-dark-text-muted">
                                    {$t("ai_generator.diff_too_large")}
                                </p>
                            {:else if advancedDiffView === "unified"}
                                <div class="rounded-xl border border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface max-h-72 overflow-y-auto">
                                    <div class="font-mono text-[11px] leading-relaxed">
                                        {#each advancedDiff.lines as line}
                                            <div
                                                class="flex items-start gap-2 px-3 py-1 {line.type === 'add'
                                                    ? 'bg-[#E6F4EA] text-[#137333]'
                                                    : line.type === 'remove'
                                                        ? 'bg-[#FCE8E6] text-[#C5221F]'
                                                        : 'text-[#3C4043] dark:text-dark-text'}"
                                            >
                                                <span class="w-4 text-[10px] font-bold">
                                                    {line.type === "add"
                                                        ? "+"
                                                        : line.type === "remove"
                                                            ? "-"
                                                            : " "}
                                                </span>
                                                <span class="whitespace-pre-wrap break-words flex-1">
                                                    {line.text || " "}
                                                </span>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {:else}
                                <div class="rounded-xl border border-[#E8EAED] dark:border-dark-border bg-white dark:bg-dark-surface max-h-72 overflow-y-auto">
                                    <div class="grid grid-cols-2 font-mono text-[11px] leading-relaxed">
                                        <div class="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-[#5F6368] dark:text-dark-text-muted bg-[#F8F9FA] dark:bg-dark-bg border-b border-[#E8EAED] dark:border-dark-border">
                                            {$t("ai_generator.diff_left")}
                                        </div>
                                        <div class="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-[#5F6368] dark:text-dark-text-muted bg-[#F8F9FA] dark:bg-dark-bg border-b border-l border-[#E8EAED] dark:border-dark-border">
                                            {$t("ai_generator.diff_right")}
                                        </div>
                                        {#each advancedDiffRows as row}
                                            <div
                                                class="flex items-start gap-2 px-3 py-1 border-r border-[#E8EAED] dark:border-dark-border {row.leftType === 'remove'
                                                    ? 'bg-[#FCE8E6] text-[#C5221F]'
                                                    : 'text-[#3C4043] dark:text-dark-text'}"
                                            >
                                                <span class="w-4 text-[10px] font-bold">
                                                    {row.leftType === "remove" ? "-" : " "}
                                                </span>
                                                <span class="whitespace-pre-wrap break-words flex-1">
                                                    {row.leftText || " "}
                                                </span>
                                            </div>
                                            <div
                                                class="flex items-start gap-2 px-3 py-1 {row.rightType === 'add'
                                                    ? 'bg-[#E6F4EA] text-[#137333]'
                                                    : 'text-[#3C4043] dark:text-dark-text'}"
                                            >
                                                <span class="w-4 text-[10px] font-bold">
                                                    {row.rightType === "add" ? "+" : " "}
                                                </span>
                                                <span class="whitespace-pre-wrap break-words flex-1">
                                                    {row.rightText || " "}
                                                </span>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {/if}
                        </div>

                        <div
                            class="bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-6 shadow-sm"
                        >
                            <div class="flex items-center justify-between gap-3 mb-3">
                                <h4 class="text-lg font-bold text-[#202124] dark:text-dark-text">
                                    {$t("ai_generator.draft_markdown_title")}
                                </h4>
                                <div class="flex items-center gap-1">
                                    <button
                                        onclick={() => (advancedDraftView = "markdown")}
                                        class="px-2 py-1 rounded-full text-[10px] font-bold border {advancedDraftView === 'markdown'
                                            ? 'bg-[#1A73E8] text-white border-[#1A73E8]'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border'}"
                                    >
                                        {$t("ai_generator.view_markdown")}
                                    </button>
                                    <button
                                        onclick={() => (advancedDraftView = "raw")}
                                        class="px-2 py-1 rounded-full text-[10px] font-bold border {advancedDraftView === 'raw'
                                            ? 'bg-[#1A73E8] text-white border-[#1A73E8]'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border'}"
                                    >
                                        {$t("ai_generator.view_raw")}
                                    </button>
                                </div>
                            </div>
                            {#if advancedDraftMarkdown}
                                {#if advancedDraftView === "markdown"}
                                    <div class="markdown-body text-sm">
                                        {@html advancedDraftHtml}
                                    </div>
                                {:else}
                                    <pre class="text-[11px] leading-relaxed font-mono whitespace-pre-wrap text-[#3C4043] dark:text-dark-text">{advancedDraftMarkdown}</pre>
                                {/if}
                            {:else}
                                <p class="text-sm text-[#9AA0A6] dark:text-dark-text-muted">
                                    {$t("ai_generator.diff_empty")}
                                </p>
                            {/if}
                        </div>

                        <div
                            class="bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-6 shadow-sm"
                        >
                            <div class="flex items-center justify-between gap-3 mb-3">
                                <h4 class="text-lg font-bold text-[#202124] dark:text-dark-text">
                                    {$t("ai_generator.revised_markdown_title")}
                                </h4>
                                <div class="flex items-center gap-1">
                                    <button
                                        onclick={() => (advancedRevisedView = "markdown")}
                                        class="px-2 py-1 rounded-full text-[10px] font-bold border {advancedRevisedView === 'markdown'
                                            ? 'bg-[#1A73E8] text-white border-[#1A73E8]'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border'}"
                                    >
                                        {$t("ai_generator.view_markdown")}
                                    </button>
                                    <button
                                        onclick={() => (advancedRevisedView = "raw")}
                                        class="px-2 py-1 rounded-full text-[10px] font-bold border {advancedRevisedView === 'raw'
                                            ? 'bg-[#1A73E8] text-white border-[#1A73E8]'
                                            : 'bg-white dark:bg-dark-surface text-[#5F6368] dark:text-dark-text-muted border-[#DADCE0] dark:border-dark-border'}"
                                    >
                                        {$t("ai_generator.view_raw")}
                                    </button>
                                </div>
                            </div>
                            {#if advancedRevisedMarkdown}
                                {#if advancedRevisedView === "markdown"}
                                    <div class="markdown-body text-sm">
                                        {@html advancedRevisedHtml}
                                    </div>
                                {:else}
                                    <pre class="text-[11px] leading-relaxed font-mono whitespace-pre-wrap text-[#3C4043] dark:text-dark-text">{advancedRevisedMarkdown}</pre>
                                {/if}
                            {:else}
                                <p class="text-sm text-[#9AA0A6] dark:text-dark-text-muted">
                                    {$t("ai_generator.diff_empty")}
                                </p>
                            {/if}
                        </div>

                        <div
                            class="bg-white dark:bg-dark-surface rounded-xl border border-[#E8EAED] dark:border-dark-border p-8 shadow-sm"
                        >
                            <h1
                                class="text-3xl font-bold text-[#202124] dark:text-dark-text mb-4"
                            >
                                {advancedRevisedData.title}
                            </h1>
                            <p
                                class="text-lg text-[#5F6368] dark:text-dark-text-muted mb-8"
                            >
                                {advancedRevisedData.description}
                            </p>

                            <div class="space-y-8">
                                {#each advancedRevisedData.steps as step, i}
                                    <div
                                        class="border border-[#F1F3F4] dark:border-dark-border rounded-lg p-6 hover:shadow-sm transition-shadow"
                                    >
                                        <h4
                                            class="font-bold text-lg text-[#202124] dark:text-dark-text mb-2"
                                        >
                                            {i + 1}. {step.title}
                                        </h4>
                                        <div
                                            class="text-[#3C4043] dark:text-dark-text-muted text-sm line-clamp-3 opacity-80"
                                        >
                                            {step.content}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    </div>
                </div>
            {/if}
        {/if}
            </div>
        </div>
    </div>
</div>
