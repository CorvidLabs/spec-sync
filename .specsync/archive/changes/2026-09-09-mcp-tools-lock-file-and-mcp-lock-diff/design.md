---
change: mcp-tools-lock-file-and-mcp-lock-diff
artifact: design
---

# Design

- Extract `mcp_tool_definitions(allow_write)` from `handle_tools_list` so server and
  lock share one catalog.
- New module `mcp_tools_lock` builds lock entries, hashes controlled fields, writes
  the lock, and diffs live vs committed.
- CLI: optional `McpAction` under `Command::Mcp` — `Lock { write, allow_write }` and
  `Diff { allow_write }`; bare `specsync mcp` still starts the server.
- Diff never writes. Missing lock is fail-closed with remedy `mcp lock --write`.
