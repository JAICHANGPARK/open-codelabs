# 2026-03-07 oc workspace file commands

## Summary
- Added branch-level workspace file commands to `oc`.
- Added folder-level workspace file commands to `oc`.
- Extended workspace JSON loading so file updates can be supplied either as a plain file array or as a full update payload.

## Added commands
- `oc workspace branch-files --codelab-id <id> --branch <name>`
- `oc workspace branch-read --codelab-id <id> --branch <name> --file <path>`
- `oc workspace branch-update --codelab-id <id> --branch <name> --files-json <path> [--delete-json <path>] [--commit-message <message>]`
- `oc workspace folder-files --codelab-id <id> --folder <name>`
- `oc workspace folder-read --codelab-id <id> --folder <name> --file <path>`
- `oc workspace folder-update --codelab-id <id> --folder <name> --files-json <path> [--delete-json <path>]`

## Input formats
- `--files-json` accepts either:
  - `[{ "path": "src/main.rs", "content": "fn main() {}" }]`
  - `{ "files": [...], "delete_files": ["old.txt"], "commit_message": "Update files" }`
- `--delete-json` accepts either:
  - `["old.txt", "tmp/data.json"]`
  - `{ "delete_files": ["old.txt", "tmp/data.json"] }`

## Verification
- `cd backend && cargo fmt`
- `cd backend && cargo test --bin oc --no-run`
- `cd backend && cargo run --bin oc -- workspace branch-update --help`
