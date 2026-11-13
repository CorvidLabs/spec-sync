---
change: the-documented-workflow-test-needs-an-instruction-to-run-the-verify-lane-before-the-human-review
artifact: testing
---

# Testing

- Integration `a_prose_line_that_only_mentions_the_verify_lane_does_not_run_it` and
  `the_human_review_must_come_after_the_verify_lane` in `tests/integration/documented_workflows.rs`
  (both fail on main: the mutated block passes).
- The README and landing-page workflow tests still pass.
