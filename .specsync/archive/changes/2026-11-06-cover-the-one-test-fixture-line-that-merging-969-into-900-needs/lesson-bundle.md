# Lesson bundle: cover-the-one-test-fixture-line-that-merging-969-into-900-needs

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cover the one test fixture line that merging #969 into #900 needs
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change_tests/send_back_for_rescope.rs
- **Acceptance**: A review an agent records for a person can carry the same record, shown the same way.
- **Acceptance**: The change's status shows it beside the approval or review it belongs to.

## Evidence

- Verification commit: `f471c6a922ac76fecc9d5382e0e68c4540a7c56f`
- Base commit: `a20353b988db27df02d3cd535e48150939fba183`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#900 (record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication, archived at 1a8d5048) adds `relayed_by` to `ScopedReviewRecord`. #969 (#960, on main at 339bee65) added the test `a_send_back_answers_review_order_by_attempt_count_and_falls_back_to_time` in `src/change_tests/send_back_for_rescope.rs`, which builds a `ScopedReviewRecord` as a struct literal. Merging main into #900 at a20353b9 therefore did not compile until that literal got `relayed_by: None`. That is the same line #900 already added to its own fixtures in `substantive_plan.rs` and `archive_post_move_failure_restores_exact_source_bytes_without_residue.rs`, and the merge commit carries it.

With the line, the verify lane passed (2870 unit, 514 integration), but `specsync change audit --strict` refused: "meaningful changed paths are not covered by an active change: src/change_tests/send_back_for_rescope.rs". #900's archived scope lists test files one by one and this one is not among them, and the first covering change (cover-the-two-paths-900-changed-outside-its-declared-scope) declared two other files. Any merge of main that includes #969 needs this line, so waiting does not avoid it.

On 2026-11-06, relayed by orc t18, Leif chose this second covering change: no spec change, spec `change`, that one path, and acceptance criteria SHIP-2.e and CHANGE-13.b verbatim.

Ruled out: sending #900 back to add the path (it would cancel the approval and repeat the whole cycle for the same result), editing either archived package, and changing the test's assertions.

## From the change's testing.md

# Testing

The line changes no behavior. It gives #969's review fixture the field every `ScopedReviewRecord` now has, set to "no relay". The tests that already cover the record and its status:

| Criterion | Test |
| --- | --- |
| SHIP-2.e | `a_relayed_review_finalizes_and_its_review_digest_leaves_the_relay_out` and `an_approval_and_a_review_without_a_relay_omit_the_record` in `tests/integration/relayed_decision.rs`: a review carries the relay record and finalizes, and a review without one omits the key. `the_review_digest_leaves_the_relay_out_and_keeps_existing_digests` in `src/change_tests/relayed_decision.rs` shows a record without a relay keeps its digest. |
| CHANGE-13.b | `status_shows_the_approval_and_review_claims_with_the_relay_and_its_boundary` in `tests/integration/relayed_decision.rs`: status shows each relay beside the approval or review it belongs to. |

`a_send_back_answers_review_order_by_attempt_count_and_falls_back_to_time` passes with the line, as do the other send-back tests (9 passed), and the test binary does not compile without it. The verify lane passed on a20353b9 (2870 unit, 2 ignored; 514 integration). After the check commit it runs again with `change audit --strict` and pre-push, and the audit must pass on the archive tip before push.

## Where these lessons go

- `specs/change/change.3md`
