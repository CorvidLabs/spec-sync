---
change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
artifact: plan
---

# Plan

## Approach

Add one check that compares the canonical requirement ids at the reference commit with those in the working tree, subtracts the ids approved `## REMOVED` items of the branch's changes account for, and refuses with the rest; call it at check, acceptance, verify and audit, and list it in ship-status.

## Out of scope

A CI-side guard on an archived tip (archive-first CI never sees an active change; a follow-up issue if wanted). Whether a requirement moved between modules needs its own delta.

## Steps

1. Commit the failing tests: the starling truncation at check, and the ship-status blocker.
2. Write `src/change/vanished_requirements.rs`: the reference, the id collection (Git and working tree), the accounting and the message.
3. Call it at check (after the read-only gates, before writing), acceptance, verify and audit; add the ship-status blocker.
4. Tests for every case in REQ-change-169's criteria; docs, CHANGELOG and the lesson.
5. `change check --commit`, the verify lane, `change audit --strict`, trust.

## Risks

A project whose requirements files are many and large pays one Git read per file at the merge-base; bounded by reading only files that differ from the reference. A false refusal on a legitimate cross-file move is avoided by counting ids across all files.

## Constraints consulted

REQ-change-142 (fail closed when Git cannot answer), #847 (refused attempts), CHANGE-11.b (the health check reports what check refuses), REQ-change-122 (the base check's placement), and the lesson that every path a test touches is declared, or `change audit --strict` refuses.
