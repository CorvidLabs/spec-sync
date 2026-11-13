---
change: the-persistent-read-cache-stays-bounded
artifact: plan
---

# Plan

## Approach

Bound the persistent read cache three ways: no approvals bytes, at most 4096 archive-introduction
entries dropped least recently used first, and no loading of a file over 8 MiB.

## Out of scope

The workflow-history, legacy-introduction and cutoff sets, which are already bounded by the
archive count or by `MAX_CHANGE_READ_CACHE_ENTRIES`; dropping unreachable commits; renaming the
cache file.

## Steps

1. Red tests for the three bounds.
2. `IntroducedApprovals` and `ArchiveIntroduction::committed_ledger`; the two ledger consumers
   read through it.
3. Cache entries record presence, a clock and `last_used`; flush stamps and bounds; load skips a
   file over 8 MiB; schema 2.
4. Targeted and broad unit tests, fmt and clippy.

## Risks

A warm read must not gain Git processes: ledgers are read only for a change with more than one
introduction. A ledger re-read that fails is an error, as any Git read failure is; the cache is
off wherever a blob could be missing. A history with more than 4096 archive-introducing commits
cannot keep every entry warm; it pays the cold reads for the rest, as before the cache existed.

## Constraints consulted

REQ-change-119 (cache rules, output identical to live reads), REQ-change-121 (cold finalize costs
at most three reads per other archive; warm costs no more as archives grow), REQ-change-134
(build identity), REQ-change-135 (archive-introduction index).
