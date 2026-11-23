---
change: synchronize-change-summary-state-after-verification
artifact: testing
---

# Testing

Use approved lifecycle fixtures to exercise first check, completed sync failure, legacy summary refresh, transaction rollback, and check --commit. Reuse the scoped review test after the commit gate.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-check-summary-state-001 | change unit tests exercise passing, failed, retried, and interrupted verification. The CLI integration test review_and_ship_refuse_a_verification_commit_that_lacks_the_work confirms the committed change.md state and records a scoped review after check --commit. |
