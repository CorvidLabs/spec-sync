---
change: split-mcp-implementation-into-cohesive-private-modules
artifact: design
---

# Design

The facade owns only module declarations, the unchanged server/catalog entry points and the complete moved-source provider-process absence test. Protocol handles transport; tools handles dispatch; confinement owns retained-root/path checks; preflight owns input checks; snapshot and snapshot_manifest own bounded copies and workspace discovery; publication owns staged writes and rollback. Shared test fixtures are cfg(test).

Module helpers use private or pub(super) visibility. Public and pub(crate) visibility are not introduced beneath src/mcp. Production bodies/constants and cfg boundaries are conserved; only import qualification and mechanical layout change. Protocol/snapshot child tests import std::fs directly because release-profile cfg(test) cannot inherit a debug_assertions-only parent import.

Historical server_name.rs and publish_staged_file_with_verification_hooks.rs are two-line comment-only files, retained for the existing GitHub source scan. They are no longer included as Rust code. A new MCP test explicitly scans all moved implementation files, so retaining the anchors does not hide actual code from that regression check.
