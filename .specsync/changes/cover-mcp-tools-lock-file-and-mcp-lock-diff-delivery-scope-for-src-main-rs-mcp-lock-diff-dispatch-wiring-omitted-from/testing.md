---
change: cover-mcp-tools-lock-file-and-mcp-lock-diff-delivery-scope-for-src-main-rs-mcp-lock-diff-dispatch-wiring-omitted-from
artifact: testing
---

# Testing

- `specsync change audit --strict` passes with no `meaningful changed paths are not covered`
  error naming `src/main.rs`.
- No product code changes; existing parent-change verification and CI gates remain the
  product proof.
