---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: context
---

# Context

PR #531 review found that text rendering validated correction-ledger health only after
`answer`, `depend`, and `supersede` had persisted their mutations. A corrupt ledger could
therefore produce an error even though the requested mutation already took effect. The same review
also found that the changed `cmd_change` behavioral contract had not incremented its spec version.

The fix must validate existing ledger health before any affected mutation and retain the fail-closed
read behavior for show, status, and list. CHG-0100 is already archived, so this is an independent,
scoped follow-up rather than a rewrite of accepted history.

A fresh PR review found that a command-layer preflight still left a time-of-check/time-of-use gap:
the ledger could change while the domain mutation waited for the project lock. The expanded fix
therefore moves the authoritative validation into each mutation's locked domain transaction. It
also records the original correction-ledger health decision in `specs/change/context.md`.

A later exact-head review found one remaining post-transaction gap: the command renderer reread
`corrections.json` after the domain operation released its lock. If the ledger changed in that
interval, the command exited nonzero after already persisting the requested mutation. The same
review caught that `REQ-change-057` had not been materialized into canonical requirements. The
follow-up carries the validated effective definition and correction history out of the transaction
for output and adds the missing canonical requirement block.

Fresh CodeQL analysis then identified aggregate-snapshot taint reaching two text-only count sinks.
Those counts are now loaded independently from `state.json` using the original command ID, while
correction-derived snapshot values remain confined to the structured JSON branch.

The next exact-head review found that JSON still recomputed `ChangeSummary` after releasing the
domain lock and that the three documented record-returning mutation wrappers were compiled only in
tests. The final repair captures normal and strict summaries inside the locked transaction and
keeps the public wrappers in production while the CLI continues using richer crate-private results.
