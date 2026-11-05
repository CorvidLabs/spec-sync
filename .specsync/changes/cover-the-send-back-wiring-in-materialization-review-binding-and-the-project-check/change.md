---
id: cover-the-send-back-wiring-in-materialization-review-binding-and-the-project-check
state: implementing
type: bug_fix
base_commit: b7a80fc2543308003303fcf179c9a5a700eb47a7
---

# Cover the send-back wiring in materialization, review binding and the project check

## Intent

Cover the send-back wiring in materialization, review binding and the project check

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A sent-back change converges on its own earlier canonical text, a send-back-cancelled approval binds no review, accept refuses orphaned applied text, and the project check reports a deleted send-back, as REQ-change-123 and send_back_for_rescope.rs already prove.

## No-spec Rationale

These edits implement REQ-change-123, already approved and archived in this PR (send-an-approved-change-back-to-draft-for-re-scoping); that change's affected_paths omitted the two files. No contract text changes.
