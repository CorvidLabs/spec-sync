---
change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
artifact: context
---

# Context

Issue #914, found while shipping #901 (#880). A change that is reviewed, sent back (`change send-back`), re-answered, re-approved and re-checked can never be reviewed again. `change status --json` says `approval_valid: true` and next `run specsync change review <id> --reviewer <human>`, but `change review` refuses with `scoped review does not bind a recorded definition approval`, and `change ship-status` reports `Review: recorded (unreadable)` and names the same refused command. Nothing recovers it: deleting the stale `review.json` / `review-attempts.json` lets `review` record, but `ship`/`finalize` still read the review as unreadable, because the next `change check --commit` already committed the pre-send-back attempt into the append-only committed history.

Root cause: `validate_scoped_review_ledger_contents` checks every attempt in `review-attempts.json` with `definition_approver_for_review_contract`, which skips every definition approval before `send_back_cancelled_prefix`. The attempt recorded before the send-back is bound to exactly the approval the send-back cancelled, so it binds nothing, the whole history is rejected, and so is every new review.

Reproduced on main cd8cab5e with only documented commands (`approve → check --commit → review → send-back → answer → approve → check --commit → review`), script attached to #914. #901 is blocked on this and waits untouched until it merges.

Out of scope: the send-back command itself, the committed-history walk, and any change to what a current review must bind.
