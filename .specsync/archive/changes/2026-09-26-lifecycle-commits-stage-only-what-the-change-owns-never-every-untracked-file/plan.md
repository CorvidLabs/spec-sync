---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: plan
---

# Plan

1. Add `LifecycleCommitScope` and `lifecycle_commit_scope` to `src/change.rs`, which owns the
   ownership decision, and a domain test that asserts the exact owned and runtime sets.
2. Replace `git_commit_all` with `git_commit_lifecycle` and `stage_lifecycle_paths` in
   `src/commands/change.rs`: stage tracked edits plus owned untracked paths through literal
   pathspecs, skip unmerged and runtime entries, and return what was left out.
3. Disclose left-out files on stderr from `run_checked_commit` (once per run, with the re-check
   guidance) and from `ship_commit_and_push_archive`.
4. Correct `run_checked_commit`'s doc comment, and name the materialize commit in the second-pass
   error.
5. Add regression tests. `check --commit` and `ship --push` run over a tree with unrelated
   untracked files, and each test fails when staging is put back to `git add -A`. Controls check
   that owned files and tracked edits still land. Add unit tests for the porcelain parse, the scope
   boundary and the warning.
6. Update `docs/ADOPTING.md`, `AGENTS.md`, the two deltas, the change spec's Public API table,
   and the companion notes for both modules.
7. Run `fledge lanes run verify` and `fledge trust verify`.
