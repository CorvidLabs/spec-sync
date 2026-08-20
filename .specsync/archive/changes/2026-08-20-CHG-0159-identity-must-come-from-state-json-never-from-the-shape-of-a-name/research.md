# Research

Both defects are live on `0387678f`, measured rather than predicted.

## The tombstone check is already wrong, today

`is_positive_legacy_tombstone` used `name.contains("-CHG-")` to mean "this is a real lifecycle
package, so it is not a tombstone." That substring test does not hold for the undated form:

```
2026-08-19-CHG-0001-foo      contains('-CHG-') = True
CHG-0001-foo                 contains('-CHG-') = False
```

So an archived package named `CHG-0001-foo` that has lost its `state.json` and its four marker
files, while keeping `deltas/*.md`, is classified as a legacy tombstone and silently skipped by
`list_all_changes_uncached` (`src/change.rs:15004`) and `located_change_sequences` (`:1918`) —
instead of being refused as corrupt. The CI harness already carries an undated
`.specsync/archive/changes/CHG-0001` fixture (`test-classify-ci-paths.sh:10`), so the shape is
not hypothetical.

The neighbouring test `archive_directory_with_files_but_no_state_is_still_refused` states the
principle the check was supposed to enforce: *"A file git could track means the checkout theory
does not apply: this package is damaged, not absent, and skipping it would hide corruption."*

## The CI classifier gates the mandatory review on a glob

`.github/scripts/classify-ci-paths.sh:433`:

```bash
for state_path in "$root"/.specsync/changes/CHG-*/state.json; do
```

`ci.yml:616` runs the one independent implementation review only when
`needs.classify.outputs.review_required == 'true'`, and that value comes from what this loop
counts. Any identity shape the glob misses yields `review_candidates=0`,
`review_required=false`, and a pull request that merges with no review — while CI goes green
*faster*, which is the worst possible shape for a gate failure.

Measured against `origin/main` with a slug-named active change carrying complete, stale-review
evidence:

```
review_required=false
review_required_change_id=
```

The loop already reads `.id` from `state.json` two lines later. The glob was the only thing
that did not.

Three sibling regexes have the same defect in the milder direction — `:246` parses the archive
directory name as `^[0-9]{4}-[0-9]{2}-[0-9]{2}-(CHG-[0-9]{4,}-.+)$`, and `:275`/`:282` require
`CHG-[0-9]{4,}-` in a review path. Those fail *closed* (the archive fast lane is withheld and
the full matrix runs), so they are wasteful rather than dangerous, but they are the same
mistake and are fixed in the same pass.

## What replaces the name, and what does not

For the classifier the answer is complete: identity comes from `state.json`, which every one of
the 159 archived packages carries — verified, not assumed.

For the tombstone check it is only partial, and this change says so rather than pretending
otherwise. A dated package reduced to `deltas/auth.md` alone is refused today *purely* because
of its name; content cannot distinguish it from a genuine `deltas/`-only legacy tombstone.
Signals 1 and 2 do not replace signal 3. Retiring the ordinal will need a provenance signal in
its place — git history of the package's `state.json` is the obvious candidate — and that
belongs to the identity migration, not here.
