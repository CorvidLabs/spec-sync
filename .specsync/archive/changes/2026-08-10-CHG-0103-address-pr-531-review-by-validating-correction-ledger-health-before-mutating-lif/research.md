---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: research
---

# Research

The PR review traced the defect to command ordering: `answer_change`, `depend_change`, and
`supersede_change` persist before `print_record` performs correction-ledger validation. Moving the
existing integrity check only into the renderer cannot protect mutation atomicity. A shared
pre-mutation check is the smallest repair and reuses the same safe diagnostic already exercised by
read-only text-view tests.

The affected contract is isolated to `cmd_change`; correction-ledger validity and transition policy
remain owned by the change domain. No schema migration or archive rewrite is required.
