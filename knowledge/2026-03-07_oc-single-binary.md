# 2026-03-07 oc single binary

## Summary
- Removed the legacy CLI alias binary.
- Standardized the repository on `oc` as the only CLI executable name.
- Updated user-facing docs and build stubs so new installs only expose `oc`.

## Changed files
- `backend/Cargo.toml`
  - removed the legacy alias bin target
- `backend/src/bin`
  - deleted the legacy wrapper binary and kept only `oc.rs`
- `backend/Dockerfile`
  - replaced the cache-priming dummy bin with `src/bin/oc.rs`
- `backend/src/cli/client.rs`
  - updated the top-level CLI description to `oc`
- `docs/specification/cli-proposal.md`
  - updated CLI naming guidance and examples to `oc`
- `knowledge/2026-03-07_oc-cli-bootstrap.md`
  - aligned historical notes with the single-binary state
- `knowledge/2026-03-07_oc-browser-auth.md`
  - removed outdated legacy verification lines

## Verification
- `cd backend && cargo fmt`
- `cd backend && cargo test --bin oc --no-run`
- `cd backend && cargo run --bin oc -- --help`
- `cd backend && rg -n 'legacy alias bin target|legacy wrapper binary' ../knowledge/2026-03-07_oc-single-binary.md`
  - kept the migration note in this work log only, while the active code and user-facing docs now use `oc` exclusively
