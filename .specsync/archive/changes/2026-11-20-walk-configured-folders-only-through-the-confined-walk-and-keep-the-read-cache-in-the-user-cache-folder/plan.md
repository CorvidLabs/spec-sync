---
change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
artifact: plan
---

# Plan

## Approach

Failing tests first, on `4516f355` with only the tests added. Then the fix.

## Out of scope

Other directory listings (`.specsync/changes`, the archive and similar): these are SpecSync's own record folders, which the lifecycle already reaches through link-checked paths.

## Steps

1. Tests.
2. `util` and `local_state`.
3. Route each walk.
4. Specs, CHANGELOG, docs.
5. The verify lane and Linux, through one-lane.

## Risks

The first archived-history read after upgrading is cold.

## Constraints consulted

`util.3md`; Leif's delegation.
