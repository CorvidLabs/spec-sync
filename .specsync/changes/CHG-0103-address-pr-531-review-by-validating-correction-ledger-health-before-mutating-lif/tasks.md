---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: tasks
---

# Tasks

- [x] Guard existing-change mutations before persistence.
- [x] Add regression coverage for invalid-ledger mutation attempts.
- [x] Increment and update the cmd_change contract.
- [x] Run focused validation and prepare scoped verification.
- [x] Move correction-ledger validation into the locked change-domain mutation path.
- [x] Add deterministic regression coverage for corruption while a mutation waits on the lock.
- [x] Record the correction-ledger decision in the change-domain companions and contract.
- [x] Renew verification and prepare the expanded scope for independent review.
