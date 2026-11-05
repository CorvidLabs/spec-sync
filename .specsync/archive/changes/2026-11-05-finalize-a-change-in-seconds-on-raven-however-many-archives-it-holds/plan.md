---
change: finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds
artifact: plan
---

# Plan

## Approach

Let finalize and archive reuse what Git history proves cannot have changed, through a scope that lives only for one command, instead of re-deriving every archive from scratch.

## Out of scope

Approve, check, review and reopen keep reading live. No gate, bound, timeout or approval rule changes. Chough automation (the rest of t813u) is a separate Chough change.

## Steps

Failing Git-process growth test first, then the history scope, the immutable-object memo and the baseline history memo, then the verify lane, `change check --commit`, and before/after timings on Raven.

## Risks

A memo could hide a history violation if its inputs changed. Only queries that name full object ids are memoized, the baseline verdict is keyed by the live-resolved `HEAD` id and the exact baseline bytes, persistent entries keep REQ-change-119's rule, and only passing verdicts are stored.

## Constraints consulted

hi SPEED-2, REQ-change-119, the change.3md read-scope lessons, and Leif's t811u and t813u rules.
