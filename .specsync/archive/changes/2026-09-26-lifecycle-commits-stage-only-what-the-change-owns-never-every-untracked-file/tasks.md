---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: tasks
---

# Tasks

- [x] Add `LifecycleCommitScope` / `lifecycle_commit_scope` to the change domain, with an exact-set test.
- [x] Replace `git_commit_all` (`git add -A`) with literal-pathspec staging of tracked edits plus owned untracked paths.
- [x] Leave unmerged entries and runtime files unstaged, and list every other left-out untracked file on stderr once per run.
- [x] Correct `run_checked_commit`'s doc comment and name the materialize commit in the second-pass error.
- [x] Regression tests for `check --commit` and `ship --push`, each shown to fail with `git add -A` restored.
- [x] Unit tests for the porcelain parse, the scope boundary, and the warning.
- [x] Deltas, Public API rows, ADOPTING/AGENTS docs, and companion notes for `change` and `cmd_change`.
- [x] `fledge lanes run verify` and `fledge trust verify`.
