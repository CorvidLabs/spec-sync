## ADDED

### REQUIREMENT REQ-cli-args-018

The `mcp` command SHALL accept optional `lock` and `diff` subcommands via `McpAction` while preserving bare `specsync mcp` as the stdio server entrypoint.

Acceptance Criteria

- `specsync mcp` / `specsync mcp --allow-write` still start the MCP server (`action: None`).
- `specsync mcp lock` / `specsync mcp lock --write` parse as `McpAction::Lock`.
- `specsync mcp diff` parses as `McpAction::Diff`.
- Help lists the lock/diff subcommands under `mcp`.
