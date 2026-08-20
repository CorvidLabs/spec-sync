# Design

The principle both halves now follow: **identity comes from `state.json`, never from the shape
of a name.**

## `is_positive_legacy_tombstone` — a union, not a replacement

Three signals say "real package, therefore not a tombstone", combined with `||`:

1. It holds a regular file outside `deltas/`.
2. It holds one of the four lifecycle marker files.
3. Its name carries a lifecycle ordinal.

A union was chosen deliberately over a substitution. Each signal can only move a directory from
*skipped* to *refused*, so adding one cannot weaken the gate — which matters because the first
attempt at this change replaced the name check with the content check and did weaken it: a
dated package holding only `deltas/auth.md` went from refused to skipped. The vacuity control
caught it. That is a fail-closed behaviour traded for a fail-open one, which is the exact
pattern this release keeps paying for, so the union stands and the comment records why.

Signal 1 is strictly stronger than the four-file allowlist it joins: a package that kept only
`plan.md` was previously misread. Verified against real data — all 159 archived packages hold
at least one top-level regular file.

Signal 3 changes from `name.contains("-CHG-")` to `name_carries_a_lifecycle_ordinal`, which
accepts the undated `CHG-NNNN-...` form as well. That is the live bug fixed.

## The classifier — read the id, do not parse the name

`record_archive_path` now reads `.id` from the archived `state.json` instead of matching the
directory name against a dated-ordinal regex. When jq is unavailable or the state is
unreadable, `change_id` stays empty and the archive fast lane is withheld — no identity, no
fast lane, full matrix runs. That is the safe direction.

The review-required loop globs `*/state.json` rather than `CHG-*/state.json`. It already read
`.id` from each file; only the glob assumed a shape. The two review-path patterns take
`([^/]+)` in place of `(CHG-[0-9]{4,}-.+)`.

`record_active_path` needed no change — it already took the first path segment.

## The fixtures were stubs, and are now faithful

The classifier's archive fixtures carried `{"workflow_version":2}` with no `id`, because the
old code never needed one. Every one of the 159 real archived `state.json` carries `id` —
checked, not assumed — so the fixtures were under-specified rather than the requirement being
new. Both fixture sets were updated.

## Discrimination

Measured against a separate checkout of `origin/main`, not by reverting files in place.

```
an_undated_package_stripped_of_its_lifecycle_files_is_still_refused    FAILED on main
a_slug_named_package_stripped_of_its_lifecycle_files_is_still_refused  FAILED on main
a_deltas_only_legacy_tombstone_is_still_skipped_whatever_it_is_named   passed on both
```

The third is the vacuity control: it proves the change is not simply "refuse everything," and
it must behave identically on both binaries.

For the classifier, the new test script run against `origin/main`'s classifier:

```
expected review_required=true, got:
review_required=false
review_required_change_id=
```
