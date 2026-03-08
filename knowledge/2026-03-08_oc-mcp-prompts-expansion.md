# 2026-03-08 oc MCP prompts expansion

## Goal

Close the remaining MVP gap in the `oc mcp serve` surface by adding reusable MCP prompts on top of the existing tools and resources.

## What changed

- Added MCP prompt support to the Open Codelabs MCP server.
- Enabled prompt capabilities in server metadata.
- Fixed MCP tool structured output so JSON-returning tools emit `{ "data": ... }` object payloads that satisfy MCP `outputSchema` requirements.
- Added four prompt templates:
  - `facilitator-brief`
  - `authoring-change-plan`
  - `help-queue-triage`
  - `learner-ops-review`
- Grounded prompts with existing MCP resource links so hosts can start from structured context instead of rebuilding the same instructions every time.
- Updated MCP and CLI documentation in both Korean and English.
- Added tests for prompt listing and prompt resource-link coverage.

## Why this matters

- MCP hosts now see a more complete MVP surface: tools, resources, and prompts.
- Repeated facilitator and authoring workflows become easier to start consistently.
- The server now gives higher-level workflow entry points without reducing the value of the lower-level tools.

## Verification

- `cargo fmt`
- `cargo check --bin oc`
- `cargo test mcp --lib`
- `cargo run --bin oc -- mcp serve --help`
- `git diff --check`
