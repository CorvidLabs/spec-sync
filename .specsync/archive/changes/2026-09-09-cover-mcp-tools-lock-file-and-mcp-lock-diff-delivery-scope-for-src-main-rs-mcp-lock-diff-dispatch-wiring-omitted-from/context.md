---
change: cover-mcp-tools-lock-file-and-mcp-lock-diff-delivery-scope-for-src-main-rs-mcp-lock-diff-dispatch-wiring-omitted-from
artifact: context
---

# Context

`mcp-tools-lock-file-and-mcp-lock-diff` delivered MCP `lock`/`diff` CLI dispatch wiring in
`src/main.rs`, but its declared `affected_paths` omitted that file, so delivery-diff coverage
and `change audit --strict` report it as uncovered. Lifecycle freezes scope at definition
approval, so a small bookkeeping change declares ownership of `src/main.rs`. No code or
canonical spec content changes.
