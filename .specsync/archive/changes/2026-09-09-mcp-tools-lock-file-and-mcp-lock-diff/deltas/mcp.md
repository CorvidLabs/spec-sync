## ADDED

### REQUIREMENT REQ-mcp-009

SpecSync SHALL expose a committed MCP tools lock and fail-closed drift check over the same tool catalog the MCP server registers for `tools/list`.

Acceptance Criteria

- SoT path is `.specsync/mcp-tools.lock.json`.
- `specsync mcp lock` prints lock JSON from the live catalog; `specsync mcp lock --write` writes it explicitly; bare `diff` never rewrites the lock.
- `specsync mcp diff` exits 0 when live catalog matches the lock; exits non-zero on tool add/remove/rename, sha256 mismatch, tool_count mismatch, or missing lock, with a remedy pointing at `mcp lock --write`.
- Canonical sha256 is SHA-256 of UTF-8 JSON with sorted object keys and separators `(',', ':')` over `{name, title?, description, inputSchema}`; `title` is omitted when null/absent.
- Lock/diff call `mcp_tool_definitions` — one catalog shared with `tools/list`.
