---
change: split-validator-validation-and-retained-coverage-into-focused-private-modules
artifact: plan
---

# Plan

## Approach

Move complete implementations into private responsibility modules behind the existing validator API paths. Keep the 670-line ordered validation function intact. Decompose retained coverage without changing its single-root authority, shared budget ownership or identity-check ordering. Move existing tests beside their owners with shared fixtures kept small.

## Out of scope

No behavior, public or crate-visible API, diagnostic ordering/text/severity, schema identity, coverage representation, filesystem authority or security-limit change. No edits to manifest internals or the active change.rs work. No new dependencies on unrelated MCP delivery. No publication, human implementation review, archive or merge is implied.

## Steps

1. Record baseline focused validator and integration results and retain the pristine-main full-suite comparison.
2. Extract validation, its policy helpers, schema checks and source mapping helpers. Retain public entry points and SourceSnapshot variants at their existing paths.
3. Extract coverage orchestration and budget owner, configuration, source traversal, spec inventory, capability access, ownership and debug-only barriers into cohesive private modules.
4. Relocate existing tests into topical child files without dropping assertions, names, helper types or platform cfgs. Minimize parent visibility rather than broadening crate interfaces.
5. Keep the facade and each new source/test file below 1000 lines wherever coherent boundaries permit; do not split function bodies merely to hit a number.
6. Synchronize canonical spec mapping, internal organization and companions through the real lifecycle. Mechanically compare moved bodies and test inventory.
7. Run formatter, focused tests, affected integration coverage, clippy, release build, strict coverage, pre-push and scoped lifecycle verification. Independently review security seams and report full-suite baseline failures and unavailable trust checks honestly.
8. Commit the local reviewable result. Human implementation review and subsequent shipping remain separate gates.

## Risks

Import/visibility rewiring may sever private test access or lose platform guards. SourceSnapshot inputs must never gain ambient fallback reads. Splitting coverage must not duplicate budgets or retained roots, reorder race checkpoints or release identity handles prematurely. Keep complete bodies, explicit seams and owner-child tests; compare every original body and run retained-race regressions.

## Constraints consulted

Honor specs/validator/requirements.md and context.md: retained project capabilities bind spec/source/manifest observations, traversal is sorted and bounded, budgets are shared, descriptor usage is depth-bounded, supplied snapshots are authoritative, and errors accumulate in their established order. Honor the recorded lesson that failed manifest inference cannot veto explicit source_dirs; retain its notice. hi/agent.md forbids self-approved implementation review and hi/plan.md requires this structured plan. There is no configured principles file.
