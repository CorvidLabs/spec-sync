---
change: CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid
artifact: design
---

# Design

Text commands need only a boolean integrity gate, not the effective definition or correction
records. Add a domain-level validation helper that reads and validates the correction history
and returns success or a static safe error classification. Command renderers call it before
printing identity, answers, next actions, or correction counts.

The helper deliberately discards parsing and digest detail. JSON continues to call the existing
typed loaders and may return their structured error to machine consumers. This keeps human
text output fail-closed without weakening the cleartext-logging boundary.
