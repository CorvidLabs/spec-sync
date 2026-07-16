---
change: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
artifact: plan
---

# Plan

1. Add the backward-compatible ownership-correction record and validation helpers.
2. Add the `change correct-owner` CLI transition and deterministic JSON/text projections.
3. Bind corrections into definition validation and reopened-definition compatibility checks.
4. Add exact corrected owners during acceptance-manifest construction without delta replay.
5. Cover success, portability, serialization compatibility, and every transactional rejection.
6. Update canonical workflow documentation and run the full release verification matrix.
7. Preserve exact legacy baseline ledger evidence while retaining ordinary archive volatility.
