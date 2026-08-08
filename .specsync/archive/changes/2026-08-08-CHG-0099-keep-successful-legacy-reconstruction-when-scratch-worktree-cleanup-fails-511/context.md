---
change: CHG-0099-keep-successful-legacy-reconstruction-when-scratch-worktree-cleanup-fails-511
artifact: context
---

# Context

Product #511: `reconstruct_legacy_at_anchor` discarded a successful reconstruction
when disposable scratch worktree removal failed. Under CI worktree contention that
turned transient hygiene failures into reconstruction failures, failing closed when
two identical anchors both hit cleanup errors.

## Fix

Always return the reconstruction `result`. Cleanup is best-effort: attempt
`git worktree remove --force`, and on failure `git worktree prune` plus
`remove_dir_all` of the temporary root. Test hook forces remove failure to prove
Ok is preserved.
