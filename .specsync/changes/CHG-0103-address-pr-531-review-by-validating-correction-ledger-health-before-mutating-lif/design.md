---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: design
---

# Design

Introduce one command-layer preflight that loads the existing change and validates its correction
ledger before a mutation-capable domain call. Apply it to `answer`, `depend`, and `supersede` while
retaining the existing renderer validation for read-only projections. The domain module remains the
authority for lifecycle transitions; the command adapter only prevents persistence from preceding a
known rendering-integrity failure.

The regression fixture snapshots lifecycle files before each invalid-ledger mutation attempt and
compares them byte-for-byte afterward. This makes the no-partial-mutation contract explicit without
changing valid command output or JSON behavior.
