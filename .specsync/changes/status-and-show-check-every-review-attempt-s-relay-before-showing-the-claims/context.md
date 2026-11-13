---
change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
artifact: context
---

# Context

Issue #974, a Codex P2 on #900 deferred by Leif's call. `recorded_claims`
(`src/change/relayed_decision.rs`), the projection `change status` and `change show` print,
read `approvals.json` and only the latest `review.json`. Between `change review` and
`change ship`, `review-attempts.json` is uncommitted by design, so an earlier attempt with a
malformed relay, or one naming a different reviewer, was shown as fine until a later gate read
it. `load_scoped_review` checks every attempt, but status does not go through it.

The fix reads the attempt ledger through `load_scoped_review_attempt_ledger`, which applies the
same relay checks and recovery as `load_scoped_review` and accepts a change that has no attempt
history yet. REQ-change-relayed-decision-001 already says reading refuses a damaged
`relayed_by` and names the recovery. No requirement text changes.
