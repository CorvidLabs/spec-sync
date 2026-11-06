---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: plan
---

# Plan

## Approach

Add an ancestor-aware git merge driver, `specsync merge-file`, and register it with `specsync merge --install`. The driver unions `files`, `db_tables`, and `depends_on`, counts both numeric version bumps from the ancestor, unions change-log rows, and unions requirement blocks. The same new requirement ID with two bodies is kept twice. `check` and `change audit` then refuse and name that ID.

## Out of scope

Issues #433, #657, and #532. The marker resolver's `max(version)` rule and its refusal of prose hunks. Looser gates, timeouts, or weaker assertions. The 6.2.0 release candidate.

## Steps

Write the failing `git merge-tree` fixture and the duplicate-ID refusal test. Implement the driver and the refusal. Keep the module specs in sync with the new exports.

## Risks

A clone that commits the attribute rules but does not run `merge --install` keeps git's ordinary merge, so those specs still conflict until the driver is registered. A broken driver command would turn a clean merge into a conflict, which is why install records the absolute path of this executable and can be run again after an upgrade.

## Constraints consulted

The marker resolver still takes max(version) and still refuses prose. MERGE-4: existing strictness stays.
