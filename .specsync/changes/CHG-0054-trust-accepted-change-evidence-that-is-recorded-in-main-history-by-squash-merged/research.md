---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: research
---

# Research

`accepted_transition_anchors` currently keeps only commits whose parents do not record the change
as accepted, so an evidence refresh committed while the change was already accepted — the normal
shape of a squash-merged re-verification — produces no eligible anchor even when its bytes match
the current workspace exactly. The staged-snapshot fallback then demands
`verification_commit_is_accepted_current`, which squash merges also break because the verification
commit stays on the discarded branch.

`git log` over the active `state.json` path already enumerates every candidate commit cheaply, and
`git_change_record_at` plus the existing byte and projection filters supply the per-anchor
authentication. Dropping the parent-state requirement only for a fallback search reuses all of
this machinery without weakening it: the eligible set is keyed by the framed digest of the exact
evidence bytes, so distinct snapshots can never collapse and ambiguity still fails closed.

The four blocked production changes are the live fixture: each has exactly one commit on `main`
recording it as accepted with current bytes (the squash commits of PRs #390 and #394), so each
yields exactly one eligible recording anchor.
