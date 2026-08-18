---
change: CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness
artifact: docs
---

# Docs

## User-visible change

A spec citing a source file that no longer exists is no longer reported as
current. Previously:

```
Result:    0/1 specs are stale
✓ All specs are up to date with their source files.        rc=0
```

Now:

```
Result:    1/1 specs are stale
✗ inv — cites a source file that no longer exists
      src/invoice.rs (deleted)                             rc=1
```

`report --format json` reports `"stale": null` with `"staleness_inconclusive": true`
instead of `"stale": false, "commits_behind": 0`. `stale --format json` gains
`unmeasurable_count`, `unmeasurable_specs[]` and `deleted_source_specs[]`, and
`stale_specs[]` entries gain `deleted_files`.

The lifecycle `no_stale` guard now fails when a spec cites a deleted file.

## Upgrade note

A repository whose specs cite files that were deleted without the spec being
updated will start failing where it previously passed. That is the defect being
fixed: those specs were never current, they were unexamined. To see the affected
set before upgrading, run `specsync check` — it has always reported this
correctly and its verdict is unchanged.

Projects with `enforcement = warn` continue to exit 0 and are unaffected.

## Unchanged

- A healthy spec still reports the all-clear.
- Sub-threshold drift is still fresh; real drift reports the same number.
- Spec scores are unchanged — the git freshness half now reports withheld rather
  than a measured zero, but no additional penalty is applied.
