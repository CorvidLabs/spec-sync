---
change: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
artifact: plan
---

# Plan

## Approach

Add an optional `relayed_by` record (`RelayedDecision`) to `ApprovalRecord` and `ScopedReviewRecord`. It is written by `change approve` and `change review` when `--words`, `--where` and `--recorded-by` are given, shown by `change status`, and covered by no digest. Narrow the HI and requirement wording so an approval reads as a recorded claim bound to content.

## Out of scope

- Authentication, signing or `attest verify` positioning (#633's residual).
- `relayed_by` on reopen, send-back, correct-owner, reconcile-history and the hidden `accept`.
- Showing SpecSync's record in Chough output (chough#123 follow-up).

## Steps

1. Write failing CLI tests for CHANGE-13 to 13.d, CHANGE-6.d and SHIP-2.e, and a unit test for the review digest.
2. Add `RelayedDecision`, its strict read-side validation and the flag builder.
3. Thread the relay through `approve` (both projections) and `review`; compute the review digest without it.
4. Add `recorded_claims` and the status text and JSON.
5. Update the CLI help, README, workflow and CLI guides, and the CHANGELOG.

## Risks

- `ScopedReviewRecord` is a digest preimage. The finalization review digest therefore hashes the record with `relayed_by` cleared, so records without a relay keep their digest and a 6.x binary that drops the key computes the same one.
- An older binary that rewrites `approvals.json` drops the unknown key, as it already does for every newer field.
- Strict read-side validation makes a hand-edited relay record unreadable. The refusal names `git checkout <commit> -- <path>`.
- #889 adds rows to the change spec's Public API. If it merges first, the Public API delta is rewritten onto it before approval.

## Constraints consulted

change.3md "Release review: claims and authentication"; the forward-compatibility note in `sdd_version.rs` (digest preimages); never weaken a gate, bound or test.
