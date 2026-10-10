---
change: split-mcp-issue-verification-into-focused-private-modules
artifact: research
---

# Research

The baseline contains 182 tracked handwritten code files under the inventory's source extensions, excluding generated/vendor/fixtures/snapshots: 2 are at least 10000 lines, 4 more are 5000-9999, and 34 more are 1000-4999. src/change.rs and src/change_tests.rs overlap active PRs #804 and #800. src/mcp.rs does not overlap either PR.

The MCP contract already requires bounded input, exact JSON-RPC behavior, retained filesystem authority, explicit GitHub configuration and authentication, fail-closed issue discovery, and content-free diagnostics. These remain unchanged. Rust source extraction excludes pub(super) from the documented external export set, allowing a genuinely private leaf module.

The pinned Rust 1.89.0 compiler, rustfmt and clippy were installed from the official distribution into a dedicated cloud-workspace toolchain after user approval. The locked baseline build passed. Full Git history is required by the adopted workflow-v2 ancestry gate and was fetched before answering the lifecycle interview.
