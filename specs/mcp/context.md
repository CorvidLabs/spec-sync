---
spec: mcp.spec.md
---

## Key Decisions

- **Stdio transport only**: The server reads JSON-RPC from stdin and writes to stdout. No HTTP/WebSocket transport — MCP clients (Claude Code, Cursor, Windsurf) all support stdio.
- **7 tools exposed**: check, coverage, generate, list_specs, init, score, issues. These cover the core spec-sync workflow without exposing low-level internals.
- **4 resources + 1 template**: `specsync:///specs`, `specsync:///graph`, `specsync:///config`, `specsync:///coverage`, plus the `specsync:///specs/{module}` template that returns a single spec as `text/markdown`. The `initialize` response advertises both `tools` and `resources` capabilities.
- **AI provider resolution**: `specsync_generate` delegates to `ai::resolve_ai_provider` (the reworked corvid-ai-backed `ai` module) when `ai: true` or a `provider` argument is given; an unresolvable provider surfaces as a tool error.
- **Errors as tool results**: Tool failures return `isError: true` in the content response, not JSON-RPC error objects. Resource read failures, however, use JSON-RPC error -32602.
- **Optional `root` parameter**: Every tool accepts an optional `root` override so agents can work on projects outside the current working directory.
- **Stateless design**: Each tool invocation loads config from scratch. No server-side state is maintained between calls, which simplifies the implementation and avoids stale data.
- **Notifications ignored**: JSON-RPC requests without an `id` field are treated as notifications and silently dropped.

## Files to Read First

- `src/mcp.rs` — Single-file module implementing the full MCP server: JSON-RPC parsing, tool/resource dispatch, and response formatting.
- `src/ai.rs` — `resolve_ai_provider`, used by `specsync_generate` for AI-backed generation.

## Current Status

Fully implemented. The MCP server is production-ready and used by Claude Code and Cursor for spec-sync integration.

## Notes

- Protocol version is pinned to `"2024-11-05"` per the MCP specification.
- The `specsync_generate` tool supports an `ai` boolean parameter and optional `provider` string for AI-powered generation through the MCP interface.
