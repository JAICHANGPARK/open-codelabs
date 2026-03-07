# 2026-03-08 oc CLI TUI upgrade

## Summary

- Replaced the hand-rolled `read_line` prompt flow in `oc` with `dialoguer`.
- Upgraded `oc init`, `oc run`, `oc connect add`, `oc connect use`, `oc auth login`, and legacy `oc login` to support richer terminal interaction.
- Updated README and MkDocs pages so the documented CLI behavior matches the new interaction model.

## What changed

### Interactive runtime setup

- Added `dialoguer = "0.12"` to the backend CLI dependencies.
- Switched prompt helpers in `backend/src/cli/app.rs` to `Input`, `Select`, `MultiSelect`, and `Password`.
- `oc run` now uses:
  - an engine picker
  - a startup options multi-select
  - a settings review multi-select
  - hidden password entry when credentials are edited

### Guided connection and auth

- `oc init` now uses multi-select follow-up steps instead of a chain of yes/no prompts.
- `oc connect add` now uses the same TUI flow for URL, profile name, runtime, and activation.
- `oc connect use` can open a saved-profile picker when `--name` is omitted in a TTY.
- `oc auth login` now offers an interactive choice between auto-opening the browser and printing the login URL.
- Legacy `oc login` now supports interactive ID/password collection with hidden password input.

### Documentation sync

- Updated `README.md` and `README.ko.md` to mention arrow-key menus and multi-select behavior.
- Updated installation and quickstart guides in both languages.
- Updated the CLI reference in both languages to describe the new TUI behavior and the new interactive semantics of `connect use`, `auth login`, and `login`.

## Verification

- `cargo fmt`
- `cargo test cli::app --lib`
- `cargo test --bin oc --no-run`
- `cargo run --bin oc -- --help`
- `cargo run --bin oc -- auth login --help`
- `cargo run --bin oc -- connect use --help`
