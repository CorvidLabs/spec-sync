---
change: the-delivery-blob-deadline-bounds-each-git-read-not-the-manifest-work-between-reads
artifact: plan
---

# Plan

## Approach

Arm the delivery blob stream's Git deadline around each read through `git_utils::BatchWatchdog`,
the per-read watchdog `ObjectBatch` already uses, so only Git's own answer counts against it.

## Out of scope

Reordering the acceptance manifest builder (digests first, owners afterwards), the clean symlink
and LFS pointer batch (#992), and any change to the deadline's length.

## Steps

1. Red: `open_with_deadline` (behaviour-preserving) and a test that sleeps longer than a short
   deadline between two reads.
2. Fix: make `BatchWatchdog` crate-visible with `expired()`; arm it per read in the stream.
3. Test that the watchdog stays quiet while unarmed and reports a killed stalled read.
4. Targeted delivery and git_utils tests, fmt, clippy.

## Risks

A read Git stalls must still be killed: the stalled-batch test and the new expiry test cover it.
Dropping the stream must still reap the child and stop the watchdog thread; the stream drops the
watchdog after reaping, as `ObjectBatch` does.

## Constraints consulted

REQ-change-144: the per-object bound, the path-count bound and the Git read deadline are
unchanged, and the manifest and input digest stay byte-identical. REQ-git-utils-010: the batch
reader's own behaviour does not change.
