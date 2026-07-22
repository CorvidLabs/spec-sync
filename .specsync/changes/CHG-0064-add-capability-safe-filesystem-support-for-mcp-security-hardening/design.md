---
change: CHG-0064-add-capability-safe-filesystem-support-for-mcp-security-hardening
artifact: design
---

# Design

Dependency ownership is intentionally separated from CHG-0063 so lifecycle path coverage remains
exact. `Cargo.toml` declares `cap-std = "4"` and moves `tempfile = "3"` from
`dev-dependencies` to `dependencies`. `Cargo.lock` records the capability stack and its
platform-specific support crates.

No public types, commands, flags, output fields, or spec requirements are introduced here.
