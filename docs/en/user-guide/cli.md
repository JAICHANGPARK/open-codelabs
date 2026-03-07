# CLI Reference

This page explains the current `oc` CLI in detail: what each command does, what every option means, which role can run it, and what kind of input files the JSON-based commands expect.

If you only need the raw syntax, use `oc --help` or `oc <group> --help`. If you need to understand which flags to use and why, use this document as the primary reference.

## Core concepts

| Concept | Meaning |
| --- | --- |
| Profile | A saved server target created with `oc connect add`. It stores a name, base URL, and runtime preference in `~/.open-codelabs/config.json`. |
| Session | The authentication state issued by `oc auth login` or `oc attendee join`. It stores cookies, CSRF state, subject, role, and expiry in a session file. |
| Runtime | The server type the CLI is talking to. `backend` means the full Rust API. `firebase` and `supabase` imply limited CLI capabilities. |
| Local stack | The local frontend/backend container stack started by `oc run` through Docker or Podman. |
| JSON output | Add `--json` to get machine-friendly JSON instead of human-oriented text output. |

## Global options

Every command supports the following global flags.

| Option | Meaning | Typical use |
| --- | --- | --- |
| `--base-url <url>` | Overrides the backend base URL for the current invocation only. | Use it when you want to hit an ad-hoc server without saving a profile first. |
| `--session-file <path>` | Overrides the session file path for the current invocation. | Useful when you want separate admin and attendee sessions. |
| `--config-file <path>` | Overrides the CLI config location. The default is `~/.open-codelabs/config.json`. | Useful when you want isolated profile sets per project or environment. |
| `--profile <name>` | Forces the CLI to use one saved profile. | Useful when the current profile is not the one you want for this command. |
| `--json` | Prints JSON instead of text/table output. | Use it in automation, CI, or scripts. |
| `-h`, `--help` | Prints help. | Use it to inspect the scoped usage for one command or command group. |

## Value precedence

The CLI resolves base URLs and session files in a fixed order.

### Base URL precedence

1. `--base-url`
2. The selected profile, including `--profile`
3. The stored session's `base_url`
4. `OPEN_CODELABS_BASE_URL`
5. `http://localhost:8080`

### Session file precedence

1. `--session-file`
2. The profile-specific session file
3. The default session file `~/.open-codelabs/session.json`

## Environment variables and storage paths

| Item | Default | Meaning |
| --- | --- | --- |
| `OPEN_CODELABS_BASE_URL` | unset | Default server URL when no profile is active. |
| `OPEN_CODELABS_PROFILE` | unset | Default profile name to use. |
| `OPEN_CODELABS_CONFIG_FILE` | `~/.open-codelabs/config.json` | Overrides the profile config location. |
| `OPEN_CODELABS_SESSION_FILE` | `~/.open-codelabs/session.json` | Overrides the non-profile session location. |
| `OPEN_CODELABS_ADMIN_ID` | unset | Used by legacy `oc login` and as a default in `oc run`. |
| `OPEN_CODELABS_ADMIN_PW` | unset | Used by legacy `oc login`, `oc run`, and `oc admin settings` encryption. |
| `~/.open-codelabs/config.json` | CLI default | Stores connection profiles. |
| `~/.open-codelabs/profiles/<name>/session.json` | when using profiles | Stores the session for one saved profile. |
| `~/.open-codelabs/runtime/local-stack/` | when using `oc run` | Stores the generated compose file and local runtime state. |

## Interactive behavior

The CLI now supports `dialoguer`-based terminal interaction. Use the arrow keys to move between options, use Space to toggle items in multi-select screens, press Enter to continue, type directly into text inputs, and rely on hidden prompts for passwords.

| Command | Interactive behavior |
| --- | --- |
| `oc init` | Always requires an interactive terminal. It guides you through the startup mode, the `oc run` wizard, profile saving, and browser auth handoff. |
| `oc run` | When run without flags in a TTY, it opens the local stack wizard. After choosing the engine, you toggle startup options and the settings you want to review. `--interactive` forces the wizard even if some flags are already set. |
| `oc connect add` | With `--interactive`, or when required values are missing in a TTY, it prompts for the URL, profile name, runtime, and activation behavior. |
| `oc connect use` | In a TTY, if you omit `--name`, the CLI shows the saved profiles and lets you pick one interactively. |
| `oc auth login` | In a TTY, running it without flags opens a choice between automatically opening the browser and only printing the login URL. |
| `oc login` | With `--interactive`, or when required values are missing in a TTY, it prompts for the admin ID and password. The password input is hidden. |
| Most other commands | Non-interactive. Required values must be provided as flags or input files. |

## Permission scopes

| Scope | How the session is created | Common commands |
| --- | --- | --- |
| Local runtime | No remote authentication required | `oc run`, `oc ps`, `oc logs`, `oc restart`, `oc down` |
| Public read | `oc connect` only | `oc codelab list`, `oc codelab reference`, `oc codelab get` |
| Admin | `oc auth login` | `admin`, `backup`, `audit`, `workspace`, `codelab create/update/delete/copy/export/import/push-steps` |
| Attendee | `oc attendee join` | `help request`, `feedback submit`, `quiz submit`, `submission file/link/delete`, `chat history` |

Final permission checks are still enforced by the connected runtime and backend.

## Recommended starting point

```bash
oc init
```

`oc init` is the fastest way to get going. It helps you choose between:

1. Starting a local stack
2. Connecting to an existing server

From there it can continue into profile creation, `oc connect status`, and `oc auth login`.

## Connection and authentication

### `oc init`

```bash
oc init
```

What it does:

- Lets you choose between local stack startup and existing-server connection.
- Uses the `oc run` wizard for local startup.
- Uses the `oc connect add` wizard for an existing server.
- Offers to save a profile and launch browser-based admin auth at the end.

Options:

- None

### `oc connect add`

```bash
oc connect add --name <name> --url <url> [--runtime <auto|backend|firebase|supabase>] [--activate] [--interactive]
```

What it does:

- Saves a connection profile.
- `--activate` makes it the current profile immediately.
- If no current profile exists yet, the first saved profile also becomes current automatically.

| Option | Required | Meaning |
| --- | --- | --- |
| `--name <name>` | usually | The profile name written into the config file. In interactive mode the CLI suggests one. |
| `--url <url>` | usually | The backend base URL, for example `http://localhost:8080`. |
| `--runtime <auto|backend|firebase|supabase>` | no | Hints which runtime to expect when probing the server. |
| `--activate` | no | Makes the saved profile the current profile. |
| `--interactive` | no | Prompts for the profile values instead of requiring all flags up front. |

Runtime value meanings:

| Value | Meaning |
| --- | --- |
| `auto` | Probe the server first and fall back to a static runtime guess if needed. |
| `backend` | Expect the full Rust backend and admin API surface. |
| `firebase` | Expect frontend-managed auth and limited CLI capabilities. |
| `supabase` | Expect frontend-managed auth and limited CLI capabilities. |

### `oc connect use`

```bash
oc connect use [--name <name>]
```

What it does:

- Switches the current profile to one of the saved profiles.

| Option | Required | Meaning |
| --- | --- | --- |
| `--name <name>` | no | The saved profile name to activate. In an interactive terminal, omitting it opens a profile picker. |

### `oc connect list`

```bash
oc connect list
```

What it does:

- Prints every saved profile and marks the current one.

Options:

- None

### `oc connect status`

```bash
oc connect status
```

What it does:

- Shows the resolved base URL, runtime, capabilities, auth methods, and probe status for the current server target.
- Uses `/api/cli/runtime` when the runtime supports the backend probe endpoint.

Options:

- None

### `oc auth login`

```bash
oc auth login [--no-open] [--interactive]
```

What it does:

- Starts browser-based CLI authentication.
- Requires a runtime that exposes browser auth support.
- Saves the resulting session to the resolved session file.

| Option | Required | Meaning |
| --- | --- | --- |
| `--no-open` | no | Do not automatically open the browser. Print the verification URL instead. Useful for SSH or headless terminals. |
| `--interactive` | no | Forces the browser-launch picker even when you already know the defaults. In a TTY with no flags, the picker is shown automatically. |

### `oc auth logout`

```bash
oc auth logout
```

What it does:

- Calls the backend logout endpoint and removes the local session file.

Options:

- None

### `oc auth status`

```bash
oc auth status
```

What it does:

- Shows whether a session file exists, whether the backend still accepts it, and what the current subject, role, and expiry are.

Options:

- None

### Legacy aliases

#### `oc login`

```bash
oc login [--admin-id <id>] [--admin-pw <pw>] [--interactive]
```

What it does:

- Uses the legacy password-based `/api/login` flow.
- New scripts should prefer `oc auth login`.

| Option | Required | Meaning |
| --- | --- | --- |
| `--admin-id <id>` | no | Admin ID. Can be omitted if `OPEN_CODELABS_ADMIN_ID` is set. In interactive mode it becomes the suggested default. |
| `--admin-pw <pw>` | no | Admin password. Can be omitted if `OPEN_CODELABS_ADMIN_PW` is set. In interactive mode it is collected through a hidden prompt. |
| `--interactive` | no | Forces a prompt-driven legacy login flow. |

#### `oc logout`

- Alias for `oc auth logout`.

#### `oc session`

- Alias for `oc auth status`.

## Local runtime

### `oc run`

```bash
oc run [--engine <auto|docker|podman>] [--postgres] [--pull] [--open] [--interactive] [--admin-id <id>] [--admin-pw <pw>] [--data-dir <path>] [--frontend-port <port>] [--backend-port <port>] [--image-registry <registry>] [--image-namespace <namespace>] [--image-tag <tag>]
```

What it does:

- Starts the published frontend/backend container stack locally.
- Detects Docker or Podman automatically.
- Writes the generated compose file and runtime state into `~/.open-codelabs/runtime/local-stack/`.
- Uses host bind mounts for database, uploads, workspaces, and PostgreSQL data.

| Option | Required | Meaning |
| --- | --- | --- |
| `--engine <auto|docker|podman>` | no | Selects the container engine. `auto` tries to detect the best available engine. |
| `--postgres` | no | Adds the bundled PostgreSQL service and points the backend to it instead of SQLite. |
| `--pull` | no | Pulls the latest images before `up -d`. |
| `--open` | no | Opens the admin login page in a browser after startup. |
| `--interactive` | no | Forces the setup wizard even when some flags are already present. |
| `--admin-id <id>` | no | Sets the default admin username for the generated local stack. |
| `--admin-pw <pw>` | no | Sets the default admin password for the generated local stack. |
| `--data-dir <path>` | no | Chooses the host directory used for persistent local data. |
| `--frontend-port <port>` | no | Host port for the frontend. Default: `5173`. |
| `--backend-port <port>` | no | Host port for the backend API. Default: `8080`. |
| `--image-registry <registry>` | no | Overrides the image registry host, for example `ghcr.io`. |
| `--image-namespace <namespace>` | no | Overrides the image namespace, for example `jaichangpark`. |
| `--image-tag <tag>` | no | Overrides the image tag. Default: `latest`. |

Important notes:

- Running `oc run` without flags in a TTY starts the interactive wizard automatically.
- The CLI does not print the literal admin password after startup; it only prints a password hint.

### `oc ps`

```bash
oc ps [--service <name>]
```

What it does:

- Shows the current local stack container status.

| Option | Required | Meaning |
| --- | --- | --- |
| `--service <name>` | no | Limits the output to one service such as `frontend`, `backend`, or `postgres`. |

### `oc logs`

```bash
oc logs [--service <name>] [--tail <n>] [--no-follow]
```

What it does:

- Reads local stack logs.
- Follows the log stream by default.

| Option | Required | Meaning |
| --- | --- | --- |
| `--service <name>` | no | Limits the logs to one service. |
| `--tail <n>` | no | Prints only the last `n` lines. |
| `--no-follow` | no | Reads the current logs once and exits instead of following the stream. |

Important notes:

- `--json` is only supported with `--no-follow`.
- In follow mode the process stays attached to the log stream until you stop it.

### `oc restart`

```bash
oc restart [--service <name>]
```

What it does:

- Restarts the full local stack or one service.

| Option | Required | Meaning |
| --- | --- | --- |
| `--service <name>` | no | Restarts one service instead of the full stack. |

### `oc down`

```bash
oc down [--volumes]
```

What it does:

- Stops the local stack.

| Option | Required | Meaning |
| --- | --- | --- |
| `--volumes` | no | Also removes the local data directories created by `oc run`. |

Important notes:

- `--volumes` is a real data cleanup operation because the local stack uses bind mounts.
- `oc ps`, `oc logs`, `oc restart`, and `oc down` depend on the local runtime state created by a previous `oc run`.

## Administrative commands

### `oc admin settings`

```bash
oc admin settings [--gemini-api-key <key>] [--admin-password <pw>]
```

What it does:

- Updates the administrator's stored Gemini API key.
- The CLI encrypts the raw key with the admin password before sending it to the backend.

| Option | Required | Meaning |
| --- | --- | --- |
| `--gemini-api-key <key>` | usually | The raw Gemini API key to store. |
| `--admin-password <pw>` | conditional | The admin password used for encryption. If omitted, the CLI tries `OPEN_CODELABS_ADMIN_PW`. |

Important notes:

- Running `oc admin settings` without `--gemini-api-key` clears the stored key.

### `oc admin updates`

```bash
oc admin updates
```

What it does:

- Checks the deployed frontend/backend versions and reports whether updates are available.

Options:

- None

## Codelab management

### Read-only codelab commands

| Command | Meaning | Options |
| --- | --- | --- |
| `oc codelab list` | Lists codelabs visible to the current session. | None |
| `oc codelab reference` | Prints the built-in reference codelab content. | None |
| `oc codelab get --id <id>` | Fetches one codelab together with its ordered steps. | `--id`: target codelab ID |

### `oc codelab create`

```bash
oc codelab create --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
```

### `oc codelab update`

```bash
oc codelab update --id <id> --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
```

Shared option meanings:

| Option | `create` | `update` | Meaning |
| --- | --- | --- | --- |
| `--id <id>` | no | yes | The codelab ID to update. |
| `--title <title>` | yes | yes | The codelab title. |
| `--description <desc>` | yes | yes | The codelab summary text. |
| `--author <author>` | yes | yes | The display author name. |
| `--private` | no | no | Flips the default visibility from public to private. |
| `--guide-file <path>` | no | no | Reads a Markdown file and stores its contents as the guide body. |
| `--quiz-enabled` | no | no | Enables quizzes for the codelab. |
| `--require-quiz` | no | no | Makes quiz completion part of the completion criteria. |
| `--require-feedback` | no | no | Makes feedback submission part of the completion criteria. |
| `--require-submission` | no | no | Makes a submission part of the completion criteria. |

Important notes:

- `update` is not a patch-style command. It rebuilds the metadata payload from the flags you pass.
- If you omit `--private`, `--quiz-enabled`, or the `--require-*` flags on update, those values may fall back to their defaults.

### Other codelab commands

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc codelab delete --id <id>` | Deletes a codelab and related data. | `--id`: codelab to delete |
| `oc codelab copy --id <id>` | Copies a codelab together with its steps. | `--id`: source codelab ID |
| `oc codelab export --id <id> [--output <path>]` | Creates a codelab ZIP archive. | `--output` defaults to `codelab_<id>.zip` |
| `oc codelab import --file <zip>` | Imports a codelab ZIP archive. | `--file`: ZIP produced by export |
| `oc codelab push-steps --id <id> --file <json>` | Replaces the entire step list from JSON. | `--file`: `UpdateStepsPayload` JSON |

## Backups and audit logs

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc backup export [--output <path>]` | Creates a full platform backup ZIP. | `--output` defaults to `backup_full.zip` |
| `oc backup inspect --file <zip>` | Inspects a backup ZIP without restoring it. | `--file`: backup ZIP to inspect |
| `oc backup restore --file <zip>` | Restores a backup ZIP into the active backend. | `--file`: backup ZIP to restore |
| `oc audit logs [--limit <n>] [--offset <n>] [--action <name>] [--codelab-id <id>]` | Lists audit logs with optional filters. | `--limit`: page size, `--offset`: pagination offset, `--action`: action name filter, `--codelab-id`: codelab-specific filter |

## Workspaces

Workspace commands manage code-server-backed file structures for codelabs.

### Creation and metadata

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc workspace create --codelab-id <id> [--structure-type <branch|folder>] [--files-json <path>]` | Creates a workspace for a codelab. | `--structure-type`: snapshot strategy, `--files-json`: initial file list JSON |
| `oc workspace info --codelab-id <id>` | Fetches workspace metadata. | `--codelab-id`: target codelab |
| `oc workspace download --codelab-id <id> [--output <path>]` | Downloads the workspace as `tar.gz`. | `--output` defaults to `workspace_<id>.tar.gz` |
| `oc workspace delete --codelab-id <id>` | Deletes the workspace. | `--codelab-id`: target codelab |

### Branch snapshots

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc workspace branches --codelab-id <id>` | Lists saved branch snapshot names. | `--codelab-id`: target codelab |
| `oc workspace branch-create --codelab-id <id> --step-number <n> [--branch-type <start|end>]` | Creates a branch snapshot for the start or end of a step. | `--step-number`: step number, `--branch-type`: `start` or `end`, default `start` |
| `oc workspace branch-files --codelab-id <id> --branch <name>` | Lists file paths inside a branch snapshot. | `--branch`: branch name |
| `oc workspace branch-read --codelab-id <id> --branch <name> --file <path>` | Reads file contents from a branch snapshot. | `--file`: file path inside the snapshot |
| `oc workspace branch-update --codelab-id <id> --branch <name> --files-json <path> [--delete-json <path>] [--commit-message <message>]` | Writes and optionally deletes files inside a branch snapshot. | `--files-json`: write payload or file list, `--delete-json`: delete list, `--commit-message`: change description |

### Folder snapshots

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc workspace folders --codelab-id <id>` | Lists saved folder snapshot names. | `--codelab-id`: target codelab |
| `oc workspace folder-create --codelab-id <id> --step-number <n> --files-json <path> [--folder-type <start|end>]` | Creates a folder snapshot for one step boundary. | `--files-json`: files to include in the snapshot |
| `oc workspace folder-files --codelab-id <id> --folder <name>` | Lists file paths inside a folder snapshot. | `--folder`: folder snapshot name |
| `oc workspace folder-read --codelab-id <id> --folder <name> --file <path>` | Reads file contents from a folder snapshot. | `--file`: file path inside the snapshot |
| `oc workspace folder-update --codelab-id <id> --folder <name> --files-json <path> [--delete-json <path>]` | Writes and optionally deletes files inside a folder snapshot. | `--files-json`: write payload or file list, `--delete-json`: delete list |

Operational tip:

- Use `branch` when you want Git-like step checkpoints.
- Use `folder` when you want each step result stored as its own directory-like snapshot.

## Attendees and live operations

### Attendee session commands

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc attendee join --codelab-id <id> --name <name> --code <code> [--email <email>]` | Registers or rejoins as an attendee and saves the attendee session. | `--code`: attendee join code, `--email`: optional attendee metadata |
| `oc attendee list --codelab-id <id>` | Lists attendees for a codelab. | `--codelab-id`: target codelab |
| `oc attendee complete --codelab-id <id>` | Marks the current attendee session as completed. | `--codelab-id`: codelab to complete |
| `oc attendee certificate [--attendee-id <id>]` | Fetches certificate information. | If `--attendee-id` is omitted, the CLI uses the current attendee session subject. |

### Help queue

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc help request --codelab-id <id> --step-number <n>` | Creates a help request for the current attendee. | `--step-number`: step where the attendee is blocked |
| `oc help list --codelab-id <id>` | Lists help requests for a codelab. | `--codelab-id`: target codelab |
| `oc help resolve --codelab-id <id> --help-id <id>` | Marks one help request as resolved. | `--help-id`: request ID to resolve |

### Feedback

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc feedback submit --codelab-id <id> --difficulty <1-5> --satisfaction <1-5> [--comment <text>]` | Submits attendee feedback. | `--difficulty` and `--satisfaction` are typically score strings such as `1` to `5` |
| `oc feedback list --codelab-id <id>` | Lists feedback rows for a codelab. | `--codelab-id`: target codelab |

## Materials, quizzes, and submissions

### Materials

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc materials list --codelab-id <id>` | Lists codelab materials. | `--codelab-id`: target codelab |
| `oc materials upload --file <path>` | Uploads a material file and returns its asset URL/path. | `--file`: local file to upload |
| `oc materials add --codelab-id <id> --title <title> --type <link|file> [--url <url>] [--file-path <path>]` | Adds a material record to a codelab. | Use `--url` for `link` and `--file-path` for `file` records. |
| `oc materials delete --codelab-id <id> --material-id <id>` | Deletes a material record. | `--material-id`: material to remove |

### Quizzes

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc quiz list --codelab-id <id>` | Lists the current quiz definitions. | `--codelab-id`: target codelab |
| `oc quiz update --codelab-id <id> --file <json>` | Replaces the full quiz set from JSON. | `--file`: `CreateQuiz[]` JSON |
| `oc quiz submit --codelab-id <id> --file <json>` | Submits answers for the current attendee. | `--file`: `QuizSubmissionPayload` JSON |
| `oc quiz submissions --codelab-id <id>` | Lists quiz submissions for administrators. | `--codelab-id`: target codelab |

### Submissions

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc submission list --codelab-id <id>` | Lists submissions visible to the current actor. | `--codelab-id`: target codelab |
| `oc submission file --codelab-id <id> [--attendee-id <id>] --file <path>` | Uploads a file submission. | If `--attendee-id` is omitted, the CLI uses the current attendee session. |
| `oc submission link --codelab-id <id> [--attendee-id <id>] --url <url> [--title <title>]` | Creates a link submission. | `--title` is the display title for the submitted link. |
| `oc submission delete --codelab-id <id> [--attendee-id <id>] --submission-id <id>` | Deletes one submission. | If `--attendee-id` is omitted, the CLI resolves it from the current attendee session. |

## Chat, uploads, and inline comments

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc chat history --codelab-id <id>` | Fetches chat history for a codelab. | `--codelab-id`: target codelab |
| `oc upload image --file <path>` | Uploads an image asset and returns its URL. | `--file`: local image path |
| `oc inline list --codelab-id <id> [--target-type <guide|step>] [--target-step-id <id>]` | Lists inline comment threads. | `--target-step-id` is typically only meaningful when `--target-type step` is used. |
| `oc inline create --codelab-id <id> --file <json>` | Creates a new inline comment thread. | `--file`: `CreateInlineCommentPayload` JSON |
| `oc inline reply --codelab-id <id> --thread-id <id> --file <json>` | Adds a reply to an inline comment thread. | `--file`: `ReplyInlineCommentPayload` JSON |
| `oc inline delete --codelab-id <id> --thread-id <id> --comment-id <id>` | Deletes one inline comment message. | `--thread-id`: thread ID, `--comment-id`: comment ID |

## AI commands

| Command | Meaning | Option details |
| --- | --- | --- |
| `oc ai conversations --codelab-id <id>` | Lists saved AI conversations for one codelab. | `--codelab-id`: target codelab |
| `oc ai stream --file <json>` | Sends an AI request payload and prints the final SSE response body. | `--file`: `AiRequest` JSON |
| `oc ai save --file <json>` | Persists one AI conversation exchange. | `--file`: `SaveAiConversationPayload` JSON |
| `oc ai threads` | Lists AI threads owned by the current admin. | None |
| `oc ai thread-create --title <title> [--codelab-id <id>]` | Creates a new AI thread. | `--codelab-id`: optional codelab association |
| `oc ai thread-delete --thread-id <id>` | Deletes one AI thread. | `--thread-id`: target thread ID |
| `oc ai messages --thread-id <id>` | Lists messages in one AI thread. | `--thread-id`: target thread ID |
| `oc ai message-add --thread-id <id> --file <json>` | Appends one message to an AI thread. | `--file`: `AddAiMessagePayload` JSON |

## JSON input file cheat sheet

Flags such as `--file` and `--files-json` do not accept arbitrary JSON. Each one expects a specific payload shape.

| Command option | Expected content | Meaning |
| --- | --- | --- |
| `codelab create/update --guide-file` | Markdown text file | The file contents are stored as guide Markdown. |
| `codelab push-steps --file` | `UpdateStepsPayload` JSON | Usually contains the full `steps` array and replaces the entire step list. |
| `workspace create --files-json` | `WorkspaceFile[]` JSON | Initial workspace file list. |
| `workspace branch-update --files-json` | `UpdateWorkspaceFilesRequest` or `WorkspaceFile[]` JSON | A plain array is treated as the write list. |
| `workspace branch-update --delete-json` | `string[]` JSON | List of file paths to delete. |
| `workspace folder-create --files-json` | `WorkspaceFile[]` JSON | Files to include in the folder snapshot. |
| `workspace folder-update --files-json` | `UpdateWorkspaceFilesRequest` or `WorkspaceFile[]` JSON | Write payload for the folder snapshot. |
| `workspace folder-update --delete-json` | `string[]` JSON | File paths to delete from the folder snapshot. |
| `quiz update --file` | `CreateQuiz[]` JSON | Replaces the full quiz definition set. |
| `quiz submit --file` | `QuizSubmissionPayload` JSON | The attendee answer payload. |
| `inline create --file` | `CreateInlineCommentPayload` JSON | Defines the initial thread target and body. |
| `inline reply --file` | `ReplyInlineCommentPayload` JSON | Defines one reply to an existing thread. |
| `ai stream --file` | `AiRequest` JSON | Full AI request payload including model, messages, and context. |
| `ai save --file` | `SaveAiConversationPayload` JSON | One persisted AI conversation exchange. |
| `ai message-add --file` | `AddAiMessagePayload` JSON | One message appended to a thread. |

## Operational tips

- Start with `oc init` unless you already know the exact server and auth flow you want.
- Prefer `--json` in automation and plain text in human-driven terminal sessions.
- If you work with both admin and attendee roles, keep separate session files or rely on profile-specific session locations.
- `firebase` and `supabase` profiles may report limited capabilities in `connect status`. In those runtimes, public-read and attendee flows are usually safer than admin automation.
- The current syntax is always available through `oc <command-group> --help`.
