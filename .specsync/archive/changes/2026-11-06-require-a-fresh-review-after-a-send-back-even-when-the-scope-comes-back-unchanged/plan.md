---
change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
artifact: plan
---

# Plan

## Approach

Record on each new send-back how many review attempts preceded it, and make review currency refuse a review recorded before the latest send-back, with a timestamp fallback for send-backs that predate the field.

## Out of scope

Review-history validity (#914); other send-back effects; the review record schema; any relaxation of an existing currency check.

## Steps

1. The failing integration test, the #960 scenario through the binary under workflow v2, committed red on main.
2. Unit tests on the real ledger: the count is recorded; a pre-send-back review is stale with the send-back reason after an identical re-approval; a fresh review is current; a legacy send-back without the count uses the strict timestamp rule.
3. Add `cancelled_review_attempts` to `SendBackRecord` and write it in `send_back_change`.
4. Add the check to `scoped_review_currency` before the contract digest check.
5. REQ-change-123 delta, spec invariant and Public API row, CHANGELOG, lesson. Then `change check --commit` and verify.

## Risks

- A legacy send-back with a review recorded in the same second would read as stale and need one fresh review. That is conservative, and no existing archive is affected.
- A persisted-schema addition: the field is optional and serialized only when present, so older ledgers parse unchanged.

## Constraints consulted

- Change lesson z=18 (#914, "History is not currency"): only currency changes.
- CHANGE-12.b/12.c/12.e and REQ-change-123.
- Append-only `send_backs` and `review-attempts.json`; committed send-backs cannot be edited.
- The #901/#880 lesson: the repro is the failing-first integration test through the binary.
