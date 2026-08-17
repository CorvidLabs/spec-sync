---
change: CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent
artifact: context
---

# Context

`change list` and `change status` printed `No active SDD changes.` and exited 0 whenever any
single workspace was malformed — indistinguishable from an empty project — while `change show`
and `change audit` on the identical tree hard-errored with the path and `line:column`.

Measured, two changes with one corrupted:

    change list    rc=0   No active SDD changes.      <- both vanish
    change status  rc=0   No active SDD changes.      <- both vanish
    change show CHG-0001 (healthy)  rc=0, prints fine <- it demonstrably exists
    change audit   rc=1, names the file and line:column

## The mechanism, in two compounding halves

    pub fn list_changes(root: &Path) -> Vec<ChangeRecord> {
        list_changes_checked(root).unwrap_or_default()
    }

1. `list_changes_uncached` used `?` on every per-workspace failure, so enumeration **aborted at
   the first bad `state.json`** and healthy siblings were never collected.
2. `unwrap_or_default()` then turned the resulting `Err` into an empty list, so the failure was
   not merely unhandled — it was **unrepresentable**.

A category was empty for want of input, and the listing read that as want of changes. This is
the release's defect class in its `Result` + `unwrap_or_default()` vehicle.

## Why this is also #603

#603 was filed as "5.2 silently strips `workflow_version` from 6.0-written records", concluding
that 6.0 must gain a way to detect the downgrade. That conclusion was wrong, and testing it is
what showed why.

**6.0 already detects it, precisely:**

    error: workflow-v1 change CHG-0001-… was not present at the trusted
           pre-v2 cutoff f7f7f3e66050c264e22f07cd8d2b353d82e03140

`validate_workflow_v2_baseline` catches it because `.specsync/workflow-v2-baseline.json` is a
repo-level file a 5.2 binary has never heard of and therefore does not strip. The existing
`(2, None)` anchor check cannot fire — 5.2 strips *both* version fields symmetrically, leaving a
record indistinguishable from a legitimate v1 — but the baseline cutoff closes exactly that hole,
and it works.

`show` and `audit` reported it. `list` and `status` swallowed it through the same roster
`unwrap_or_default()` and reported the change **absent rather than damaged**.

So no version stamp had to move. The refusal already existed and was being discarded.

## Sibling sites

The bug report named `list`/`status`. Three further callers were reading the same empty roster
as fact, and were fixed in the same change because the site named in a report has survived
alone seven times in this release:

- `policy_at_comparison_base` selected the pull-request diff base from the roster, so an
  unreadable workspace silently changed which base a policy was compared against.
- `sibling_active_change_ids` reported no other changes in flight when it could not tell.
- `ship` inferred which change to ship from whatever remained readable. **This one writes
  commits.**

## Ruled out

Making `list_changes` return `Result<Vec<ChangeRecord>>` and letting callers `?`. That converts
"one workspace is broken" into "you may not see any of your changes", which is the same failure
with a different exit code. The roster has to carry both facts.
