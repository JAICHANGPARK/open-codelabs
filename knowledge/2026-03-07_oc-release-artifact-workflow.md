# 2026-03-07 oc release artifact workflow

## Summary
- Added a dedicated GitHub Actions workflow to build `oc` release archives.
- The workflow builds native binaries for Linux, macOS Intel, macOS Apple Silicon, and Windows.
- Tag pushes create GitHub Release assets; manual runs keep the packaged binaries as workflow artifacts only.

## Changed files
- `.github/workflows/oc-release.yml`
  - added `push.tags: oc-v*` and `workflow_dispatch`
  - added a multi-OS build matrix
  - added archive packaging with `README.md` and `LICENSE`
  - added GitHub Release publishing via `softprops/action-gh-release`

## Artifact naming
- `oc-<version>-x86_64-unknown-linux-gnu.tar.gz`
- `oc-<version>-x86_64-apple-darwin.tar.gz`
- `oc-<version>-aarch64-apple-darwin.tar.gz`
- `oc-<version>-x86_64-pc-windows-msvc.zip`

## Verification
- `sed -n '1,260p' .github/workflows/oc-release.yml`
- `rg -n 'oc-v\\*|dtolnay/rust-toolchain|upload-artifact|action-gh-release|x86_64-unknown-linux-gnu|aarch64-apple-darwin|x86_64-pc-windows-msvc' .github/workflows/oc-release.yml`

## Notes
- GitHub Actions was not executed from this local environment.
- The workflow is designed so `workflow_dispatch` produces downloadable artifacts without creating a GitHub Release.
