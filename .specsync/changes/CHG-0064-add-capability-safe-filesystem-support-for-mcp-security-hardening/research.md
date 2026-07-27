---
change: CHG-0064-add-capability-safe-filesystem-support-for-mcp-security-hardening
artifact: research
---

# Research

`cap-std::fs::Dir` represents an already-open directory and resolves subsequent operations relative
to that capability. Its read, directory traversal, create, and open-with methods avoid ambient path
authority and are designed to prevent symlink/junction escape races across Unix and Windows.

The selected 4.x release supports the repository's Rust 1.89 MSRV. `tempfile` is already locked and
used by tests; MCP now also uses it to own and automatically clean invocation-scoped snapshots.
