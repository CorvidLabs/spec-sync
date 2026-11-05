---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: plan
---

# Plan

## Approach

Cut the git processes SpecSync starts per command (memoize repository facts, batch object reads), prove it with spawn-count tests, and run the release-candidate suite under a pinned cargo-nextest.

## Out of scope

- Returning the qualify timeout from 75 to 45 minutes. That needs a measured macOS qualify run of a candidate carrying this change; a one-line follow-up does it once that run is well under 45 minutes (see scope question).
- Changing what any test asserts, or removing, ignoring or filtering any test.
- A libgit2 dependency, larger runners, or the `change audit --archives` walk.

## Steps

1. Add the repository-fact memo and invalidation to `git_utils`, with unit tests for invalidation after commit, index write and ref move.
2. Route the repeated fact reads in `change/` and `commands/change.rs` through it.
3. Add the batched `cat-file --batch` reader and route `show <sha>:<path>`, `show HEAD:<path>` and `ls-tree` reads through it.
4. Add the spawn-ceiling tests; measure the slowest test again.
5. Add the `test-nextest` task, switch `lanes.release-candidate`, and install pinned cargo-nextest in `release.yml`.
6. Record before and after numbers in `testing.md`; update specs, RELEASING.md and CHANGELOG.

## Risks

- A stale memo after a write would give a wrong answer; mitigated by one invalidation point called from every writing path, and tests that commit mid-command.
- A long-lived `cat-file` child could leak or deadlock on large blobs; mitigated by reading exact sizes from the batch header and closing the child at command end.
- nextest runs each test in its own process; the full suite already passes under it locally.

## Constraints consulted

SPEED-1.c: a cache needs a correct invalidation rule. git_utils lessons: git failures return safe defaults, and `git_cmd` is the single cleared-environment spawn point.
