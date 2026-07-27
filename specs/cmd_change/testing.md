---
spec: cmd_change.spec.md
---

# Testing

CLI integration coverage validates creation, JSON schema, rationale errors, adoption dry runs, initialization, and the complete stale accepted → reopen → verify → reaccept flow. Reopen assertions cover deterministic audit JSON and preserved approval/reopen ledger history. Domain transitions are covered by change-module unit tests.

`REQ-cmd-change-002` is covered by the accepted → correct → approve → verify → reaccept CLI integration flow. It asserts equivalent text and JSON original/effective projections, correction history, added-artifact next actions, and persisted append-only evidence.

`REQ-cmd-change-003` is covered by the reopened → correct-owner CLI integration flow. It asserts deterministic JSON persistence, equivalent human output, required audit inputs, exact path/spec ownership validation, next-gate guidance, and transactional rejection.

`REQ-cmd-change-004` is covered by the batch correct-owner CLI integration flow. It asserts repeated-path batch success, atomic rejection when any entry is invalid, and deterministic JSON persistence of every appended correction.

`REQ-cmd-change-005` is covered by integration regressions for healthy records surviving one
corrupt workspace in text and degraded JSON, corrupt status failure, missing affected-spec
rejection before state allocation, optional supersede digest resolution, and the explicit warning
when definition approval interrupts ordinary verification.
