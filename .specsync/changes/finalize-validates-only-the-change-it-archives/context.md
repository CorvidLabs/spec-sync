---
change: finalize-validates-only-the-change-it-archives
artifact: context
---

# Context

Issue #856. `change finalize` and `change ship` re-validated every other archived change's workflow-version history each time they archived one change, so their cost grew with the number of archives and with the length of history. On this repository (about 130 archives) `change ship` took 3 to 11 minutes while `change check --commit` took about 30 seconds, and sampling showed back-to-back `git` processes from the archive walk. A downstream consumer's CI Archive job grew with the commits in the PR and timed out at 91 commits. #840 (read cache) and #842 (finalize memo, REQ-change-121) made the walk cheaper but kept it. The walk lives in `list_all_changes_uncached`, which calls `validate_workflow_version_history` (and the parallel prefetch) on every archived record.

This branch stacks on #842 (`perf/fast-finalize`). Leif asked for it in 6.2.0. Already-archived changes were validated when they were archived, so the per-archive walk protects nothing new during a finalize. The issue moves it to `change audit --strict` and keeps every refusal for the change being archived.
