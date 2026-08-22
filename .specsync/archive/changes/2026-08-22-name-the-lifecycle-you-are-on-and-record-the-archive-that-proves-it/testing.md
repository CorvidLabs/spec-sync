---
change: name-the-lifecycle-you-are-on-and-record-the-archive-that-proves-it
artifact: testing
---

# Testing

## Discrimination — separate checkout, per protocol

Baseline worktree at `4e861401`:

    a_squash_merged_archive_is_recorded_under_its_archive_path ... FAILED
    an_archive_absent_from_the_default_branch_is_not_recorded   ... ok

    test result: FAILED. 1 passed; 1 failed

The discriminator fails on the old binary; the control passes on BOTH. That combination is what
makes the widening bounded rather than vacuous — a predicate rewritten to match everything would
pass the discriminator and fail the control.

## The corpus measurement is the stronger evidence

Unit tests prove the mechanism; the corpus proves it matters and is safe. All 172 archives:

    before:  anchored =  71 / 172   commit_current = 19   on_remote_default =  68
    after:   anchored = 172 / 172   commit_current = 19   on_remote_default = 172

Zero archives move from valid to invalid.

## Announcement controls

Upgrade path reproduced in a clean fixture (synthetic 5.2.0-era policy, `version: 1`):

    $ specsync init
      ! this project is on workflow v1 (legacy) — new changes will use `change accept`/...
    $ specsync change new ...
      ! workflow v1 (legacy) — this change uses `change accept` and `change archive`, ...

CONTROL — a workflow-v2 repository must stay silent, or this ships noise on every verb in every
healthy project:

    $ specsync init            -> "already exists"                (nothing added)
    $ specsync change new ...  -> identity, state, next action     (nothing added)
    $ specsync change list     (this repository, 172 archives)     (nothing added)

## Suite

`cargo test`: 2347 unit + 405 integration, 0 failed. `cargo fmt --check` clean. `specsync check`:
104/104 exports, 62 specs passed, 0 warnings.

## Not covered

Whether the archived path SHOULD additionally check `verification.commit` reachability. That is the
remaining open question on #677, deliberately untouched: this makes the existing predicate answer
correctly, it does not decide what else the archived path ought to verify.
