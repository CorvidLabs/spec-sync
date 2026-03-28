---
spec: mcp.spec.md
---

## Key Decisions

- **Stdio transport only**: The server reads JSON-RPC from stdin and writes to stdout. No HTTP/WebSocket transport — MCP clients (Claude Code, Cursor, Windsurf) all support stdio.
- **6 tools exposed**: check, coverage, generate, list_specs, init, score. These cover the core spec-sync workflow without exposing low-level internals.
- **Errors as tool results**: Tool failures return `isError: true` in the content response, not JSON-RPC error objects. This follows the MCP convention where tool errors are expected outcomes, not protocol errors.
- **Optional `root` parameter**: Every tool accepts an optional `root` override so agents can work on projects outside the current working directory.
- **Stateless design**: Each tool invocation loads config from scratch. No server-side state is maintained between calls, which simplifies the implementation and avoids stale data.
- **Notifications ignored**: JSON-RPC requests without an `id` field are treated as notifications and silently dropped.

## Files to Read First

- `src/mcp.rs` — Single-file module implementing the full MCP server: JSON-RPC parsing, tool dispatch, and response formatting.

## Current Status

Fully implemented. The MCP server is production-ready and used by Claude Code and Cursor for spec-sync integration.

## Notes

- Protocol version is pinned to `"2024-11-05"` per the MCP specification.
- The `specsync_generate` tool supports an `ai` boolean parameter and optional `provider` string for AI-powered generation through the MCP interface.
