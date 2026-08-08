---
change: CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid
artifact: requirements
---

# Requirements

Text lifecycle inspection is a safety surface. A malformed or unauthenticated correction ledger
must never be presented as a successful text status simply because the renderer avoided loading
the full audit record.

The human-facing error must be generic and actionable: identify the correction ledger as invalid,
state that inspection cannot continue, and direct the operator to restore it from trusted history.
It must not contain ledger contents, correction values, or digests.
