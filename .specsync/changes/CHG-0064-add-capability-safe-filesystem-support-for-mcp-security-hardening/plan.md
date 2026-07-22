---
change: CHG-0064-add-capability-safe-filesystem-support-for-mcp-security-hardening
artifact: plan
---

# Plan

1. Add `cap-std` as a direct runtime dependency.
2. Promote the existing `tempfile` test dependency to a runtime dependency without duplication.
3. Regenerate `Cargo.lock` with versions compatible with Rust 1.89.
4. Verify native type-checking, linting, MCP tests, and Windows cross-compilation.
5. Keep the public CLI and canonical specs unchanged in this dependency-only change.
