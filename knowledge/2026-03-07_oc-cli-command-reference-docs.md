# 2026-03-07 OC CLI Command Reference Docs

## Summary
- Added dedicated CLI reference pages for Korean and English docs based on the current `oc --help` surface.
- Linked the new command reference from the installation guides, MkDocs navigation, and README entry points.

## Changes
- Added `docs/user-guide/cli.md` with grouped command coverage for connect/auth, local runtime, admin operations, attendee flows, uploads, and AI tools.
- Added `docs/en/user-guide/cli.md` with the matching English command reference.
- Added the CLI reference page to MkDocs navigation.
- Updated installation guides to point readers from setup instructions to the full CLI command reference.
- Updated repository docs notes and README entry points so users can find the new CLI reference quickly.

## Verification
- `cd /Users/jaichang/Documents/GitHub/open-codelabs && git diff --check`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs && cargo run --bin oc -- --help`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs && mkdocs build --strict`

## Notes
- `mkdocs build --strict` could not be executed in this environment because `mkdocs` is not installed.
