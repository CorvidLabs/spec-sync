---
change: CHG-0051-govern-the-deterministic-reconciliation-of-concurrent-accepted-chg-0048-sequence
artifact: research
---

# Research

Rebasing rewrites accepted verification commits and makes otherwise-current terminal evidence
off-history. Reopening either CHG-0048 while its collision acknowledgement is present is correctly
rejected because acknowledged collisions must contain only immutable accepted or archived records.
Renumbering either accepted record would invalidate its signed definition and closing approvals.

The existing lifecycle contract already permits a valid later sequence claim to supersede only the
historical sequence-ledger bytes. A new canonical change is therefore the deterministic path that
preserves both immutable CHG-0048 histories and governs the merged ledger bytes.
