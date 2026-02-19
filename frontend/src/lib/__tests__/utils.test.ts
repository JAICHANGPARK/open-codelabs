import { describe, expect, test } from "bun:test";
import { cn } from "../utils";

describe("cn", () => {
    test("merges class names and resolves tailwind conflicts", () => {
        const value = cn("px-2", "px-4", "text-sm", { block: true, hidden: false });
        expect(value.includes("px-4")).toBe(true);
        expect(value.includes("px-2")).toBe(false);
        expect(value.includes("text-sm")).toBe(true);
        expect(value.includes("block")).toBe(true);
    });
});

