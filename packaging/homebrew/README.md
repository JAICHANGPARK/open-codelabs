# Homebrew draft for `oc`

This directory contains a draft Homebrew formula for the Open Codelabs CLI.

## Intended publication flow

1. Push an `oc-v<version>` tag.
2. Let `.github/workflows/oc-release.yml` publish release archives.
3. Compute SHA256 for each archive.
4. Copy `oc.rb` into a Homebrew tap repository.
5. Replace the draft version, URLs, and SHA256 placeholders.

## Expected release assets

- `oc-<version>-x86_64-apple-darwin.tar.gz`
- `oc-<version>-aarch64-apple-darwin.tar.gz`
- `oc-<version>-x86_64-unknown-linux-gnu.tar.gz`

## Notes

- Windows artifacts are released too, but Homebrew only needs macOS and Linux.
- The formula in this repository is intentionally a draft and is not expected to install until placeholders are replaced.
