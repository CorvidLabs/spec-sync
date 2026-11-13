---
change: change-ship-status-exits-1-when-it-reports-a-blocker
artifact: testing
---

# Testing

- `ship_status_exits_one_when_it_reports_a_blocker` (tests/integration/change.rs): an approved,
  unverified change. Text `ship-status <id>` prints `Blocker:` and exits 1; `--json` prints one
  parseable document with a non-empty `blockers` list and exits 1; the roster form, text and
  JSON, exits 1. Red on main: every form exits 0.
- Three existing tests that ran ship-status on a blocked change through a success-asserting
  helper now assert exit 1 and read the same report:
  `a_change_reviewed_before_its_send_back_is_reviewed_again_and_finalizes`,
  `a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged`,
  `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`.
- Ready-to-finalize ship-status still exits 0 (the last test's `ready` read asserts success).
