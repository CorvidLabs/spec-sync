---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: plan
---

# Plan

1. Add an `UnownedProductionSource` policy to `acceptance_manifest`/`acceptance_input_owners` and
   relax it only in `reconstruct_legacy_at_anchor`.
2. Cover the relaxed legacy reconstruction and the strict current-acceptance path with
   regression tests.
3. Add canonical requirement REQ-change-038 and extend the canonical Invariants section.
4. Accept the change, then verify the CorvidLabs/agent-findings archived ledger validates with
   the fixed binary.
5. Run forced strict and the complete Trust lane.
