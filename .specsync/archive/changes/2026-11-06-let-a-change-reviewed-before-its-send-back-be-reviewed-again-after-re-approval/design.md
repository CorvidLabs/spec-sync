---
change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
artifact: design
---

# Design

Two questions were answered by one function; separate them.

- **History validity** (`validate_scoped_review_ledger_contents`): every attempt must bind *some* definition approval the ledger records, counting approvals a later send-back cancelled. A pre-send-back attempt was valid evidence when recorded; CHANGE-12.c keeps it, and the committed-history walk already forbids removing or rewriting it.
- **Currency** (`scoped_review_currency`): unchanged. A review bound only to a cancelled approval has a contract digest no live approval carries, so the existing contract check already reports it stale; once history loads, status and ship-status name the re-review instead of calling the evidence unreadable, and finalize refuses it as any stale review. (An explicit send-back check in currency was prototyped and removed: it is unreachable, because a byte-identical re-approval records a live approval with the same digest.)

Implementation: `definition_approver_for_review_contract(approvals, review)` keeps its behaviour and delegates to `definition_approver_bound_from(approvals, review, first)` with `first = send_back_cancelled_prefix(approvals)`; history validation calls `historical_definition_approver_for_review`, which passes `first = 0`. No new public API.

Out of scope, noted for Leif: after a byte-identical re-approval, a review recorded before the send-back binds the new approval by digest and can count again if nothing else about the change moved. That is main's existing behaviour, not part of the wedge in #914.

Rejected: dropping pre-send-back attempts at send-back time (CHANGE-12.c forbids losing evidence, and committed history forbids it anyway); relaxing currency (a review bound to a cancelled approval must never let finalize through).
