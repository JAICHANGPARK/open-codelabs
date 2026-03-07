# 2026-03-07 oc homebrew formula draft

## Summary
- Added a draft Homebrew formula for the `oc` CLI.
- Added a small packaging note that explains how to turn the draft into a published tap formula after a release.

## Changed files
- `packaging/homebrew/oc.rb`
  - added a draft formula for macOS Intel, macOS Apple Silicon, and Linux
  - left version, URLs, and checksums as explicit placeholders
- `packaging/homebrew/README.md`
  - documented the intended tag -> release -> checksum -> tap publication flow

## Verification
- `ruby -c packaging/homebrew/oc.rb`
- `sed -n '1,220p' packaging/homebrew/oc.rb`
- `sed -n '1,220p' packaging/homebrew/README.md`
- `rg -n 'REPLACE_WITH_|oc-v<version>|oc-v0.0.0|x86_64-unknown-linux-gnu' packaging/homebrew`

## Notes
- This is intentionally a draft and is not yet suitable for `brew install` until the placeholders are replaced with a real release version and checksums.
