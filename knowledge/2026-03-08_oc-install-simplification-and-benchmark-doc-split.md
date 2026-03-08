## Summary

- Simplified end-user installation guidance back to `cargo install --path backend --bin oc`.
- Re-centered release packaging and the Homebrew draft around the single `oc` binary.
- Moved benchmark usage out of the general CLI docs and into maintainer-oriented benchmark guides.

## Changes

### User-facing install docs

- Updated:
  - `README.md`
  - `README.ko.md`
  - `docs/getting-started/installation.md`
  - `docs/en/getting-started/installation.md`
- Removed companion benchmark binaries from the default install/build examples.
- Restored the local build examples to `cargo build --release --bin oc`.
- Kept uninstall guidance based on the Cargo package name while leaving the user-facing install path simple.

### Release packaging

- Updated `.github/workflows/oc-release.yml` to build and package only `oc` in the standard release archive flow.
- Updated `packaging/homebrew/oc.rb` to install only `oc`.

### Bench documentation split

- Removed detailed benchmark usage from the general CLI reference:
  - `docs/user-guide/cli.md`
  - `docs/en/user-guide/cli.md`
- Added maintainer-specific benchmark guides:
  - `docs/contributing/benchmarks.md`
  - `docs/en/contributing/benchmarks.md`
- Added benchmark guide navigation entries to `mkdocs.yml`.

## Verification

- `git diff --check`
- `ruby -c packaging/homebrew/oc.rb`
