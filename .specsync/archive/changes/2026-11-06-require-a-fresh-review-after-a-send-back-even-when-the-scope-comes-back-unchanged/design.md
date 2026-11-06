---
change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
artifact: design
---

# Design

A send-back already records `cancelled_approvals`, the number of approvals before it, which is how the definition gate ignores an approval the send-back cancelled. The review side gets the same treatment.

- **Record:** `SendBackRecord` gains `cancelled_review_attempts: Option<usize>`, the number of entries in `review-attempts.json` when the send-back is written. It is written on every new send-back and is absent on send-backs recorded before this change. Old records keep their committed bytes; the committed-send-back immutability check is unchanged.
- **Currency:** `scoped_review_currency` refuses a review that was not recorded after the latest send-back, before the contract digest check, with a stale reason naming the send-back and a fresh `change review`.
  - When the latest send-back carries the count *n*, the review counts only if it is one of the attempts after the first *n*.
  - When it does not (a send-back recorded by an older SpecSync), it counts only if its timestamp is strictly later than the send-back's. Every archive on main with a send-back has its review recorded after it, so no existing verdict changes.
- **Unchanged:** review-history validity (#914) still binds any recorded approval. The CHANGE-12.b check, the contract, execution, project-input and commit checks are unchanged. No gate, bound or timeout is relaxed.

Rejected:
- Timestamps alone: a send-back and a fresh review can land in the same second in scripted flows and tests, which makes the order ambiguous.
- Binding a review to an approval ledger index: that needs a new field on every review record, and an old review would still need a fallback. The send-back is the event that cancels, so it is where the count belongs, as with `cancelled_approvals`.
- Dropping or rewriting earlier attempts: CHANGE-12.c and append-only history forbid it.
