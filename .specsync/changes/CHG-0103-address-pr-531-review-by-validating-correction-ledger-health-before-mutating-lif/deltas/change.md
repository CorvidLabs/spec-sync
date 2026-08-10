## MODIFIED

### REQUIREMENT REQ-change-057

Existing-change definition mutations SHALL validate correction-ledger integrity inside the same
project-lock transaction that persists their state.

Acceptance Criteria

- `answer_question`, `add_dependency`, and `add_supersedes_obligation` acquire the project lock
  before loading and validating the current correction ledger.
- A ledger corrupted while a mutation waits for the lock causes a deterministic safe failure and
  leaves every lifecycle file other than the external corruption byte-for-byte unchanged.
- The safe diagnostic contains no correction value, ledger fragment, or digest.
- A successful mutation returns the effective definition, correction history, and normal/strict
  summaries built while the project lock is held, so command rendering cannot reread the ledger
  and report a false failure or contradictory JSON after persistence.
- The documented `answer_question`, `add_dependency`, and `add_supersedes_obligation` wrappers
  remain available in production builds while command-only snapshot variants carry richer output.
- Valid mutations retain their established state and output behavior.
