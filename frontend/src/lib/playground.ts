export type PlaygroundLanguage = "dart" | "go" | "python" | "jupyter";

export type PlaygroundBlock = {
    language: PlaygroundLanguage;
    code: string;
};

const LANGUAGE_ALIASES: Record<string, PlaygroundLanguage> = {
    dart: "dart",
    go: "go",
    golang: "go",
    python: "python",
    py: "python",
    jupyter: "jupyter",
    ipynb: "jupyter",
    notebook: "jupyter",
};

const RUN_MARKERS = new Set(["run", "playground", "runnable"]);
const RUN_SUFFIX = /-(run|playground)$/i;

function normalizeLanguage(raw: string): PlaygroundLanguage | null {
    const cleaned = raw.trim().toLowerCase();
    if (!cleaned) return null;
    if (LANGUAGE_ALIASES[cleaned]) return LANGUAGE_ALIASES[cleaned];
    const withoutSuffix = cleaned.replace(RUN_SUFFIX, "");
    return LANGUAGE_ALIASES[withoutSuffix] ?? null;
}

export function extractPlaygrounds(markdown: string): PlaygroundBlock[] {
    if (!markdown) return [];

    const blocks: PlaygroundBlock[] = [];
    const seen = new Set<PlaygroundLanguage>();
    const fenceRegex = /(^|\n)[ \t]*(```|~~~)([^\n]*)\n([\s\S]*?)\n[ \t]*\2/g;
    let match: RegExpExecArray | null;

    while ((match = fenceRegex.exec(markdown)) !== null) {
        const info = (match[3] || "").trim();
        if (!info) continue;

        const tokens = info.split(/\s+/);
        const rawLang = tokens[0] || "";
        const flags = tokens.slice(1).map((flag) => flag.toLowerCase());

        const language = normalizeLanguage(rawLang);
        if (!language) continue;

        const hasRunSuffix = RUN_SUFFIX.test(rawLang);
        const hasRunFlag = flags.some((flag) => RUN_MARKERS.has(flag));
        if (!hasRunSuffix && !hasRunFlag) continue;

        if (seen.has(language)) continue;

        const code = (match[4] || "").trim();
        blocks.push({ language, code });
        seen.add(language);
    }

    return blocks;
}
