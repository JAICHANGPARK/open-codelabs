# 2026-03-07 oc installation docs

## Summary
- Added an `oc` CLI install and first-run section to the root README.
- Added matching CLI install sections to the Korean and English installation guides.
- Updated `docs/README.md` so maintainers know where CLI installation docs live.

## Changed files
- `README.md`
  - added a user-facing `CLI (oc)` section with source install, release build, and first connection examples
- `docs/getting-started/installation.md`
  - added Korean CLI installation steps
- `docs/en/getting-started/installation.md`
  - added English CLI installation steps
- `docs/README.md`
  - added a short note pointing maintainers to the CLI installation docs

## Verification
- `rg -n 'CLI \\(`oc`\\)|Install the CLI|CLI 설치 문서 위치|oc connect add --name local' README.md docs/README.md docs/getting-started/installation.md docs/en/getting-started/installation.md`
- `command -v mkdocs`
  - `mkdocs` is not installed in this environment, so `mkdocs build --strict` was not run
