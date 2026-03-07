# 2026-03-07 oc learner, ops, and ai commands

## Summary
- Expanded `oc` to cover the remaining command-style HTTP APIs beyond admin/codelab/workspace basics.
- Enabled public `codelab list`, `codelab get`, and `codelab reference` without a saved session.
- Added attendee, help-request, feedback, materials, quiz, submission, chat, upload, inline comment, and AI conversation/thread commands.

## Added command groups
- `attendee`
  - `join`
  - `list`
  - `complete`
  - `certificate`
- `help`
  - `request`
  - `list`
  - `resolve`
- `feedback`
  - `submit`
  - `list`
- `materials`
  - `list`
  - `upload`
  - `add`
  - `delete`
- `quiz`
  - `list`
  - `update`
  - `submit`
  - `submissions`
- `submission`
  - `list`
  - `file`
  - `link`
  - `delete`
- `chat`
  - `history`
- `upload`
  - `image`
- `inline`
  - `list`
  - `create`
  - `reply`
  - `delete`
- `ai`
  - `conversations`
  - `stream`
  - `save`
  - `threads`
  - `thread-create`
  - `thread-delete`
  - `messages`
  - `message-add`

## Input conventions
- Complex writes use `--file <json>` and map directly to backend payloads.
- `submission file`, `submission link`, and `submission delete` accept `--attendee-id`, but also fall back to the current attendee session when available.
- `attendee join` stores the issued attendee session in the active session file, so the same `auth status` / session plumbing works for attendee flows too.

## Verification
- `cd backend && cargo fmt`
- `cd backend && cargo test --bin oc --no-run`
- `cd backend && cargo run --bin oc -- --help`
- `cd backend && cargo run --bin oc -- codelab list --json`
  - confirmed it reaches the HTTP request path without requiring a saved session
- `cd backend && cargo run --bin oc -- codelab reference`
- `cd backend && cargo run --bin oc -- ai stream --help`
