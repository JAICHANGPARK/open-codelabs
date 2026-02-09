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

// Pre-configured instances for different parts of the app
export const adminMarked = createMarkdownParser({ highlight: true });
export const attendeeMarked = createMarkdownParser({ highlight: true });
