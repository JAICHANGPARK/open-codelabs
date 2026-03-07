# 2026-03-07 OC CLI Review Fixes

## Summary
- Fixed `oc down --volumes` so it removes the local bind-mounted data directories created by `oc run`.
- Fixed subcommand help handling so `oc <command> --help` prints usage and exits successfully instead of returning an error.

## Changes
- Stored `data_dir` in local stack runtime state so follow-up lifecycle commands can operate on the same local data path.
- Added local data cleanup for `data`, `uploads`, `workspaces`, and `postgres` directories when `oc down --volumes` is used.
- Updated `oc down --volumes` output to describe local data removal instead of generic Docker volumes removal.
- Replaced string-based help errors with a dedicated help sentinel so subcommand help is treated as a normal success path.
- Added topic-scoped help output for nested commands such as `oc connect add --help` and `oc help request --help`.

## Verification
- `cd backend && cargo fmt`
- `cd backend && cargo test cli::app --lib`
- `cd backend && cargo test --bin oc --no-run`
- `cd backend && cargo run --bin oc -- run --help`
- `cd backend && cargo run --bin oc -- connect add --help`
- `cd backend && cargo run --bin oc -- help request --help`
- `cd backend && cargo run --bin oc -- ps --json`
