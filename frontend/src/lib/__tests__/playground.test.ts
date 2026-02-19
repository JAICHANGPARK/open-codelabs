import { describe, expect, test } from "bun:test";
import { extractPlaygrounds } from "../playground";

describe("extractPlaygrounds", () => {
    test("returns empty array for empty input", () => {
        expect(extractPlaygrounds("")).toEqual([]);
    });

    test("extracts only runnable blocks with normalized languages and de-duplicates by language", () => {
        const markdown = [
            "```dart run",
            "print('first-dart');",
            "```",
            "",
            "```dart playground",
            "print('second-dart');",
            "```",
            "",
            "~~~golang runnable",
            "fmt.Println('go');",
            "~~~",
            "",
            "```py-run",
            "print('python');",
            "```",
            "",
            "```notebook-playground",
            "print('jupyter');",
            "```",
            "",
            "```python",
            "print('not-runnable');",
            "```",
            "",
            "```ruby run",
            "puts 'unsupported';",
            "```",
            "",
            "```",
            "no info",
            "```",
        ].join("\n");

        expect(extractPlaygrounds(markdown)).toEqual([
            { language: "dart", code: "print('first-dart');" },
            { language: "go", code: "fmt.Println('go');" },
            { language: "python", code: "print('python');" },
            { language: "jupyter", code: "print('jupyter');" },
        ]);
    });
});

