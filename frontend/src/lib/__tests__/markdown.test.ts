import { describe, expect, test } from "bun:test";
import {
    adminMarked,
    attendeeMarked,
    createMarkdownParser,
    transformEmbeddableLinks,
} from "../markdown";

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

describe("transformEmbeddableLinks", () => {
    test("replaces youtube links with a privacy-enhanced embed", () => {
        const html = '<p><a href="https://youtu.be/dQw4w9WgXcQ">watch</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(transformed.includes("youtube-nocookie.com/embed/dQw4w9WgXcQ")).toBe(
            true
        );
        expect(transformed.includes("<iframe")).toBe(true);
    });

    test("replaces Google Slides links with an embed player", () => {
        const html =
            '<p><a href="https://docs.google.com/presentation/d/slide123/edit?usp=sharing">slides</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(
            transformed.includes(
                "https://docs.google.com/presentation/d/slide123/embed?start=false&loop=false&delayms=3000"
            )
        ).toBe(true);
        expect(transformed.includes('title="Google Slides"')).toBe(true);
    });

    test("supports published Google Slides links", () => {
        const html =
            '<p><a href="https://docs.google.com/presentation/d/e/2PACX-slide/pub?start=true&loop=true&delayms=5000">slides</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(
            transformed.includes(
                'src="https://docs.google.com/presentation/d/e/2PACX-slide/embed?start=true&loop=true&delayms=5000"'
            )
        ).toBe(true);
    });

    test("replaces Google Docs links with a preview iframe", () => {
        const html =
            '<p><a href="https://docs.google.com/document/d/doc123/edit?tab=t.0">doc</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(
            transformed.includes(
                'src="https://docs.google.com/document/d/doc123/preview"'
            )
        ).toBe(true);
        expect(transformed.includes('title="Google Docs"')).toBe(true);
    });

    test("supports published Google Docs links", () => {
        const html =
            '<p><a href="https://docs.google.com/document/d/e/2PACX-doc/pub">doc</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(
            transformed.includes(
                'src="https://docs.google.com/document/d/e/2PACX-doc/pub?embedded=true"'
            )
        ).toBe(true);
    });

    test("preserves selected sheet gid in spreadsheet previews", () => {
        const html =
            '<p><a href="https://docs.google.com/spreadsheets/d/sheet123/edit?gid=456#gid=456">sheet</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(
            transformed.includes(
                'src="https://docs.google.com/spreadsheets/d/sheet123/preview?gid=456"'
            )
        ).toBe(true);
        expect(transformed.includes('title="Google Sheets"')).toBe(true);
    });

    test("supports published Google Sheets links", () => {
        const html =
            '<p><a href="https://docs.google.com/spreadsheets/d/e/2PACX-sheet/pubhtml?widget=true&amp;headers=false">sheet</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(
            transformed.includes(
                'src="https://docs.google.com/spreadsheets/d/e/2PACX-sheet/pubhtml?widget=true&headers=false"'
            )
        ).toBe(true);
    });

    test("supports share links with user-scoped Google Docs paths", () => {
        const html =
            '<p><a href="https://docs.google.com/document/u/0/d/doc123/edit?usp=sharing">doc</a></p>';
        const transformed = transformEmbeddableLinks(html);

        expect(
            transformed.includes(
                'src="https://docs.google.com/document/d/doc123/preview"'
            )
        ).toBe(true);
    });

    test("leaves unrelated links unchanged", () => {
        const html = '<p><a href="https://example.com">example</a></p>';

        expect(transformEmbeddableLinks(html)).toBe(html);
    });

    test("leaves inline embeddable links as normal anchors", () => {
        const html =
            '<p>자료는 <a href="https://docs.google.com/document/d/doc123/edit">여기</a> 참고</p>';

        expect(transformEmbeddableLinks(html)).toBe(html);
    });
});
