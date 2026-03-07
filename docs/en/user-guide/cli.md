# CLI Reference

This page documents the currently supported Open Codelabs CLI surface based on `oc --help`.

`oc <command> --help` prints scoped usage for each command group.

## Global options

Every command supports the following global flags.

| Option | Description |
| --- | --- |
| `--base-url <url>` | Override the backend base URL directly. |
| `--session-file <path>` | Override the session file path. |
| `--config-file <path>` | Change the CLI config file path. The default is `~/.open-codelabs/config.json`. |
| `--profile <name>` | Select one of the saved connection profiles. |
| `--json` | Print machine-friendly JSON instead of text/table output. |
| `-h`, `--help` | Show help. |

## Install and remove

### Install

```bash
cargo install --path backend --bin oc
oc init
```

- `oc init` walks through local stack startup or existing-server connection with prompts.
- If you want to jump straight in, use `oc run --open` or `oc connect add --interactive`.

### Remove

```bash
oc down --volumes
cargo uninstall oc
rm -rf ~/.open-codelabs
```

- `cargo uninstall oc` removes the Cargo-installed binary.
- If you built or downloaded the binary manually, delete the copied `oc` binary from your `PATH`.
- The full removal matrix is documented in the [Installation Guide](../getting-started/installation.md).

## Recommended workflows

### 0. Interactive onboarding

```bash
oc init
```

- This is the easiest first-run entry point.
- It lets you choose between starting a local stack and connecting to an existing server.
- At the end, it can save a profile and continue into `oc auth login`.

### 1. Start a local stack with the CLI only

```bash
oc run --open
oc ps
oc logs --service backend --tail 200 --no-follow
```

### 2. Connect to an existing server

```bash
oc connect add --name local --url http://localhost:8080 --runtime backend --activate
oc connect status
```

### 3. Authenticate as an administrator

```bash
oc auth login
oc auth status
```

### 4. Start an attendee session

```bash
oc attendee join --codelab-id demo --name "Jane" --code ABC123
```

## Session and permission scopes

| Scope | How the session is created | Common commands |
| --- | --- | --- |
| Local runtime | No remote authentication required | `oc run`, `oc ps`, `oc logs`, `oc restart`, `oc down` |
| Public read | `oc connect` only | `oc codelab list`, `oc codelab reference`, `oc codelab get` |
| Admin | `oc auth login` | `admin`, `backup`, `audit`, `workspace`, `codelab create/update/delete/copy/export/import/push-steps` |
| Attendee | `oc attendee join` | `help request`, `feedback submit`, `quiz submit`, `submission file/link/delete`, `chat history` |

Final access control is enforced by the connected runtime and backend configuration.

## Connection and authentication

### Connection profiles

```bash
oc connect add --name <name> --url <url> [--runtime <auto|backend|firebase|supabase>] [--activate] [--interactive]
oc connect use --name <name>
oc connect list
oc connect status
```

- Use profiles to store and switch between multiple servers.
- `connect status` reports the current URL, runtime probe, and available capabilities.
- With `--interactive`, or when required fields are missing in a TTY, the CLI prompts for the profile values.

### Administrator authentication

```bash
oc auth login [--no-open]
oc auth logout
oc auth status
```

- `oc auth login` starts the browser-based admin sign-in flow.
- Session files are stored under `~/.open-codelabs/` by default.

### Legacy aliases

```bash
oc login --admin-id <id> --admin-pw <pw>
oc logout
oc session
```

- Prefer the `auth` command group for new scripts.

## Local stack runtime

```bash
oc run [--engine <auto|docker|podman>] [--postgres] [--pull] [--open] [--interactive] [--admin-id <id>] [--admin-pw <pw>] [--data-dir <path>] [--frontend-port <port>] [--backend-port <port>] [--image-registry <registry>] [--image-namespace <namespace>] [--image-tag <tag>]
oc ps [--service <name>]
oc logs [--service <name>] [--tail <n>] [--no-follow]
oc restart [--service <name>]
oc down [--volumes]
```

- `oc run` detects `docker` or `podman` and prints install/start guidance when the engine is missing or not running.
- `oc down --volumes` removes the local bind-mounted data directories created by `oc run`.
- Running `oc run` without flags in an interactive terminal starts the setup wizard automatically.

## Administrative commands

### Admin settings

```bash
oc admin settings [--gemini-api-key <key>] [--admin-password <pw>]
oc admin updates
```

### Codelabs

```bash
oc codelab list
oc codelab reference
oc codelab get --id <id>
oc codelab create --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
oc codelab update --id <id> --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
oc codelab delete --id <id>
oc codelab copy --id <id>
oc codelab export --id <id> [--output <path>]
oc codelab import --file <zip>
oc codelab push-steps --id <id> --file <json>
```

### Backups and audit logs

```bash
oc backup export [--output <path>]
oc backup inspect --file <zip>
oc backup restore --file <zip>
oc audit logs [--limit <n>] [--offset <n>] [--action <name>] [--codelab-id <id>]
```

### Workspaces

```bash
oc workspace create --codelab-id <id> [--structure-type <branch|folder>] [--files-json <path>]
oc workspace info --codelab-id <id>
oc workspace download --codelab-id <id> [--output <path>]
oc workspace delete --codelab-id <id>
oc workspace branches --codelab-id <id>
oc workspace branch-create --codelab-id <id> --step-number <n> [--branch-type <start|end>]
oc workspace branch-files --codelab-id <id> --branch <name>
oc workspace branch-read --codelab-id <id> --branch <name> --file <path>
oc workspace branch-update --codelab-id <id> --branch <name> --files-json <path> [--delete-json <path>] [--commit-message <message>]
oc workspace folders --codelab-id <id>
oc workspace folder-create --codelab-id <id> --step-number <n> --files-json <path> [--folder-type <start|end>]
oc workspace folder-files --codelab-id <id> --folder <name>
oc workspace folder-read --codelab-id <id> --folder <name> --file <path>
oc workspace folder-update --codelab-id <id> --folder <name> --files-json <path> [--delete-json <path>]
```

## Attendee operations and live tools

### Attendees

```bash
oc attendee join --codelab-id <id> --name <name> --code <code> [--email <email>]
oc attendee list --codelab-id <id>
oc attendee complete --codelab-id <id>
oc attendee certificate [--attendee-id <id>]
```

### Help queue and feedback

```bash
oc help request --codelab-id <id> --step-number <n>
oc help list --codelab-id <id>
oc help resolve --codelab-id <id> --help-id <id>
oc feedback submit --codelab-id <id> --difficulty <1-5> --satisfaction <1-5> [--comment <text>]
oc feedback list --codelab-id <id>
```

### Materials, quizzes, and submissions

```bash
oc materials list --codelab-id <id>
oc materials upload --file <path>
oc materials add --codelab-id <id> --title <title> --type <link|file> [--url <url>] [--file-path <path>]
oc materials delete --codelab-id <id> --material-id <id>
oc quiz list --codelab-id <id>
oc quiz update --codelab-id <id> --file <json>
oc quiz submit --codelab-id <id> --file <json>
oc quiz submissions --codelab-id <id>
oc submission list --codelab-id <id>
oc submission file --codelab-id <id> [--attendee-id <id>] --file <path>
oc submission link --codelab-id <id> [--attendee-id <id>] --url <url> [--title <title>]
oc submission delete --codelab-id <id> [--attendee-id <id>] --submission-id <id>
```

### Chat, uploads, and inline comments

```bash
oc chat history --codelab-id <id>
oc upload image --file <path>
oc inline list --codelab-id <id> [--target-type <guide|step>] [--target-step-id <id>]
oc inline create --codelab-id <id> --file <json>
oc inline reply --codelab-id <id> --thread-id <id> --file <json>
oc inline delete --codelab-id <id> --thread-id <id> --comment-id <id>
```

## AI commands

```bash
oc ai conversations --codelab-id <id>
oc ai stream --file <json>
oc ai save --file <json>
oc ai threads
oc ai thread-create --title <title> [--codelab-id <id>]
oc ai thread-delete --thread-id <id>
oc ai messages --thread-id <id>
oc ai message-add --thread-id <id> --file <json>
```

- `conversations` lists codelab-scoped AI conversations.
- `threads`, `messages`, and `message-add` manage persistent thread-based workflows.
- `stream` and `save` accept JSON payload files, which makes them useful for automation and scripted runs.

## Operational tips

- Use `oc <group> --help` for scoped option details.
- Add `--json` for CI, scripts, or machine-readable output.
- Use `oc connect add` together with `oc --profile <name>` when you work across multiple environments.
