---
change: refuse-incomplete-configured-root-coverage-after-cargo-discovery-loss-for-specsync-6-1-1
artifact: plan
---

# Plan

## Approach

Fail inconclusive when an eligible selected regular root file has lost the checked Cargo authority required for exact-file measurement. Retain existing directory and filter behavior, rather than silently omitting the file or broadening admission.

## Out of Scope

No arbitrary exact-file support, parent-directory expansion, workspace glob discovery, owner remapping, dependencies, CLI options, release-policy changes or rewritten human acceptance.

## Steps

1. Preserve exact published subject and actual partial-denominator failure fixtures.
2. Add targeted regressions that fail on the original source; cover malformed/missing/changed valid Cargo authority, valid root targets and explicit directory-only controls.
3. Implement the smallest retained-source admission guard, preserving filters, confinement and budgets.
4. Review actual source and evidence independently; correct canonical specification and later3md lesson without rewriting earlier planes.
5. Freeze final runtime/test/tool inputs, run appropriate complete pinned local gates and current audit, and preserve exact results.
6. Present the concrete tested repair and four-fix implementation for genuine human definition/implementation acceptance as appropriate. Never fabricate a human review.
7. Use supported native verification, exact-head product CI, sequential review/archive and archive CI, normal merge and immutable dual-OS qualification before authorized publication.

## Risks

A guard applied only to explicitly configured lists can miss a retained inferred file selection whose manifest disappears between discovery phases; exercise that sequence. Checking too early can wrongly reject excluded or non-source files, while checking too late can reproduce the false denominator. Preserve retained no-follow metadata and input bounds. Prior successful CI does not qualify changed source, and the known local Mac watcher control limitation remains disclosed.

## Constraints Consulted

Existing REQ-validator-044 restricts regular exact-file admission to checked declared root Cargo targets; REQ-validator-008 retains explicitly configured directory fallback after discovery failure. The validator3md retained capability, identity and resource-limit lessons constrain the guard. AGENTS.md requires real human review before same-PR finalization and a full local verification lane before completion. Independent source analysis and the actual mixed-inventory causal fixture establish the selected narrow repair.
