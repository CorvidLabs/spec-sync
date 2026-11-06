---
change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
artifact: design
---

# Design

Two questions were answered by one function; separate them.

- **History validity** (`validate_scoped_review_ledger_contents`): every attempt must bind *some* definition approval the ledger records, counting approvals a later send-back cancelled. A pre-send-back attempt was valid evidence when recorded; CHANGE-12.c keeps it, and the committed-history walk already forbids removing or rewriting it.
- **Currency** (`scoped_review_currency`): a review is current only if it binds a definition approval recorded after the latest send-back (the existing `definition_approver_for_review_contract`, unchanged). Otherwise it is `Stale("the review was recorded before the change was sent back, and the approval it bound is cancelled")`, so status and ship-status name the re-review instead of calling the evidence unreadable, and finalize refuses it as it refuses any stale review.

Implementation: `definition_approver_for_review_contract(approvals, review)` keeps its behaviour and delegates to `definition_approver_bound_from(approvals, review, first)` with `first = send_back_cancelled_prefix(approvals)`; history validation calls it with `first = 0`. No new public API.

Rejected: dropping pre-send-back attempts at send-back time (CHANGE-12.c forbids losing evidence, and committed history forbids it anyway); relaxing currency (a review bound to a cancelled approval must never let finalize through).
