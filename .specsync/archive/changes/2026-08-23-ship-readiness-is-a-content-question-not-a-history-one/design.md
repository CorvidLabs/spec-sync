---
change: ship-readiness-is-a-content-question-not-a-history-one
artifact: design
---

# Design

## One export, no new resolver

`recorded_verification_is_current(root, record)` loads the evidence and delegates to the existing
`verification_is_current`. One export rather than two, because callers outside the module need the
answer, not the record. Missing or unreadable evidence is not current — total by design, since a
strict `?` would turn `ship-status` from rc=0 into rc=1 on a workspace whose evidence is already
damaged, and the fix for an inspection command must not brick inspection.

## Two sites in ship-status

    ready_to_finalize:  verification_ancestor  ->  recorded_verification_is_current
    the blocker:        "not an ancestor of HEAD"
                        ->  "verification evidence is stale for the current tree"

Both were required. Fixing only `ready_to_finalize` left the blocker in place, and readiness also
requires `blockers.is_empty()` — measured: the fixture still reported "not ready" until the second
site changed.

`verification_ancestor_of_head` stays in the JSON report. It is true information and consumers may
want it; it simply no longer decides readiness.
