---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: research
---

# Research

## Where the sweep happened

- `src/commands/change.rs` `git_commit_all` ran `git add -A`. It had three callers: the
  materialize and verification-evidence commits in `run_checked_commit` (`check --commit`), and
  the archive commit in `ship_commit_and_push_archive` (`ship --push`).
- Nothing else in the lifecycle stages files. `finalize` and plain `check` only write the working
  tree.

## What the lifecycle writes between `change new` and the archive commit

- `create_change`: the workspace, and `.specsync/workflow-v2-baseline.json` when the project was
  just adopted (`ensure_workflow_v2_baseline`).
- `materialize_change_deltas`: each affected spec's canonical spec file and `requirements.md`
  (`prepare_pending_delta_application` via `canonical_module_paths`), plus the workspace
  `state.json` and `change.md`.
- `verify_change_locked`: `verification.json`, `verification-attempts.json` and `state.json`
  in the workspace.
- The staging path: `.specsync/change-sequence.json` (`floor_sequence_ledger_to_committed`).
- `finalize_change`: moves the workspace into `.specsync/archive/changes/<date>-<id>/` and
  writes `accepted-state.json`, `finalization.json` and `lesson-bundle.md` there.
- Every locked operation: `.specsync/change.lock`, and `.specsync/change-transaction.json` while
  a transaction is in flight. Both are volatile to the project-input digest and ignored by
  `init`'s `.specsync/.gitignore`.

## Why tracked edits stay staged

`project_input_digest` walks `git ls-files --cached --others --exclude-standard` and hashes the
working-tree content. If a tracked edit were verified but left uncommitted, CI would check out a
tree the evidence does not describe.

## Measured on the unfixed code

With staging put back to `git add -A`, both new regression tests fail on `debug-dump.zip`, and the
unfixed history also carries `.specsync/change.lock`. With only the archive commit put back, the
ship test still fails, so it guards that path by itself.
