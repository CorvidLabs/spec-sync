---
id: reuse-one-cat-file-batch-reader-per-repository-root-across-nested-repository-fact-scopes
state: draft
type: bug_fix
base_commit: ad50bdef3db58a715877877d2407485bcaed0f32
---

# Reuse one cat-file batch reader per repository root across nested repository-fact scopes

## Intent

Reuse one cat-file batch reader per repository root across nested repository-fact scopes

## Affected Canonical Specs

- `git_utils`

## Acceptance Criteria

- A batched read inside a scope nested in an open same-root scope reuses the outer scope's cat-file --batch reader instead of starting a second one (REQ-git-utils-010)
- A failing unit test shows two batch children for one root before the fix and one after
- Invalidation still closes every frame's reader, so a read after a git write sees the new state
- Per-command spawn ceilings in lifecycle_commands_stay_within_git_spawn_ceilings still pass and no count rises

## No-spec Rationale

Not applicable
