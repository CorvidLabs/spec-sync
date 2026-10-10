---
change: split-remaining-mcp-server-responsibilities-into-focused-private-modules
artifact: plan
---

# Plan

## Approach

Decompose the remaining MCP server by responsibility, retaining a small public facade and private capability, protocol, resource, tool, confinement, snapshot and publication owners. Keep the atomic publication state machine together and preserve all existing function bodies and tests.

## Out of scope

No protocol, CLI, serialization, diagnostics, limits, authentication, filesystem authority or transaction semantics change. Do not modify unrelated change lifecycle code, weaken tests, publish through a browser, or merge. Human implementation review and prerequisite acceptance remain separate gates.

## Steps

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

## Risks

Module wiring can accidentally change visibility, platform cfg boundaries or ownership of security-sensitive state. Keep interfaces parent-private, retain debug integration barriers and compare complete production bodies and test inventories. Existing sandbox failures and unavailable trust tooling must remain explicit.

## Constraints consulted

Honor MCP requirements and recorded context: retained directory capabilities are authority, snapshot budgets stay cumulative, notifications never dispatch work, read roots remain confined, and publication keeps identity checks, rollback and quarantine ordering. Honor the first issue-split dependency; do not bypass its required acceptance. The repository has no configured principles file. Latest hi/plan.md requires this plan to identify its approach, exclusions, steps, risks and consulted constraints.
