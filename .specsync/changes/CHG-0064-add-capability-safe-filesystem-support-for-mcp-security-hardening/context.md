---
change: CHG-0064-add-capability-safe-filesystem-support-for-mcp-security-hardening
artifact: context
---

# Context

CHG-0063 closes the independent security review gaps for MCP issue #414. Its implementation needs
race-resistant filesystem traversal and writes on Unix and Windows. The standard-library
canonicalize-then-open pattern has a check/use race, so the implementation uses `cap-std` directory
capabilities and a bounded temporary project snapshot.

This change owns only dependency declaration and lockfile resolution. CHG-0063 owns all behavior,
tests, documentation, and canonical MCP contract changes.
