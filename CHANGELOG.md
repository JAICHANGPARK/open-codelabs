# Changelog

All notable changes to Open Codelabs will be documented in this file.

## [2026.8.11] - 2026-08-11

### 🚀 Highlights
- Updated default Gemini AI model across frontend, backend, and documentation to **Gemini 3.6 Flash** (`gemini-3.6-flash`).
- Upgraded all Frontend dependencies (SvelteKit 2.70, Svelte 5.56, Vite 8.2, Tailwind 4.3, Supabase JS 2.112, Firebase JS 12.17).
- Upgraded all Backend dependencies (`tokio`, `sqlx`, `axum`, `tower-http`, `tracing`, `reqwest`, `bollard`, `zip`).

### 🛠️ Backend
- Updated default AI handler model fallback to `gemini-3.6-flash`.
- Added validation check to `update_quizzes` handler preventing empty quiz payload processing (returns 400 Bad Request).
- Fixed test environment variable isolation (`EnvRestore`) and HTTP status assertions in API integration test suites.
- Synced `Cargo.toml` & `Cargo.lock` version to `2026.8.11`.

### 🎨 Frontend
- Updated default model in Gemini client (`src/lib/gemini.ts`), `AskGemini.svelte`, `AiCodelabGenerator.svelte`, `FacilitatorConsultant.svelte`, `WorkspaceMode.svelte`, and `admin/[id]/+page.svelte`.
- Verified type check (`bun run check`), build (`bun run build`), and test suite (`bun test`).
- Synced `package.json` & `package-lock.json` version to `2026.8.11`.

### 📚 Documentation & Tooling
- Updated `.env.sample` and self-hosting / API reference docs to reflect `gemini-3.6-flash`.
- Updated `VERSION` file to `2026.8.11`.
