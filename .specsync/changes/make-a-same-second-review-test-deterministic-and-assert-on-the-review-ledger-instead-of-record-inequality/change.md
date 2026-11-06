---
id: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
state: draft
type: bug_fix
base_commit: 5181fb2f6952ef086eed6cb0dff976b41123cd77
---

# Make a same-second review test deterministic and assert on the review ledger instead of record inequality

## Intent

Make a same-second review test deterministic and assert on the review ledger instead of record inequality

## Affected Canonical Specs

- None

## Acceptance Criteria

- With now() pinned to one second through a test-only clock (compiled only into test builds, so release builds behave exactly as before), a_review_recorded_before_a_send_back_never_counts_after_an_identical_reapproval fails on every run at assert_ne!(fresh, earlier) before the fix.
After the fix the test asserts that review-attempts.json grew by exactly one attempt and that its last entry is the fresh review and is current, and it passes on every run, pinned or not.
Every other assertion in the test is unchanged and still passes in the same second: the send-back records one cancelled review attempt, the earlier review is stale for the send-back reason, and the fresh review is current.
No product code path changes; no test is dropped or weakened (SPEED-2.b.2).

## No-spec Rationale

Test-only: a clock pin compiled only into test builds and one unit-test assertion; no contract, public behaviour or release build changes
