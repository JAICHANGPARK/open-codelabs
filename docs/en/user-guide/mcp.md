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
- `create_codelab`, `update_codelab`, `replace_codelab_steps`, `list_attendees`, `list_help_requests`, and `resolve_help_request` require an admin session.

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

| Tool | Meaning | Access |
| --- | --- | --- |
| `get_connection` | Returns the current profile, base URL, runtime probe, and session status. | Any session |
| `list_codelabs` | Returns codelabs visible to the current session. | Any session |
| `get_codelab` | Returns metadata, guide markdown, and ordered steps for one codelab. | Any session |
| `create_codelab` | Creates a new codelab. | Admin |
| `update_codelab` | Updates existing codelab metadata. | Admin |
| `replace_codelab_steps` | Replaces the full ordered step list for a codelab. | Admin |
| `list_attendees` | Returns attendee records for one codelab. | Admin |
| `list_help_requests` | Returns the help queue for one codelab. | Admin |
| `resolve_help_request` | Resolves one help request. | Admin |

## Exposed resources

| Resource URI | Meaning |
| --- | --- |
| `oc://connection` | Current connection state and runtime probe result |
| `oc://session` | Current session subject, role, and session file |
| `oc://codelabs` | Codelabs visible to the current session |
| `oc://codelabs/{id}` | Codelab metadata |
| `oc://codelabs/{id}/guide` | Guide markdown |
| `oc://codelabs/{id}/steps` | Ordered steps |
| `oc://codelabs/{id}/attendees` | Attendee list, requires admin session |
| `oc://codelabs/{id}/help` | Help request list, requires admin session |

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
- The guide and steps are split into `oc://codelabs/{id}/guide` and `oc://codelabs/{id}/steps`, which makes it easier to add only the required context.
- Refresh the same profile with `oc auth login` before using admin write tools.
- If you want to isolate the MCP session from your normal terminal usage, point the host at a dedicated `--session-file`.
