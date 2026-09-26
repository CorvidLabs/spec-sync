---
id: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
state: archived
type: bug_fix
base_commit: be3d90d8a67b31204623dd48c761e9c25770636f
---

# Lifecycle commits stage only what the change owns, never every untracked file

## Intent

Lifecycle commits stage only what the change owns, never every untracked file

## Affected Canonical Specs

- `cmd_change`
- `change`

## Acceptance Criteria

- change check --commit and change ship --push never commit an untracked file outside the paths the change owns: an untracked file elsewhere in the tree stays untracked and is absent from every lifecycle commit, and each one left out is listed on standard error. Those commits still carry the change workspace, its archive package, the canonical spec and requirements files its deltas write, the sequence ledger, and every tracked edit, staged through explicit literal pathspecs rather than git add -A. The run_checked_commit doc comment states what is actually committed when the second verification fails, and that error names the materialize commit already made. Regression tests drive both commands over a tree holding an unrelated untracked file and fail if it is committed.

## No-spec Rationale

Not applicable
