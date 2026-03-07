#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const versionFile = path.join(repoRoot, "VERSION");
const requestedVersion = process.argv[2];
const version = (requestedVersion ?? readFileSync(versionFile, "utf8")).trim();

validateVersion(version);

writeFileSync(versionFile, `${version}\n`);
updateJsonFile(path.join(repoRoot, "frontend", "package.json"), (pkg) => {
	pkg.version = version;
});
updateJsonFile(path.join(repoRoot, "frontend", "package-lock.json"), (lockfile) => {
	lockfile.version = version;
	if (lockfile.packages?.[""]) {
		lockfile.packages[""].version = version;
	}
});
updateTextFile(path.join(repoRoot, "backend", "Cargo.toml"), (contents) =>
	replaceOnce(
		contents,
		/(\[package\][\s\S]*?^version = ")([^"]+)(")$/m,
		`$1${version}$3`,
		"backend/Cargo.toml package version",
	),
);
updateTextFile(path.join(repoRoot, "backend", "Cargo.lock"), (contents) =>
	replaceOnce(
		contents,
		/(\[\[package\]\]\nname = "backend"\nversion = ")([^"]+)(")/,
		`$1${version}$3`,
		"backend/Cargo.lock root package version",
	),
);

console.log(`Synced repository version to ${version}`);

function validateVersion(value) {
	const match = value.match(/^(\d{4})\.(\d{1,2})\.(\d{1,2})$/);
	if (!match) {
		throw new Error(
			`Invalid version "${value}". Expected date-based format YYYY.M.D, for example 2026.3.7.`,
		);
	}

	const month = Number(match[2]);
	const day = Number(match[3]);

	if (month < 1 || month > 12) {
		throw new Error(`Invalid month "${match[2]}". Expected a value between 1 and 12.`);
	}

	if (day < 1 || day > 31) {
		throw new Error(`Invalid day "${match[3]}". Expected a value between 1 and 31.`);
	}
}

function updateJsonFile(filePath, mutate) {
	const parsed = JSON.parse(readFileSync(filePath, "utf8"));
	mutate(parsed);
	writeFileSync(filePath, `${JSON.stringify(parsed, null, "\t")}\n`);
}

function updateTextFile(filePath, transform) {
	const current = readFileSync(filePath, "utf8");
	const next = transform(current);
	writeFileSync(filePath, next);
}

function replaceOnce(contents, pattern, replacement, label) {
	if (!pattern.test(contents)) {
		throw new Error(`Could not update ${label}.`);
	}
	return contents.replace(pattern, replacement);
}
