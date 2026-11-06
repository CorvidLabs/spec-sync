---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: context
---

# Context

Issue #862: two branches that each finalize a change to the same module conflict on that module's spec. Each materialize writes `version: N+1`, one change-log row, any new `files:` entries, and a new requirement block at the same place, so the second merge conflicts.

Leif confirmed MERGE-1 through MERGE-4 on 2026-11-05. Those sentences are in `hi/merge.md`. They are the only acceptance criteria.

The marker resolver in `src/merge.rs` still takes `max(version)` and still refuses prose, because a conflict hunk has no ancestor. The new driver in `src/merge_driver.rs` has the ancestor, so it can count both bumps. `specsync merge --install` registers it for `git merge-tree`. GitHub's merge button does not run a custom driver.

This change is waiting on Leif to record `change approve` and `change review`. Those were not recorded here.
