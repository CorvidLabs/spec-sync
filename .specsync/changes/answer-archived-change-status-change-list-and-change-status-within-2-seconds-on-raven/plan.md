---
change: answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven
artifact: plan
---

# Plan

## Approach

Stop re-deriving archived evidence on read-only paths: compute only the asked-for archived record's terminal evidence, memoize repeated Git reads per read scope, and keep passing archived history verdicts and archive introductions in a digest-keyed cache under the Git directory that Git history invalidates.

## Out of scope

Mutating commands (approve, check, review, finalize, reopen) keep reading live. No gate, bound, timeout or approval rule changes. The other items in Leif's t811u brief (amend paths, the approve ownership message) are separate changes.

## Steps

Failing Git-process growth test first, then the three fixes, then invalidation and parity tests, the Raven-sized benchmark, the pre-push and verify lanes, `change check --commit`, and before/after timings on Raven.

## Risks

A stale cache could hide a history violation. Entries are reused only under the digest and history rule in REQ-change-119, refusals are never cached, the baseline check stays live, and tests prove a later hidden downgrade, an edited archive and an unrelated HEAD all recompute.

## Constraints consulted

hi SPEED-1 and SHIP-7, the change.3md read-scope lessons, and Leif's t811u rules.
