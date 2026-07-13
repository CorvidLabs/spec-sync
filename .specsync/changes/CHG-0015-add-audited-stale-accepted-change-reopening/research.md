---
change: CHG-0015-add-audited-stale-accepted-change-reopening
artifact: research
---

# Research

The existing `verification.json` is intentionally the latest evidence, while `approvals.json` is an ordered ledger. Replacing verification without snapshotting would destroy the prior accepted evidence; mutating or deleting the old acceptance approval would weaken auditability. Embedding both immutable records in an appended reopen event preserves compatibility with existing ledgers through `serde(default)`.

Returning to `implementing` would remove the accepted stale check and could make CI green before fresh verification. Returning to `verifying` preserves the stale current evidence, so the existing strict verification gate remains red until `change verify` records fresh inputs.
