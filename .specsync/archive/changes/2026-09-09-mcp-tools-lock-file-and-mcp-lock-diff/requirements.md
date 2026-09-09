---
change: mcp-tools-lock-file-and-mcp-lock-diff
artifact: requirements
---

# Requirements

1. SoT path: `.specsync/mcp-tools.lock.json` (versioned lock document).
2. `specsync mcp lock` prints lock JSON from the live catalog (read-only).
3. `specsync mcp lock --write` explicitly writes the lock under the project root.
4. `specsync mcp diff` exits 0 on match; non-zero on add/remove/rename, sha256
   mismatch, tool_count mismatch, or missing lock, with a clear remedy.
5. Canonical sha256: SHA-256 of UTF-8 JSON with sorted keys and separators
   `(',', ':')` over `{name, title?, description, inputSchema}`; omit title when
   null/absent.
6. Lock/diff reuse `mcp_tool_definitions` — the same list `tools/list` registers.
7. Dogfood: commit SpecSync's own read-only five-tool lock.
