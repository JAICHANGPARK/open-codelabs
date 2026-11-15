import adapter from "@sveltejs/adapter-bun";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig, loadEnv } from "vite";
import tailwindcss from "@tailwindcss/vite";

const rootDir = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, process.cwd(), "");
	const apiModule =
		env.VITE_USE_SUPABASE === "true"
			? "src/lib/api-runtime-supabase.ts"
			: env.VITE_USE_FIREBASE === "true"
				? "src/lib/api-runtime-firebase.ts"
				: "src/lib/api-runtime-backend.ts";
	const apiEntry = path.resolve(rootDir, "src/lib/api.ts");
	const apiRuntime = path.resolve(rootDir, apiModule);
	const normalizePath = (value: string) =>
		value.split("?")[0].split(path.sep).join("/");

	return {
		plugins: [
			{
				name: "select-api-runtime",
				enforce: "pre",
				resolveId(source) {
					if (
						source === "#lib/api.ts" ||
						source === "#lib/api.js" ||
						source === "#lib/api" ||
						source === "$lib/api"
					) {
						return apiRuntime;
					}

					const normalizedSource = normalizePath(source);
					if (
						normalizedSource === normalizePath(apiEntry) ||
						normalizedSource.endsWith("/src/lib/api.ts")
					) {
						return apiRuntime;
					}

					return null;
				},
				load(id) {
					if (normalizePath(id) !== normalizePath(apiEntry)) {
						return null;
					}

					return `export * from ${JSON.stringify(apiRuntime)};`;
				}
			},
			tailwindcss(),
			sveltekit({
				preprocess: vitePreprocess(),
				csrf: {
					trustedOrigins: [
						"http://localhost:5173",
						"http://127.0.0.1:5173"
					]
				},
				adapter: adapter({
					serverOptions: {
						maxRequestBodySize: 1024 * 1024 * 100
					}
				})
			})
		],
		build: {
			reportCompressedSize: false
		}
	};
});
