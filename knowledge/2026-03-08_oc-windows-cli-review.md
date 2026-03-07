# 2026-03-08 oc Windows CLI review

## Summary

- Re-reviewed `oc` for Windows-specific risks in path handling, shell command rendering, browser launch behavior, and installation/removal docs.
- Hardened CLI path resolution and compose path generation for Windows-style paths.
- Updated install/remove docs to include Windows PowerShell examples.

## Code changes

### Shared home directory resolution

- Added `backend/src/cli/paths.rs`.
- Centralized home directory discovery for CLI modules.
- Added fallback support for `HOMEDRIVE` + `HOMEPATH`, which is common on Windows shells where `HOME` may not be present.
- Switched `config.rs`, `session.rs`, and `app.rs` to use the shared helper.

### Safer compose/runtime behavior on Windows

- Local stack compose generation now normalizes Windows host paths from `C:\...` to `C:/...` before embedding them in compose YAML.
- This reduces risk around bind mount parsing and keeps compose output more portable across Docker Desktop and Podman environments.

### Safer displayed commands

- `compose_command`, `logs_command`, and `stop_command` now quote arguments when paths contain spaces or shell-sensitive characters.
- This matters most on Windows where user profile paths commonly include spaces.

### Browser launch behavior

- Switched Windows browser launch from `rundll32 url.dll,FileProtocolHandler` to `cmd /C start "" <url>`.
- This aligns better with normal Windows shell behavior for opening the default browser.

## Documentation updates

- Added Windows PowerShell cleanup examples to:
  - `README.md`
  - `README.ko.md`
  - `docs/getting-started/installation.md`
  - `docs/en/getting-started/installation.md`
- Clarified manual binary removal on Windows with `oc.exe`.
- Added `where.exe oc` guidance for verifying removal on Windows.

## Verification

- `cargo fmt`
- `cargo test cli::app --lib`
- `cargo test cli::paths --lib`
- `cargo run --bin oc -- --help`

## Windows target compile note

- Installed the Rust target with `rustup target add x86_64-pc-windows-msvc`.
- Attempted `cargo check --target x86_64-pc-windows-msvc --bin oc`.
- The check did not complete on this macOS host because native Windows C toolchain headers are unavailable here (`aws-lc-sys` / `ring` fail while looking for standard Windows/MSVC headers such as `assert.h` and `stdio.h`).
- This limitation is in the host cross-compilation environment, not in the `oc` Rust source touched in this change.
