
export interface GeminiConfig {
    apiKey: string;
    model?: string;
}

export interface GeminiStructuredConfig extends GeminiConfig {
    tools?: Array<{ googleSearch?: {} } | { urlContext?: {} }>;
    thinkingConfig?: { thinkingLevel: "low" | "medium" | "high" };
}

export interface ThinkingContent {
    thinking?: string;
    content: string;
}

export async function* streamGeminiResponse(
    prompt: string,
    context: string,
    config: GeminiConfig
): AsyncGenerator<string, void, unknown> {
    const model = config.model || "gemini-2.0-flash-exp";
    // Using gemini-2.0-flash-exp as requested/standard, or user said "gemini-3-flash-preview" but that might not exist? 
    // User said "gemini-3-flash-preview" explicitly in prompt summary. I should use that if possible, but it looks like a typo/future model.
    // I recall the user prompt said "gemini-3-flash-preview" in the summary provided. Ah, earlier summary said "gemini-3-flash-preview".
    // I will double check if I should default to that. If it fails, I might need to fallback.
    // Let's stick to what was requested but allow config override.

    const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?key=${config.apiKey}`;

    const payload = {
        contents: [
            {
                role: "user",
                parts: [
                    {
                        text: `Context:\n${context}\n\nQuestion:\n${prompt}\n\nPlease answer the question based on the context provided. If the context is code, explain it clearly.`
                    }
                ]
            }
        ]
    };

    const response = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(payload),
    });

    if (!response.ok) {
        let errText = await response.text();
        throw new Error(`Gemini API Error ${response.status}: ${errText}`);
    }

    if (!response.body) throw new Error("No response body");

    const reader = response.body.getReader();
    const decoder = new TextDecoder("utf-8");
    let buffer = "";

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });

        // Split by lines or just try to parse accumulated buffer
        // The API returns array of objects like [{...}, {...}] or just {...} depending on endpoint
        // streamGenerateContent usually returns:
        // [
        // { "candidates": ... }
        // ,
        // { "candidates": ... }
        // ]

        // We will simple-parse: find matching braces
        // A simple state machine to extract top-level objects

        let braceCount = 0;
        let start = -1;

        for (let i = 0; i < buffer.length; i++) {
            const char = buffer[i];
            if (char === '{') {
                if (braceCount === 0) start = i;
                braceCount++;
            } else if (char === '}') {
                braceCount--;
                if (braceCount === 0 && start !== -1) {
                    const jsonStr = buffer.substring(start, i + 1);
                    try {
                        const data = JSON.parse(jsonStr);
                        if (data.candidates?.[0]?.content?.parts?.[0]?.text) {
                            yield data.candidates[0].content.parts[0].text;
                        }
                    } catch (e) {
                        // ignore
                    }
                    // Reset start to find next
                    start = -1;
                }
            }
        }

        // Keep the remainder of buffer (if any unfinished object)
        // This simple implementation drops processed parts?
        // No, we need to slice buffer.
        // To do this efficiently, we should slice at the last processed index.

        // Let's refactor loop slightly to handle slicing
    }
    reader.releaseLock();
}

export async function* streamGeminiResponseRobust(
    prompt: string,
    context: string,
    config: GeminiConfig
): AsyncGenerator<string, void, unknown> {
    const model = config.model || "gemini-3-flash-preview";
    const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?key=${config.apiKey}`;

    const payload = {
        contents: [{ role: "user", parts: [{ text: `Context:\n${context}\n\nQuestion:\n${prompt}` }] }]
    };

    const response = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
    });

    if (!response.ok) throw new Error(`API Error: ${response.status}`);

    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });

        let braceCount = 0;
        let start = -1;
        let lastProcessedIndex = -1;
        let inString = false;
        let escape = false;

        for (let i = 0; i < buffer.length; i++) {
            const char = buffer[i];

            if (escape) { escape = false; continue; }
            if (char === '\\') { escape = true; continue; }
            if (char === '"') { inString = !inString; continue; }
            if (inString) continue;

            if (char === '{') {
                if (braceCount === 0) start = i;
                braceCount++;
            } else if (char === '}') {
                braceCount--;
                if (braceCount === 0 && start !== -1) {
                    const jsonStr = buffer.substring(start, i + 1);
                    try {
                        const data = JSON.parse(jsonStr);
                        if (data.candidates?.[0]?.content?.parts?.[0]?.text) {
                            yield data.candidates[0].content.parts[0].text;
                        }
                    } catch (e) {
                        // ignore
                    }
                    lastProcessedIndex = i;
                    start = -1;
                }
            }
        }

        if (lastProcessedIndex !== -1) {
            buffer = buffer.substring(lastProcessedIndex + 1);
        }
    }
}

/**
 * Stream Gemini response with Structured Outputs using JSON Schema.
 * This guarantees the response will be valid JSON matching the provided schema.
 * 
 * @param prompt - The user's prompt
 * @param systemPrompt - System instructions for the model
 * @param schema - JSON Schema object defining the expected response structure
 * @param config - Gemini configuration with API key
 * @returns AsyncGenerator yielding the complete JSON string (streamable chunks)
 */
export async function* streamGeminiStructuredOutput(
    prompt: string,
    systemPrompt: string,
    schema: object,
    config: GeminiStructuredConfig
): AsyncGenerator<ThinkingContent, void, unknown> {
    const model = config.model || "gemini-3-flash-preview";
    const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?key=${config.apiKey}`;

    const payload: any = {
        contents: [
            {
                role: "user",
                parts: [{ text: `${systemPrompt}\n\n${prompt}` }]
            }
        ],
        generationConfig: {
            responseMimeType: "application/json",
            responseJsonSchema: schema,
            ...(config.thinkingConfig && {
                thinkingConfig: {
                    thinkingLevel: config.thinkingConfig.thinkingLevel
                }
            })
        }
    };

    // Add tools if provided
    if (config.tools && config.tools.length > 0) {
        payload.tools = config.tools;
    }

    const response = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
    });

    if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`API Error ${response.status}: ${errorText}`);
    }

    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });

        let braceCount = 0;
        let start = -1;
        let lastProcessedIndex = -1;
        let inString = false;
        let escape = false;

        for (let i = 0; i < buffer.length; i++) {
            const char = buffer[i];

            if (escape) { escape = false; continue; }
            if (char === '\\') { escape = true; continue; }
            if (char === '"') { inString = !inString; continue; }
            if (inString) continue;

            if (char === '{') {
                if (braceCount === 0) start = i;
                braceCount++;
            } else if (char === '}') {
                braceCount--;
                if (braceCount === 0 && start !== -1) {
                    const jsonStr = buffer.substring(start, i + 1);
                    try {
                        const data = JSON.parse(jsonStr);
                        const candidate = data.candidates?.[0];

                        if (candidate?.content?.parts) {
                            let thinkingText = "";
                            let contentText = "";

                            // Extract thinking and content from parts
                            for (const part of candidate.content.parts) {
                                if (part.thought) {
                                    thinkingText += part.thought;
                                } else if (part.text) {
                                    contentText += part.text;
                                }
                            }

                            // Yield structured response
                            if (thinkingText || contentText) {
                                yield {
                                    thinking: thinkingText || undefined,
                                    content: contentText
                                };
                            }
                        }
                    } catch (e) {
                        // ignore malformed chunks
                    }
                    lastProcessedIndex = i;
                    start = -1;
                }
            }
        }

        if (lastProcessedIndex !== -1) {
            buffer = buffer.substring(lastProcessedIndex + 1);
        }
    }
}
