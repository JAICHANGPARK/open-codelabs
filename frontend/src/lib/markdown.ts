import { Marked } from "marked";
import { markedHighlight } from "marked-highlight";
import hljs from "highlight.js";
import markedKatex from "marked-katex-extension";

/**
 * Creates a configured Marked instance.
 * @param options Configuration options
 */
export function createMarkdownParser(options: { highlight?: boolean } = {}) {
    const marked = new Marked();

    if (options.highlight) {
        marked.use(
            markedHighlight({
                emptyLangClass: "hljs",
                langPrefix: "hljs language-",
                highlight(code, lang) {
                    const normalized = (lang || "").trim().toLowerCase();
                    if (normalized && hljs.getLanguage(normalized)) {
                        return hljs.highlight(code, { language: normalized }).value;
                    }
                    return hljs.highlightAuto(code).value;
                },
            })
        );
    }

    marked.use({
        gfm: true,
        breaks: true,
    });

    marked.use(markedKatex({
        throwOnError: false
    }));

    return marked;
}

type EmbedDescriptor = {
    src: string;
    title: string;
    variant: "wide" | "document";
    allow?: string;
    allowFullscreen?: boolean;
};

const paragraphAnchorRegex =
    /<p>\s*(<a\b[^>]*href="([^"]+)"[^>]*>.*?<\/a>)\s*<\/p>/gi;

type GoogleWorkspaceKind = "document" | "presentation" | "spreadsheets";
type GoogleWorkspaceTarget = {
    kind: GoogleWorkspaceKind;
    id: string;
    published: boolean;
    action: string | null;
    url: URL;
};

function normalizeHost(hostname: string) {
    return hostname.replace(/^www\./, "").toLowerCase();
}

function decodeHtmlUrl(rawUrl: string) {
    return rawUrl.replace(/&amp;/gi, "&");
}

function getGoogleWorkspaceTarget(rawUrl: string): GoogleWorkspaceTarget | null {
    try {
        const url = new URL(decodeHtmlUrl(rawUrl));
        if (normalizeHost(url.hostname) !== "docs.google.com") {
            return null;
        }

        const segments = url.pathname.split("/").filter(Boolean);
        const kind = segments[0];
        if (
            kind !== "document" &&
            kind !== "presentation" &&
            kind !== "spreadsheets"
        ) {
            return null;
        }

        const dIndex = segments.indexOf("d");
        if (dIndex < 0) {
            return null;
        }

        const identifier = segments[dIndex + 1];
        if (!identifier) {
            return null;
        }

        if (identifier === "e") {
            const publishedId = segments[dIndex + 2];
            if (!publishedId) {
                return null;
            }

            return {
                kind,
                id: publishedId,
                published: true,
                action: segments[dIndex + 3] ?? null,
                url,
            };
        }

        return {
            kind,
            id: identifier,
            published: false,
            action: segments[dIndex + 2] ?? null,
            url,
        };
    } catch {
        return null;
    }
}

function extractYoutubeId(rawUrl: string): string | null {
    try {
        const url = new URL(decodeHtmlUrl(rawUrl));
        const host = normalizeHost(url.hostname);
        if (host === "youtu.be") {
            return url.pathname.replace("/", "").split("/")[0] || null;
        }
        if (
            host === "youtube.com" ||
            host.endsWith(".youtube.com") ||
            host === "youtube-nocookie.com"
        ) {
            if (url.pathname === "/watch") {
                return url.searchParams.get("v");
            }
            if (url.pathname.startsWith("/embed/")) {
                return url.pathname.split("/")[2] || null;
            }
            if (url.pathname.startsWith("/shorts/")) {
                return url.pathname.split("/")[2] || null;
            }
        }
    } catch {
        // ignore invalid urls
    }
    return null;
}

function getGoogleWorkspaceEmbed(rawUrl: string): EmbedDescriptor | null {
    const target = getGoogleWorkspaceTarget(rawUrl);
    if (!target) {
        return null;
    }

    const gid =
        target.url.searchParams.get("gid") ??
        (target.url.hash.startsWith("#gid=") ? target.url.hash.slice(5) : null);

    if (target.published) {
        if (target.kind === "presentation") {
            const embedUrl = new URL(
                `https://docs.google.com/presentation/d/e/${target.id}/embed`
            );
            embedUrl.search = target.url.search;
            if (!embedUrl.searchParams.has("start")) {
                embedUrl.searchParams.set("start", "false");
            }
            if (!embedUrl.searchParams.has("loop")) {
                embedUrl.searchParams.set("loop", "false");
            }
            if (!embedUrl.searchParams.has("delayms")) {
                embedUrl.searchParams.set("delayms", "3000");
            }
            return {
                src: embedUrl.toString(),
                title: "Google Slides",
                variant: "wide",
                allow: "autoplay",
                allowFullscreen: true,
            };
        }

        if (target.kind === "document") {
            const embedUrl = new URL(
                `https://docs.google.com/document/d/e/${target.id}/pub`
            );
            embedUrl.search = target.url.search;
            if (!embedUrl.searchParams.has("embedded")) {
                embedUrl.searchParams.set("embedded", "true");
            }
            return {
                src: embedUrl.toString(),
                title: "Google Docs",
                variant: "document",
            };
        }

        return {
            src: target.url.toString(),
            title: "Google Sheets",
            variant: "document",
        };
    }

    if (target.kind === "presentation") {
        const embedUrl = new URL(
            `https://docs.google.com/presentation/d/${target.id}/embed`
        );
        embedUrl.searchParams.set("start", "false");
        embedUrl.searchParams.set("loop", "false");
        embedUrl.searchParams.set("delayms", "3000");
        return {
            src: embedUrl.toString(),
            title: "Google Slides",
            variant: "wide",
            allow: "autoplay",
            allowFullscreen: true,
        };
    }

    if (target.kind === "document") {
        return {
            src: `https://docs.google.com/document/d/${target.id}/preview`,
            title: "Google Docs",
            variant: "document",
        };
    }

    const embedUrl = new URL(
        `https://docs.google.com/spreadsheets/d/${target.id}/preview`
    );
    if (gid) {
        embedUrl.searchParams.set("gid", gid);
    }
    return {
        src: embedUrl.toString(),
        title: "Google Sheets",
        variant: "document",
    };
}

function resolveEmbed(rawUrl: string): EmbedDescriptor | null {
    const youtubeId = extractYoutubeId(rawUrl);
    if (youtubeId) {
        return {
            src: `https://www.youtube-nocookie.com/embed/${youtubeId}`,
            title: "YouTube video",
            variant: "wide",
            allow:
                "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture",
            allowFullscreen: true,
        };
    }

    return getGoogleWorkspaceEmbed(rawUrl);
}

function buildWideEmbedMarkup(embed: EmbedDescriptor) {
    const attributes = [
        `src="${embed.src}"`,
        `title="${embed.title}"`,
        'loading="lazy"',
        'style="width:100%;height:auto;aspect-ratio:16/9;display:block;border:0;border-radius:16px;background:#000;"',
        'referrerpolicy="strict-origin-when-cross-origin"',
    ];

    if (embed.allow) {
        attributes.push(`allow="${embed.allow}"`);
    }

    if (embed.allowFullscreen) {
        attributes.push("allowfullscreen");
    }

    return `
<div class="link-embed link-embed--wide" style="width:100%;max-width:100%;display:block;margin:1.25rem 0;">
  <iframe
    ${attributes.join("\n    ")}
  ></iframe>
</div>`;
}

function buildDocumentEmbedMarkup(embed: EmbedDescriptor) {
    return `
<div class="link-embed link-embed--document" style="width:100%;max-width:100%;display:block;margin:1.25rem 0;">
  <iframe
    src="${embed.src}"
    title="${embed.title}"
    loading="lazy"
    style="width:100%;height:clamp(420px,72vh,960px);display:block;border:1px solid rgba(15,23,42,0.12);border-radius:16px;background:#fff;"
    referrerpolicy="strict-origin-when-cross-origin"
  ></iframe>
</div>`;
}

function buildEmbedMarkup(rawUrl: string) {
    const embed = resolveEmbed(rawUrl);
    if (!embed) {
        return null;
    }

    return embed.variant === "document"
        ? buildDocumentEmbedMarkup(embed)
        : buildWideEmbedMarkup(embed);
}

export function transformEmbeddableLinks(html: string) {
    const replaceParagraphAnchor = (
        full: string,
        _anchor: string,
        href: string
    ) => buildEmbedMarkup(href) ?? full;

    return html.replace(paragraphAnchorRegex, replaceParagraphAnchor);
}

// Pre-configured instances for different parts of the app
export const adminMarked = createMarkdownParser({ highlight: true });
export const attendeeMarked = createMarkdownParser({ highlight: true });
