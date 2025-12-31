const EXT_BLOCKLIST = new Set([
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".svg",
    ".ico",
    ".pdf",
    ".zip",
    ".tar",
    ".gz",
    ".mp3",
    ".mp4",
    ".apk",
    ".aab",
    ".ipa",
    ".exe",
    ".dll",
    ".so",
    ".dylib",
    ".o",
    ".class",
    ".jar",
    ".aar",
    ".woff",
    ".woff2",
    ".ttf",
    ".eot",
]);

const PATH_BLOCKLIST = [
    "node_modules/",
    ".git/",
    "dist/",
    "build/",
    "out/",
    "bin/",
    "obj/",
    "app/build/",
    "android/app/build/",
    "ios/build/",
    ".dart_tool/",
    ".pub-cache/",
    ".flutter-plugins",
    ".flutter-plugins-dependencies",
    ".fvm/",
    ".gradle/",
    ".idea/",
    ".svelte-kit/",
    "target/",
    "venv/",
];

const MEDIA_EXTENSIONS = new Set([
    ".mp3",
    ".wav",
    ".aac",
    ".flac",
    ".ogg",
    ".m4a",
    ".mp4",
    ".mov",
    ".avi",
    ".mkv",
    ".webm",
]);

const ENV_PATTERNS = [/^\.env(\..+)?$/];

export function normalizePath(path: string): string {
    return path.toLowerCase().replace(/\\/g, "/");
}

export function isEnvFile(name: string): boolean {
    const normalized = normalizePath(name).split("/").pop() || "";
    return ENV_PATTERNS.some((re) => re.test(normalized));
}

export function isMediaFile(name: string): boolean {
    const lower = normalizePath(name);
    return Array.from(MEDIA_EXTENSIONS).some((ext) => lower.endsWith(ext));
}

export function isBlockedByPath(path: string): boolean {
    const lower = normalizePath(path);
    return PATH_BLOCKLIST.some((p) => lower.includes(p));
}

export function isBlockedByExt(path: string): boolean {
    const lower = normalizePath(path);
    return Array.from(EXT_BLOCKLIST).some((ext) => lower.endsWith(ext));
}

export function shouldSkipFile(path: string): boolean {
    return isEnvFile(path) || isMediaFile(path) || isBlockedByPath(path) || isBlockedByExt(path);
}

export function getBlocklists() {
    return { EXT_BLOCKLIST, PATH_BLOCKLIST, MEDIA_EXTENSIONS };
}
