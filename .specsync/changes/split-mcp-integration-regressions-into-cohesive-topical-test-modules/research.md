---
change: split-mcp-integration-regressions-into-cohesive-topical-test-modules
artifact: research
---

# Research

Pristine main contains 86 complete function declarations in tests/integration/mcp.rs: 77 tests and nine helpers. Of the tests, 51 are unrestricted, 19 Unix-only, five Windows-only, and two Unix-or-Windows; 72 compile and pass on Linux. Four debug rendezvous tests retain cfg_attr(not(debug_assertions), ignore) rather than disappearing from release builds. Child processes execute the MCP binary, not a test-harness self-reexec, so only displayed module-qualified names change.

The root tests/integration.rs registration guard scans its immediate directory. Keep its existing MCP registration unchanged and explicitly register every private child in the facade. The Windows create_windows_junction helper belongs to crate::commands and its import/path must remain intact. FIFO coverage is mandatory; only host PermissionDenied creating a Unix socket fixture permits that fixture to be skipped.

The supplied pristine-main complete integration baseline has 425 passes and 13 lifecycle/comment failures. Existing all-target clippy diagnostics, missing fledge/Attest/Ruby, and unavailable native Windows runtime are limitations to report, not reasons to weaken tests.
