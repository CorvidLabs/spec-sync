---
change: CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires
artifact: design
---

# Design

Single dispatch path in `cmd_change` for `Check { commit, push }`:

1. If `push && !commit` → hard error.
2. If `commit` → `run_checked_commit`: check → `git add -A` + materialize commit
   (no-op if empty) → check again → evidence commit → optional push.
3. Else existing single-pass `check_change`.

Commits use fixed message prefixes:
- `chore(lifecycle): materialize {id}`
- `chore(lifecycle): record {id} verification`

`git add -A` is intentional for the materialize step so delta application and
spec updates land together; callers must keep the worktree free of unrelated dirt.
