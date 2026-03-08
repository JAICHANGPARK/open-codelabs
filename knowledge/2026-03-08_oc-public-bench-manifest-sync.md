## Summary

- Completed the missing `oc public` command family and finished the previously partial implementation so the CLI compiles again.
- Added `oc bench <local|ops|ws>` as a dispatcher that prefers companion binaries next to `oc` and falls back to `cargo run --release --bin ...` inside a source checkout.
- Expanded `oc codelab pull` and `oc codelab push` from metadata/guide/steps only to full manifest sync for quizzes and materials.
- Updated release packaging, Homebrew draft packaging, and install/reference docs so installed users can use `oc bench` and see the new `public` and manifest commands.

## Implementation details

### `oc public`

- Added CLI parsing and help output for:
  - `oc public up`
  - `oc public status`
  - `oc public down`
- Added persistent runtime state under `~/.open-codelabs/runtime/public/`.
- Implemented tunnel process launch, status persistence, URL discovery, PID checks, and stop logic.
- Supported tunnel providers:
  - `ngrok`
  - `bore`
  - `cloudflared`

### `oc bench`

- Added `bench` command parsing under `oc`.
- Added target selection for:
  - `local`
  - `ops`
  - `ws`
- Dispatch order:
  1. sibling benchmark binaries next to `oc`
  2. source-checkout fallback through `cargo run --release --bin ...`

### Manifest sync

- Added manifest support for:
  - quizzes
  - materials
- `oc codelab pull` now exports:
  - metadata
  - guide
  - steps
  - quizzes
  - uploaded materials
- `oc codelab push` now syncs:
  - metadata
  - guide
  - steps
  - quizzes
  - materials
- File materials are uploaded first and then re-created through the material API.
- Enabled empty quiz replacement on the backend so full sync can clear quizzes intentionally.

### Packaging and docs

- GitHub release workflow now packages:
  - `oc`
  - `local_bench`
  - `ops_bench`
  - `ws_bench`
- Homebrew draft formula installs the companion benchmark binaries too.
- Updated README and installation docs to use the multi-binary Cargo install/build flow and corrected Cargo uninstall guidance to use the package name.
- Updated CLI reference docs with:
  - public tunnel commands
  - bench dispatch command
  - richer `codelab push/pull` manifest description

## Verification

- `cargo fmt`
- `cargo test --bin oc --no-run`
- `cargo test cli::app --lib`
- `cargo run --bin oc -- public --help`
- `cargo run --bin oc -- bench --help`
- `cargo run --bin oc -- bench local -- --help`
- `cargo run --bin oc -- codelab pull --help`
- `git diff --check`
- `ruby -c packaging/homebrew/oc.rb`
