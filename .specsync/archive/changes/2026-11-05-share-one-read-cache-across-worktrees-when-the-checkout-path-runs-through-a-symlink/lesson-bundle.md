# Lesson bundle: share-one-read-cache-across-worktrees-when-the-checkout-path-runs-through-a-symlink

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Share one read cache across worktrees when the checkout path runs through a symlink
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/read_cache.rs
- **Acceptance**: a_second_worktree_reads_warm_from_the_shared_cache passes on macOS: the main and a linked worktree resolve the same persistent cache path even when the checkout path runs through a symlink.

## Evidence

- Verification commit: `be6c1eafec38d502fc05622c9cdfd85898bf08df`
- Base commit: `6b222ff623079b0b3e328db29e8df078cb80a60d`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

`a_second_worktree_reads_warm_from_the_shared_cache` (#840) fails on macOS, where the release candidate is qualified: the main worktree's `git rev-parse --git-common-dir` is the relative `.git`, joined onto the root as given (`/var/...`), while a linked worktree gets Git's absolute canonical path (`/private/var/...`). The two worktrees then use different cache files. CI's product lane is Ubuntu-only, so it never saw this.

## From the change's testing.md

# Testing

`cargo test --bin specsync -- a_second_worktree_reads_warm_from_the_shared_cache` failed on macOS before the fix and passes after it.

## Where these lessons go

- `specs/change/change.3md`
