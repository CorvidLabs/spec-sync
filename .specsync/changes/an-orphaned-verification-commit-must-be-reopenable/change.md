---
id: an-orphaned-verification-commit-must-be-reopenable
state: implementing
type: bug_fix
base_commit: 3997fc5bedfb634ee0ba7262b9fe6d79a681accc
---

# An orphaned verification commit must be reopenable

## Intent

an orphaned verification commit must be reopenable

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- An accepted change whose verification commit is unreachable can be reopened even when its delivery inputs are byte-identical, and the reopen ledger records VerificationCommitUnanchored as the cause. Evidence that is still anchored continues to refuse reopen, so the widening is bounded. The sibling validator reopened_change_preserves_sequence_history accepts an equal-digest reopen carrying that cause, so no project-wide sequence freeze occurs. Amended invariants 15 and 18 and REQ-change-017/018/034/035 describe the admitted axis.

## No-spec Rationale

Not applicable
