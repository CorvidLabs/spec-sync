---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: docs
---

# Docs

- `docs/ADOPTING.md`, section "Drive one real change end to end", gains a paragraph. It says
  `check --commit` and `ship --push` commit tracked edits plus the change's own untracked files,
  never any other untracked file, and that a new source file has to be `git add`ed before
  `check --commit`.
- `AGENTS.md` Quick Reference row for `change check [id] --commit` says the same in one clause.
- `specs/cmd_change/requirements.md`: REQ-cmd-change-012 no longer names `git add -A`, and
  REQ-cmd-change-017 is added (delta `deltas/cmd_change.md`).
- `specs/change/requirements.md`: REQ-change-102 is added, and `specs/change/change.spec.md`
  Public API documents `LifecycleCommitScope` and `lifecycle_commit_scope` (delta
  `deltas/change.md`).
- Companion context, testing and tasks notes are updated for both modules. The stale
  `git_commit_all` reference in `specs/change/context.md` now names `git_commit_lifecycle`.

Reader-visible behavior change: a lifecycle commit now prints a `warning: left N untracked
file(s) out of this lifecycle commit` block on stderr when it leaves files out. Standard output,
including `--format json`, is unchanged.
