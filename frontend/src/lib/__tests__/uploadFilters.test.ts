import { describe, expect, test } from "bun:test";
import {
    getBlocklists,
    isEnvFile,
    isMediaFile,
    isBlockedByExt,
    isBlockedByPath,
    normalizePath,
    shouldSkipFile,
} from "../uploadFilters";

describe("uploadFilters", () => {
    test("skips env files", () => {
        expect(isEnvFile(".env")).toBe(true);
        expect(isEnvFile(".env.local")).toBe(true);
        expect(isEnvFile("configs/.env.prod")).toBe(true);
        expect(isEnvFile("README.md")).toBe(false);
    });

    test("skips media files", () => {
        expect(isMediaFile("video.mp4")).toBe(true);
        expect(isMediaFile("audio/mix.WAV")).toBe(true);
        expect(isMediaFile("code/main.rs")).toBe(false);
    });

    test("blocks known build paths and extensions", () => {
        expect(isBlockedByPath("project/build/index.js")).toBe(true);
        expect(isBlockedByPath("android/app/build/outputs.apk")).toBe(true);
        expect(isBlockedByPath("src/main.ts")).toBe(false);
        expect(isBlockedByExt("binary/a.out")).toBe(false);
        expect(isBlockedByExt("lib/some.dll")).toBe(true);
    });

    test("shouldSkipFile combines all rules", () => {
        expect(shouldSkipFile("node_modules/react/index.js")).toBe(true);
        expect(shouldSkipFile(".env")).toBe(true);
        expect(shouldSkipFile("assets/logo.png")).toBe(true);
        expect(shouldSkipFile("src/app.dart")).toBe(false);
    });

    test("normalizes path separators and casing", () => {
        expect(normalizePath("SRC\\Assets\\Image.PNG")).toBe("src/assets/image.png");
    });

    test("exposes blocklist collections", () => {
        const lists = getBlocklists();
        expect(lists.EXT_BLOCKLIST.has(".png")).toBe(true);
        expect(lists.PATH_BLOCKLIST.includes("node_modules/")).toBe(true);
        expect(lists.MEDIA_EXTENSIONS.has(".mp4")).toBe(true);
    });
});
