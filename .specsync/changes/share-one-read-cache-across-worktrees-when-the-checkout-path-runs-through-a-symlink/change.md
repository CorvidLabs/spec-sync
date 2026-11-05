---
id: share-one-read-cache-across-worktrees-when-the-checkout-path-runs-through-a-symlink
state: approved
type: bug_fix
base_commit: 6b222ff623079b0b3e328db29e8df078cb80a60d
---

# Share one read cache across worktrees when the checkout path runs through a symlink

## Intent

Share one read cache across worktrees when the checkout path runs through a symlink

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a_second_worktree_reads_warm_from_the_shared_cache passes on macOS: the main and a linked worktree resolve the same persistent cache path even when the checkout path runs through a symlink.

## No-spec Rationale

REQ-change-119 already requires the cache under the Git common directory shared by worktrees; this resolves that directory to its canonical path so a symlinked checkout path (macOS /var -> /private/var) names the same file. No contract text changes.
