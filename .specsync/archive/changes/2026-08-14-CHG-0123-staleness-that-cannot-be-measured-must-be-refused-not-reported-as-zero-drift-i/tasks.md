---
change: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
artifact: tasks
---

# Tasks

1. `MissingHistory` + `missing_history()` in `git_utils`, with the #558 strings.
2. Refactor `stale.rs` onto it; prove output is byte-identical.
3. Guard `report` after coverage, before the per-module loop.
4. Guard `check --stale`; leave plain `check` alone.
5. Block the lifecycle `no_stale` transition.
6. `GitFreshness` in scoring; withhold rather than award.
7. Integration matrix over all four fixtures.
8. CHANGELOG entry.
