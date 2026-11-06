---
id: cover-the-one-test-fixture-line-that-merging-969-into-900-needs
state: implementing
type: bug_fix
base_commit: a20353b988db27df02d3cd535e48150939fba183
---

# Cover the one test fixture line that merging #969 into #900 needs

## Intent

Cover the one test fixture line that merging #969 into #900 needs

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A review an agent records for a person can carry the same record, shown the same way.
- The change's status shows it beside the approval or review it belongs to.

## No-spec Rationale

Merging main through #969 into #900 needs one line in #969's new test a_send_back_answers_review_order_by_attempt_count_and_falls_back_to_time: its ScopedReviewRecord literal gets relayed_by: None, the field #900 adds, as #900's own fixtures already do. Without it the merged tree does not compile. This change only declares that file; no spec text and no other code changes.
