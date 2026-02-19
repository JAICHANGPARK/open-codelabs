import { describe, expect, test } from "bun:test";
import { adminMarked, attendeeMarked, createMarkdownParser } from "../markdown";

async function parseToHtml(parser: { parse: (text: string) => string | Promise<string> }, input: string) {
    const parsed = parser.parse(input);
    return typeof parsed === "string" ? parsed : await parsed;
}

describe("createMarkdownParser", () => {
    test("renders gfm and katex without highlight option", async () => {
        const parser = createMarkdownParser();
        const html = await parseToHtml(parser as any, "**bold**\n\n$E=mc^2$");
        expect(html.includes("<strong>bold</strong>")).toBe(true);
        expect(html.includes("katex")).toBe(true);
    });

    test("highlights known language blocks when enabled", async () => {
        const parser = createMarkdownParser({ highlight: true });
        const html = await parseToHtml(
            parser as any,
            "```javascript\nconst answer = 42;\n```"
        );
        expect(html.includes("hljs")).toBe(true);
    });

    test("falls back to auto-highlight for unknown languages", async () => {
        const parser = createMarkdownParser({ highlight: true });
        const html = await parseToHtml(parser as any, "```unknownlang\nhello()\n```");
        expect(html.includes("hljs")).toBe(true);
    });

    test("pre-configured parser instances are usable", async () => {
        const adminHtml = await parseToHtml(adminMarked as any, "`admin`");
        const attendeeHtml = await parseToHtml(attendeeMarked as any, "`attendee`");
        expect(adminHtml.includes("<code>admin</code>")).toBe(true);
        expect(attendeeHtml.includes("<code>attendee</code>")).toBe(true);
    });
});

