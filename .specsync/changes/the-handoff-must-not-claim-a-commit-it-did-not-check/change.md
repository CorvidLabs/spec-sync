---
id: the-handoff-must-not-claim-a-commit-it-did-not-check
state: implementing
type: bug_fix
base_commit: 68fab607785719474db1c3573b6d25cce9309e95
---

# The handoff must not claim a commit it did not check

## Intent

the handoff must not claim a commit it did not check

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- The verifying handoff says verification is current for this working tree and a fresh session here resumes at the independent review. It does not say the implementation is committed. The approved and implementing handoff does not say the tree is committed.

## No-spec Rationale

REQ-change-093 already says a current verification is safe and the reason names where a session resumes. The sentence claimed a commit HandoffSignals does not record.
