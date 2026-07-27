---
change: CHG-0064-add-capability-safe-filesystem-support-for-mcp-security-hardening
artifact: tasks
---

# Tasks

- [x] Characterize the canonicalize/open race through independent adversarial review.
- [x] Select a cross-platform capability filesystem implementation compatible with Rust 1.89.
- [x] Add direct runtime dependencies and update the lockfile.
- [x] Pass locked Rust 1.89 dependency resolution and native type-checking.
- [x] Pass native MCP unit and integration tests with the runtime dependency layout.
- [x] Pass locked Windows cross-target compilation.
- [x] Confirm dependency wiring introduces no public CLI or canonical spec contract change.
