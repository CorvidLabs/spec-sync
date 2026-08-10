---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: design
---

# Design

Introduce one change-domain loader that reads the existing change and validates its correction
ledger only after the mutation has acquired the project lock. Apply it to `answer_question`,
`add_dependency`, and `add_supersedes_obligation`; keep the command adapter as thin dispatch and
retain renderer validation for read-only projections. Both private guards use the same fixed safe
diagnostic so neither domain nor renderer failures expose correction values, ledger bytes, or
digests without adding a new exported surface.

The regression fixture snapshots lifecycle files before each invalid-ledger mutation attempt and
compares them byte-for-byte afterward. A domain race fixture holds the project lock, starts a
mutation, corrupts the ledger while that mutation is blocked, releases the lock, and proves the
mutation revalidates before persistence. This makes the atomic no-partial-mutation contract explicit
without changing valid command output or JSON behavior.
