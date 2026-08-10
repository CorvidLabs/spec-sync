---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: requirements
---

# Requirements

## REQ-cmd-change-010

Lifecycle command adapters SHALL delegate existing-change mutations to domain operations that
validate correction-ledger integrity while holding the same project lock used for persistence.

Acceptance Criteria

- `answer`, `depend`, and `supersede` reject an invalid existing correction ledger before
  changing lifecycle files.
- A ledger that becomes invalid while a mutation waits for the project lock is rejected after lock
  acquisition and before persistence.
- Read-only text show, status, and list views retain their existing fail-closed behavior.
- Successful mutations render effective definition, correction history, and the selected
  normal/strict summary from the snapshot validated by the domain transaction; they cannot report
  a false failure or contradictory JSON because the ledger changes after persistence.
- Valid mutation and rendering behavior is unchanged.
- The `cmd_change` canonical contract version is incremented.

## REQ-change-057

Existing-change definition mutations SHALL validate correction-ledger integrity inside the same
project-lock transaction that persists their state.

Acceptance Criteria

- `answer_question`, `add_dependency`, and `add_supersedes_obligation` acquire the project lock
  before loading and validating the current correction ledger.
- A ledger corrupted while a mutation waits for the lock causes a deterministic safe failure and
  leaves every lifecycle file other than the external corruption byte-for-byte unchanged.
- The safe diagnostic contains no correction value, ledger fragment, or digest.
- Successful mutation machine projections, including normal and strict summaries, are built while
  the project lock is held and remain internally consistent after the lock is released.
- The documented record-returning mutation wrappers remain compiled in production builds.
- Valid mutations retain their established state and output behavior.
