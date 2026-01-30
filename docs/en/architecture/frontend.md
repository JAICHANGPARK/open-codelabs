# Frontend Architecture

This document describes the SvelteKit 5 frontend architecture.

## Directory structure

```
frontend/src/
|-- routes/                  # page routes
|   |-- +layout.svelte       # root layout
|   |-- +page.svelte         # home
|   |-- admin/               # admin
|   |   |-- +page.svelte
|   |   |-- [id]/+page.svelte
|   |   `-- audit-logs/+page.svelte
|   |-- codelabs/            # codelab views
|   |   |-- +page.svelte
|   |   `-- [id]/+page.svelte
|   |-- codelabs/[id]/entry/+page.svelte
|   |-- codelabs/[id]/live/+page.svelte
|   |-- certificate/[id]/+page.svelte
|   |-- verify/[id]/+page.svelte
|   `-- login/+page.svelte   # login
|-- lib/                     # libraries
|   |-- api.ts               # API router
|   |-- api-backend.ts       # backend API
|   |-- api-firebase.ts      # Firebase API
|   |-- api-supabase.ts      # Supabase API
|   |-- components/          # shared components
|   |-- i18n/                # i18n resources
|   |-- types.ts             # shared types
|   |-- markdown.ts          # Markdown utilities
|   |-- crypto.ts            # crypto utilities
|   |-- gemini.ts            # AI integration
|   |-- tts.ts               # TTS
|   |-- playground.ts        # playground
|   |-- uploadFilters.ts     # upload filters
|   |-- theme.svelte.ts      # theme state
|   `-- utils.ts             # utilities
|-- hooks.server.ts          # server hooks (proxy)
|-- app.css                  # global styles
`-- app.html                 # HTML template
```

## Key functionality

### 1. Routing
File-based routing (SvelteKit).

### 2. State management
Svelte stores and context.

### 3. API communication
Fetch API wrapper.

### 4. Real-time updates
WebSocket client.

## Next steps

- [Frontend code examples](../code-guide/frontend-examples.md)
- [API usage](../code-guide/api-usage.md)
