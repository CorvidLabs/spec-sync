---
change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
artifact: context
---

# Context

#982, a Codex P2 on #979 deferred by Leif's call (P2 policy: issue, reply, resolve).
`lifecycle_state_history` (`src/change/validate_loaded_change.rs`) kept `(root, HEAD, built)` in
a per-thread slot whether `built` was an index or `None`. After one transient failure (for
example an offline partial clone that cannot fetch a tree), every later read scope at the same
HEAD got the stored `None` and never tried again, so a long-lived library caller stayed on the
per-archive fallback and paid Git processes per archive. Speed only: the fallback gives the same
verdicts. A CLI command is one process, so only library callers were affected.

The fix keeps only a built index in the per-thread slot and records a failed build in the
current read scope, so that scope does not retry it on every archive and the next scope builds
again. A test-only hook (`fail_lifecycle_state_history_builds`) fails a build on purpose, beside
the existing `without_lifecycle_state_history` hook.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.
