# Plan

Two halves, producer and consumer. Fixing either alone leaves the class alive: the producer
fix does not help a repository that already has a husk on disk, and the consumer fix leaves
the tool still writing new ones.

## Producer — stop creating untrackable directories in committed history

`archive_change_with_options` moves the active workspace to the dated destination with
`rename_durable`, carrying the empty `deltas/` with it. Add `prune_empty_package_directories`,
called after `validate_archived_integrity` succeeds so every rollback path above still restores
an intact source.

The prune is general rather than `deltas`-specific — any directory in the package holding no
regular file at any depth — so a subdirectory added later cannot reintroduce the class. It is
deepest-first, so a parent emptied by its children goes in the same pass, and best-effort: a
failed `remove_dir` must not undo an archive that already validated.

The active workspace keeps its eager `deltas/`. Authors and agents write
`.specsync/changes/<id>/deltas/<module>.md` directly, so removing the directory would break
authoring, and the active reader already tolerates the husk on purpose.

## Consumer — read an empty directory as an absent change, not a damaged one

Add `is_untrackable_husk`: a directory holding no regular file at any depth. Wire it into the
two archive-side readers that hard-fail, beside the existing legacy-tombstone allowance:

| site | verb it breaks |
|---|---|
| `located_change_sequences`, archived arm | `change new` |
| `list_all_changes_uncached` | `change audit`, `check`, `change adopt` |

`is_positive_legacy_tombstone` is not subsumed: it admits a directory that *does* hold
`deltas/*.md` files, which is not a husk. Both allowances are needed.

## The line that keeps the fix honest

A directory holding at least one regular file but no `state.json` is damaged, not absent, and
stays refused. Without that line the change would be indistinguishable from deleting the check.
`dated_lifecycle_archive_missing_state_fails_global_enumeration` already pins it and is
untouched by this change.
