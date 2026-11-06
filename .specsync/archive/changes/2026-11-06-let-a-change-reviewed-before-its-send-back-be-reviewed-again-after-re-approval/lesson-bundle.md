# Lesson bundle: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Let a change reviewed before its send-back be reviewed again after re-approval
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, src/change_tests/, tests/integration/change.rs, CHANGELOG.md, hi/change.md
- **Acceptance**: A change that was reviewed, sent back, re-answered, re-approved and re-checked can be reviewed again: `change review` records the new review instead of refusing with `scoped review does not bind a recorded definition approval`, and `change finalize` archives it.
- **Acceptance**: The review recorded before the send-back stays in the append-only review history and in committed history, and still validates against the approval it bound, including an approval the send-back cancelled; it is never removed or rewritten.
- **Acceptance**: Until the fresh review, `change status` and `ship-status` report the earlier review stale (not unreadable) and name `change review`, and `change ship` refuses.
- **Acceptance**: Whether a review counts is decided as before: it must match the current contract and evidence, and a review bound only to an approval a send-back cancelled is never current (CHANGE-12.b, REQ-change-123); no gate, bound or timeout is relaxed.
- **Acceptance**: The integration test built from the #914 repro fails on main cd8cab5e and passes with the fix.

## Evidence

- Verification commit: `419db6dfb2ac5e98e8c7d882f0655acd46fddc30`
- Base commit: `cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #914, found while shipping #901 (#880). A change that is reviewed, sent back (`change send-back`), re-answered, re-approved and re-checked can never be reviewed again. `change status --json` says `approval_valid: true` and next `run specsync change review <id> --reviewer <human>`, but `change review` refuses with `scoped review does not bind a recorded definition approval`, and `change ship-status` reports `Review: recorded (unreadable)` and names the same refused command. Nothing recovers it: deleting the stale `review.json` / `review-attempts.json` lets `review` record, but `ship`/`finalize` still read the review as unreadable, because the next `change check --commit` already committed the pre-send-back attempt into the append-only committed history.

Root cause: `validate_scoped_review_ledger_contents` checks every attempt in `review-attempts.json` with `definition_approver_for_review_contract`, which skips every definition approval before `send_back_cancelled_prefix`. The attempt recorded before the send-back is bound to exactly the approval the send-back cancelled, so it binds nothing, the whole history is rejected, and so is every new review.

Reproduced on main cd8cab5e with only documented commands (`approve → check --commit → review → send-back → answer → approve → check --commit → review`), script attached to #914. #901 is blocked on this and waits untouched until it merges.

Out of scope: the send-back command itself, the committed-history walk, and any change to what a current review must bind.

## From the change's design.md

# Design

Two questions were answered by one function; separate them.

- **History validity** (`validate_scoped_review_ledger_contents`): every attempt must bind *some* definition approval the ledger records, counting approvals a later send-back cancelled. A pre-send-back attempt was valid evidence when recorded; CHANGE-12.c keeps it, and the committed-history walk already forbids removing or rewriting it.
- **Currency** (`scoped_review_currency`): unchanged. A review bound only to a cancelled approval has a contract digest no live approval carries, so the existing contract check already reports it stale; once history loads, status and ship-status name the re-review instead of calling the evidence unreadable, and finalize refuses it as any stale review. (An explicit send-back check in currency was prototyped and removed: it is unreachable, because a byte-identical re-approval records a live approval with the same digest.)

Implementation: `definition_approver_for_review_contract(approvals, review)` keeps its behaviour and delegates to `definition_approver_bound_from(approvals, review, first)` with `first = send_back_cancelled_prefix(approvals)`; history validation calls `historical_definition_approver_for_review`, which passes `first = 0`. No new public API.

Out of scope, noted for Leif: after a byte-identical re-approval, a review recorded before the send-back binds the new approval by digest and can count again if nothing else about the change moved. That is main's existing behaviour, not part of the wedge in #914.

Rejected: dropping pre-send-back attempts at send-back time (CHANGE-12.c forbids losing evidence, and committed history forbids it anyway); relaxing currency (a review bound to a cancelled approval must never let finalize through).

## From the change's testing.md

# Testing

Failing test first, through the built binary, under workflow v2.

1. Integration (`tests/integration/change.rs`), the #914 repro: approve, `check --commit`, review, send-back, re-answer the criteria, approve, `check --commit`.
   - Before the second review: `change status --json` reports the review stale with a next action naming `change review` (not unreadable), and `change ship` refuses.
   - The second `change review` records a pass, and `change ship`/`finalize` archive the change; `review-attempts.json` in the archive holds both attempts, the first unchanged.
   Must fail on main cd8cab5e at the second review.
2. Unit (`send_back_for_rescope.rs`): review, send-back, re-answer, re-approve, check-and-commit; on the real ledger the earlier review binds historically and not live; it loads unchanged, is `Stale`, and the fresh review is current and binds the post-send-back approval.
3. Existing send-back, scoped-review and committed-history tests unchanged and green.
4. `specsync change check <id> --commit`, `fledge lanes run verify` (pinned toolchain), pre-push.

## Results

Pending.

## Where these lessons go

- `specs/change/change.3md`
