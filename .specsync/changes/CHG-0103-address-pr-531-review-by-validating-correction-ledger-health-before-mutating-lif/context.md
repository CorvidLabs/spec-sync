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
