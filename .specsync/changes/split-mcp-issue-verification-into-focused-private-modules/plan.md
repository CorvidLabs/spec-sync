---
change: split-mcp-issue-verification-into-focused-private-modules
artifact: plan
---

# Plan

## Approach

Move the issue-verification implementation and its existing regressions into one private module and one child test file. Preserve every observable behavior and use only parent-level visibility for the handler.

## Out of scope

Protocol, snapshot, publication, authentication, confinement policy, CLI and public API changes are outside this issue-only split. Subsequent private module decomposition belongs to its separately scoped continuation.

## Steps

1. Preserve the baseline MCP test inventory and results before editing.
2. Move issue verification, checked issue discovery, relative diagnostics and their private limit into src/mcp/issues.rs. Only tool_issues is visible to the parent through pub(super). Import required parent confinement helpers explicitly; do not move or rewrite them.
3. Move the 17 existing issue regressions unchanged into src/mcp/issues/tests.rs. Reuse the existing project fixtures with test-only parent visibility rather than duplicating them. Preserve test names and cfg guards; only their module-qualified paths change.
4. Keep source and tests below 1000 lines apiece. Update the living MCP spec file mappings and companions, and materialize the internal-organization delta through the lifecycle.
5. Check exact moved function bodies and test inventory, run formatting, compilation, all MCP unit/integration tests, strict spec coverage, and the available full verification commands. Report unavailable trust tooling and pre-existing failures distinctly.
6. Deliver a local reviewable patch with before/after lengths and test evidence. Do not publish or claim human implementation review.

## Scope

src/mcp.rs, new files beneath src/mcp/, and specs/mcp/ only, plus this change's own lifecycle artifacts. Snapshot authority, output staging, protocol envelopes, CLI, JSON shapes, limit values, diagnostic strings, GitHub authentication and ordering are invariant.

## Risks

Import and test-fixture wiring can accidentally widen authority or omit platform coverage. Compare complete moved bodies and test inventories, retain cfg guards, and run focused security regressions.

## Constraints consulted

Honor the existing MCP requirements and context: explicit GitHub repository/token configuration, bounded capability snapshots, relative diagnostics, unchanged limits and JSON/error shapes. Keep the retained-root security boundary intact. The repository has no configured principles file; the existing MCP context supplies the applicable constraints.
