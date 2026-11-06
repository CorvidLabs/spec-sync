---
change: cover-the-one-test-fixture-line-that-merging-969-into-900-needs
artifact: context
---

# Context

#900 (record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication, archived at 1a8d5048) adds `relayed_by` to `ScopedReviewRecord`. #969 (#960, on main at 339bee65) added the test `a_send_back_answers_review_order_by_attempt_count_and_falls_back_to_time` in `src/change_tests/send_back_for_rescope.rs`, which builds a `ScopedReviewRecord` as a struct literal. Merging main into #900 at a20353b9 therefore did not compile until that literal got `relayed_by: None`. That is the same line #900 already added to its own fixtures in `substantive_plan.rs` and `archive_post_move_failure_restores_exact_source_bytes_without_residue.rs`, and the merge commit carries it.

With the line, the verify lane passed (2870 unit, 514 integration), but `specsync change audit --strict` refused: "meaningful changed paths are not covered by an active change: src/change_tests/send_back_for_rescope.rs". #900's archived scope lists test files one by one and this one is not among them, and the first covering change (cover-the-two-paths-900-changed-outside-its-declared-scope) declared two other files. Any merge of main that includes #969 needs this line, so waiting does not avoid it.

On 2026-11-06, relayed by orc t18, Leif chose this second covering change: no spec change, spec `change`, that one path, and acceptance criteria SHIP-2.e and CHANGE-13.b verbatim.

Ruled out: sending #900 back to add the path (it would cancel the approval and repeat the whole cycle for the same result), editing either archived package, and changing the test's assertions.
