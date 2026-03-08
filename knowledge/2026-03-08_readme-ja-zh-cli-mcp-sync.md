# 2026-03-08 README ja zh CLI MCP sync

## Goal

Bring the Japanese and Chinese README files up to date with the current `oc` CLI and MCP messaging already reflected in the English and Korean README files.

## What changed

- Added `oc` quickstart installation snippets to `README.ja.md` and `README.zh.md`.
- Added dedicated CLI sections to both files.
- Documented:
  - `oc init`
  - `oc run --open`
  - `oc connect add` + `oc auth login`
  - `oc mcp serve`
- Added wording that the MCP server now exposes reusable facilitator/authoring prompt templates in addition to codelab data.

## Why this matters

- The localized README files were lagging behind the actual CLI surface.
- Users reading Japanese or Chinese documentation now get the same top-level onboarding path as English and Korean readers.

## Verification

- `git diff --check`
