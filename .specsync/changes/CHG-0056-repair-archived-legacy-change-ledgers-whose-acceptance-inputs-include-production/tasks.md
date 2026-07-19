---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: tasks
---

# Tasks

- [x] Implement the `UnownedProductionSource` policy and relax it only in legacy reconstruction.
- [x] Add regression coverage for the relaxed legacy path and the strict current path.
- [x] Add and map canonical requirement REQ-change-038 and extend the canonical Invariants
  section.
- [x] Run pre-acceptance formatting, lint, unit/integration tests, and release validators.
- [x] Prepare post-acceptance validation of the CorvidLabs/agent-findings archived ledger with
  the fixed binary, then run forced strict and Trust.
