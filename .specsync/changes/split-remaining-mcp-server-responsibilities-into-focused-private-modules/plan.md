---
change: split-remaining-mcp-server-responsibilities-into-focused-private-modules
artifact: plan
---

# Plan

1. Keep first-PR snapshot frozen; work only in the separate continuation. Reconcile onto its exact published commit before claiming a stacked delivery.
2. Extract pure protocol/catalog and resource handlers, preserving entry points through explicit private imports and the existing public catalog facade.
3. Extract tool-domain operations and shared findings, without duplicating coverage or scoring logic.
4. Separate retained capability primitives, read-root selection and the complete atomic publication engine. Keep their authority, ownership, ordering and cleanup conditions unchanged.
5. Separate pure manifest-path interpretation from confinement preflight and snapshot construction. Organize snapshot collection, retained input reading and traversal as private children sharing the same budget owner.
6. Keep debug integration barriers under cfg(debug_assertions), preserve platform guards, and move owner-specific unit tests alongside their private modules. Reuse a small shared fixture facade; do not expose internal production helpers merely for sibling tests.
7. Keep orchestration and each new source/test file below 1000 lines where coherent boundaries allow. Record any exception with its ownership rationale rather than introducing include-based shards or duplicate code.
8. Synchronize canonical MCP file coverage, internal organization and companions through the lifecycle. Compare moved bodies, test inventory, catalog and diagnostics to the first split.
9. Run focused unit/integration tests after each coherent extraction; finish with formatter, compiler, clippy, release, pre-push, scoped verification and independent review. Compare baseline full-suite failures without hiding them.
10. Return a separate local reviewable continuation. Publication requires coordination after the first PR; no merge or human implementation review is implied.
