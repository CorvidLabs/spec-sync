---
change: split-remaining-mcp-server-responsibilities-into-focused-private-modules
artifact: design
---

# Design

The server facade retains run_mcp_server and tool dispatch. Existing public mcp_tool_definitions remains at the same path and delegates to the single unchanged private catalog.

## Responsibility Modules

- protocol: bounded transport, request envelopes, initialize negotiation, exact tool arguments, tool catalog and protocol error shaping; approximately 360 production lines.
- resources: resource list/read dispatch and handlers with their existing snapshot and route revalidation; approximately 310 lines.
- tools: domain operations, shared findings/coverage payload, generation preflight and conservative scoring; approximately 535 lines.
- capabilities: no-follow acquisition, startup root binding, native directory/file/snapshot identities, bounded digest and retained directory opener; approximately 475 lines.
- read_root: lexical root selection, native Windows containment and opaque retained-route revalidation; approximately 355 lines.
- publication: entire staged write/publish/quarantine/rollback transaction; approximately 710 lines. Do not split its ownership state machine further.
- manifest_paths: pure Cargo/package/TOML path interpretation shared by snapshot collection and confinement; approximately 250 lines.
- confinement with manifests child: config/path/source/spec mapping validation and bounded manifest preflight; approximately 700 and 312 lines.
- snapshot with manifests, inputs and traversal children: snapshot orchestrator and shared budget, collection of retained manifest bytes, identity-bound input/config reads, descriptor-bounded tree traversal; approximately 220, 250, 460 and 320 lines.
- test_barriers: existing debug-build race-test hooks and environment constants; approximately 190 lines under cfg(debug_assertions).

All implementation modules remain private. Use pub(super) or narrower parent-scoped visibility for necessary cross-module functions/types, not pub(crate). Private helper fields stay private where module nesting permits. Existing issue-verification bodies remain unchanged; only imports or shared fixture paths may need adjustment.

## Invariants and Review Risks

Keep capabilities, not ambient paths, authoritative. Retain startup identity binding before canonicalization; notification suppression before dispatch; retained roots for mutation; exactly-once manifest-byte charging and preloading; selected-route revalidation before success; whole-batch publication ownership and exact rollback; Unix special-file guards; native Windows comparison and final quarantine-handle consumption. Preserve all CLI/API names, JSON shape, catalog, diagnostics, defaults, limits and provider authentication. No new runtime dependencies or format migrations.

Test modules belong beneath their implementation owner so private functions remain private. Shared fixture helpers can remain in a small cfg(test) mcp::tests facade. Keep every cfg guard with its original test/function and gate platform-only imports equivalently.
