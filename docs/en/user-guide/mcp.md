# MCP Server

`oc mcp serve` exposes Open Codelabs as a stdio MCP server by reusing the current `oc` profile and session. It does not need a separate token flow or a second config file. If `oc connect` and `oc auth login` already work, the MCP host can reuse that state directly.

## When to use it

- When you want Claude Desktop, Codex, Cursor, or another MCP host to inspect Open Codelabs data
- When you want AI tooling to read codelab metadata, guide markdown, and step content directly
- When you want an authenticated admin session to create or update codelabs and resolve help requests

## Prerequisites

The minimum setup is:

```bash
oc connect add --name local --url http://localhost:8080 --runtime backend --activate
oc auth login
```

- If you only need public read-only codelabs, some tools and resources can still work without `oc auth login`.
- `create_codelab`, `update_codelab`, `copy_codelab`, `delete_codelab`, `replace_codelab_steps`, the material/quiz/workspace tools, `list_attendees`, `list_help_requests`, and `resolve_help_request` require an admin session.
- Prompts such as `help-queue-triage` and `learner-ops-review` become more useful when an admin session is available because they can attach richer resources.

## Start the server

```bash
oc mcp serve
```

This command uses the stdio transport. In practice that means it is meant to be launched by an MCP host as a child process, not used as a human-facing interactive command.

The command has no dedicated flags of its own, but it accepts the normal global `oc` options:

- `oc --profile workshop-prod mcp serve`
- `oc --base-url https://labs.example.com mcp serve`
- `oc --session-file /tmp/oc-admin-session.json mcp serve`

## Exposed tools

For MCP `outputSchema` compliance, JSON-returning tools emit their structured result as an object shaped like `{ "data": ... }`.

| Tool | Meaning | Access |
| --- | --- | --- |
| `get_connection` | Returns the current profile, base URL, runtime probe, and session status. | Any session |
| `get_codelab_reference` | Returns the built-in reference payload. | Any session |
| `list_codelabs` | Returns codelabs visible to the current session. | Any session |
| `get_codelab` | Returns metadata, guide markdown, and ordered steps for one codelab. | Any session |
| `get_codelab_bundle` | Returns metadata, guide, steps, materials, and quizzes together. | Admin |
| `create_codelab` | Creates a new codelab. | Admin |
| `update_codelab` | Updates existing codelab metadata. | Admin |
| `copy_codelab` | Copies an existing codelab. | Admin |
| `delete_codelab` | Deletes an existing codelab. | Admin |
| `replace_codelab_steps` | Replaces the full ordered step list for a codelab. | Admin |
| `list_materials` | Returns codelab materials. | Admin |
| `upload_material_asset` | Uploads a local file and returns the material asset URL. | Admin |
| `add_material` | Adds a link or file material record to a codelab. | Admin |
| `delete_material` | Deletes a material record. | Admin |
| `list_quizzes` | Returns quiz definitions for a codelab. | Admin |
| `update_quizzes` | Replaces the full quiz set for a codelab. | Admin |
| `list_feedback` | Returns attendee feedback for a codelab. | Admin |
| `list_submissions` | Returns learner submissions for a codelab. | Admin |
| `list_quiz_submissions` | Returns quiz submissions for a codelab. | Admin |
| `get_chat_history` | Returns stored chat history for a codelab. | Admin |
| `list_attendees` | Returns attendee records for one codelab. | Admin |
| `list_help_requests` | Returns the help queue for one codelab. | Admin |
| `resolve_help_request` | Resolves one help request. | Admin |
| `get_workspace_info` | Returns workspace metadata for a codelab. | Admin |
| `list_workspace_branches` | Returns branch snapshot names for a workspace. | Admin |
| `list_workspace_folders` | Returns folder snapshot names for a workspace. | Admin |
| `list_workspace_branch_files` | Returns file paths inside a branch snapshot. | Admin |
| `read_workspace_branch_file` | Returns file contents from a branch snapshot. | Admin |
| `list_workspace_folder_files` | Returns file paths inside a folder snapshot. | Admin |
| `read_workspace_folder_file` | Returns file contents from a folder snapshot. | Admin |

## Exposed prompts

The current MVP exposes these reusable prompt templates:

| Prompt | Meaning | Access |
| --- | --- | --- |
| `facilitator-brief` | Builds a facilitator briefing from the guide, steps, materials, and quizzes. | Any session, with richer links for admin |
| `authoring-change-plan` | Turns a codelab change request into an authoring plan grounded in the current content. | Any session, more precise with admin bundle access |
| `help-queue-triage` | Reviews the help queue and learner activity to prioritize facilitator follow-up. | Admin recommended |
| `learner-ops-review` | Reviews attendees, submissions, quiz submissions, and feedback together. | Admin recommended |

These prompts are exposed through `prompts/list` and `prompts/get`, so MCP hosts can start from a reusable workflow template instead of manually reconstructing the same instructions on every run.

## Exposed resources

| Resource URI | Meaning |
| --- | --- |
| `oc://connection` | Current connection state and runtime probe result |
| `oc://session` | Current session subject, role, and session file |
| `oc://reference` | Built-in reference payload |
| `oc://codelabs` | Codelabs visible to the current session |
| `oc://codelabs/{id}` | Codelab metadata |
| `oc://codelabs/{id}/bundle` | Combined metadata, guide, steps, materials, and quizzes |
| `oc://codelabs/{id}/guide` | Guide markdown |
| `oc://codelabs/{id}/steps` | Ordered steps |
| `oc://codelabs/{id}/attendees` | Attendee list, requires admin session |
| `oc://codelabs/{id}/help` | Help request list, requires admin session |
| `oc://codelabs/{id}/materials` | Material list, requires admin session |
| `oc://codelabs/{id}/quizzes` | Quiz definition list, requires admin session |
| `oc://codelabs/{id}/quiz-submissions` | Quiz submission list, requires admin session |
| `oc://codelabs/{id}/feedback` | Feedback rows, requires admin session |
| `oc://codelabs/{id}/submissions` | Learner submissions, requires admin session |
| `oc://codelabs/{id}/chat` | Stored chat history, requires admin session |
| `oc://codelabs/{id}/workspace` | Workspace metadata, requires admin session |
| `oc://codelabs/{id}/workspace/branches` | Workspace branch snapshot list, requires admin session |
| `oc://codelabs/{id}/workspace/folders` | Workspace folder snapshot list, requires admin session |

The resource list expands dynamically from `oc://codelabs`, so each visible codelab also exposes its own detail, guide, and steps resources.

## Claude Desktop example

If `oc` is already on your PATH, you can register it like this:

```json
{
  "mcpServers": {
    "open-codelabs": {
      "command": "oc",
      "args": ["mcp", "serve"]
    }
  }
}
```

If you want to pin the server to a specific saved profile, place the global option before `mcp serve`:

```json
{
  "mcpServers": {
    "open-codelabs-prod": {
      "command": "oc",
      "args": ["--profile", "prod", "mcp", "serve"]
    }
  }
}
```

## Operational tips

- Read `oc://connection` first so the model understands the current server target and permission scope.
- Use `oc://codelabs/{id}/bundle` when you need the full authoring context, or read `guide`, `steps`, `materials`, and `quizzes` separately when you want tighter context control.
- For recurring workflows, start from `facilitator-brief`, `authoring-change-plan`, `help-queue-triage`, or `learner-ops-review` before calling lower-level tools directly.
- Refresh the same profile with `oc auth login` before using admin write tools.
- If you want to isolate the MCP session from your normal terminal usage, point the host at a dedicated `--session-file`.
