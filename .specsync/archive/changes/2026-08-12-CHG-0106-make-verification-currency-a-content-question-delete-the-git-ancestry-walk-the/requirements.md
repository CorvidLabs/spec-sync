---
change: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
artifact: requirements
---

# Requirements

Modifies two living requirements whose implementation this change deletes.

## REQ-change-013 — content-only freshness

Freshness SHALL be decided by content equality alone. No commit ancestry, intervening-commit
inspection, or path allowlist participates in the decision.

## REQ-change-016 — verifying evidence judged on content

Verification currency SHALL NOT depend on commit ancestry, on inspecting intervening commits,
or on restricting which paths may change after verification. `verification.commit` SHALL be
retained as an informational correlation key and SHALL never be a gate. Accepted closing
evidence obligations are unchanged by this step.
