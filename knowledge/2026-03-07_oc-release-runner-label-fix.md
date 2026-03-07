# 2026-03-07 OC Release Runner Label Fix

## Summary
- Fixed the `oc` release workflow after GitHub Actions macOS 13 runner support was removed.

## Changes
- Updated the Intel macOS release job from `macos-13` to `macos-15-intel`.
- Updated the Apple Silicon macOS release job from `macos-14` to `macos-15`.
- Kept the existing release targets and archive names unchanged.

## Why
- GitHub-hosted runner support for `macos-13` has been retired, which caused the `x86_64-apple-darwin` build job to fail with an unsupported configuration error.

## Verification
- `cd /Users/jaichang/Documents/GitHub/open-codelabs && git diff --check`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs && ruby -e 'require "yaml"; YAML.load_file(".github/workflows/oc-release.yml"); puts "yaml ok"'`
