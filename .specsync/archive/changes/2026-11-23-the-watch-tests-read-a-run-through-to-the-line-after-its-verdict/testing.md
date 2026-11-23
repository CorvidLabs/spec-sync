---
change: the-watch-tests-read-a-run-through-to-the-line-after-its-verdict
artifact: testing
---

# Testing

- Unit `a_watch_run_is_finished_only_at_the_line_after_its_verdict` in
  `tests/integration/watch.rs` (fails on main: the verdict alone counts as finished).
- `watch_does_not_claim_a_pass_over_zero_specs` and the other watch tests still pass.
