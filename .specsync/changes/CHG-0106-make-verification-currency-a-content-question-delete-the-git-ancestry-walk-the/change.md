---
id: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
state: implementing
type: refactor
base_commit: 4f8f6be812b87d95acdabc8ef0c238106f9061af
---

# Make verification currency a content question: delete the git-ancestry walk, the REQ-change-016 persistence allowlist, and the verification-commit ancestry binding

## Intent

Make verification currency a content question: delete the git-ancestry walk, the REQ-change-016 persistence allowlist, and the verification-commit ancestry binding

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Verification currency SHALL be decided by content equality alone — passed evidence, matching plan digest, matching project-input digest — with no git-ancestry walk, no persistence path allowlist, and no verification-commit ancestry binding; verification.commit SHALL remain recorded as an informational correlation key; squash-merged evidence SHALL no longer be orphaned; and REQ-change-013 and REQ-change-016 SHALL be updated so no living requirement describes the deleted history-trust behaviour.

## No-spec Rationale

Not applicable
