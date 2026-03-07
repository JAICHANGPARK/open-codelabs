# 2026-03-07 OC Interactive Onboarding

## Summary
- Added guided interactive onboarding to `oc` so first-run setup no longer depends on remembering long flag-heavy commands.

## Changes
- Added `oc init` as the main interactive entry point for first-time CLI users.
- `oc init` now guides users through either:
  - starting a local stack on the current machine, or
  - connecting to an existing Open Codelabs server
- After setup, `oc init` can save a connection profile and continue into browser-based admin authentication.
- Added interactive prompting to `oc run`:
  - running `oc run` with no flags in an interactive terminal now starts a setup wizard
  - `oc run --interactive` forces the same guided flow
- Added interactive prompting to `oc connect add`:
  - `oc connect add --interactive` prompts for URL, profile name, runtime, and activation
  - missing required fields in an interactive terminal also drop into the guided prompt flow
- Added profile-name suggestion from the URL for interactive connect setup.
- Updated CLI docs and installation docs to document `oc init` and the new interactive behavior.

## Verification
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/backend && cargo fmt`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/backend && cargo test cli::app --lib`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/backend && cargo test --bin oc --no-run`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/backend && cargo run --bin oc -- init --help`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/backend && cargo run --bin oc -- --help`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/backend && cargo run --bin oc -- connect add --help`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/backend && cargo run --bin oc -- --config-file /tmp/oc-interactive-smoke.json connect add --name smoke-local --url http://localhost:8080 --runtime backend --activate --json`

## Notes
- Full interactive wizards were not executed end-to-end in this environment because terminal automation here is non-interactive by default.
- Help, compile, parser, and non-interactive smoke paths were verified.
