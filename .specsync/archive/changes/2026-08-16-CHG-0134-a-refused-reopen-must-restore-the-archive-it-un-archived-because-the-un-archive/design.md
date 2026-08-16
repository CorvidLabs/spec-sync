---
change: CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive
artifact: design
---

# Design

Validate-then-move is the obvious fix and it is NOT available here. The
preconditions read the workspace through `find_change_dir`/`change_dir`, and
`authenticate_accepted_evidence` is state-sensitive: it runs
`validate_archived_accepted_snapshot` only while the record is Archived.
Validating before the move would silently change what those checks mean.

So this uses the move-then-restore pattern the file already had. `archive_change`
does exactly this for #540, emitting "archived evidence failed post-move
validation; source restored". The correct shape was sitting in the same file,
unused by its sibling.

`reopen_change` now records the dated archive path it un-archived from, runs the
whole reopen in a new private `reopen_unarchived_change`, and on ANY error
renames the package back to that exact path before returning. The refusal text is
preserved and suffixed with "; archive restored" — or, if the restore itself
fails, a distinct message naming the path to move back by hand, because a user
who has just lost an archive needs the path more than they need a tidy error.

Nothing partial can be left behind: the only write in the reopen body is the
final `write_prepared_files`, which is journaled and rolls back its own targets,
so the restored package is byte-identical to what finalize wrote.

The Accepted-state path is untouched — `unarchived_from` is `None` there, so the
outcome returns verbatim with no suffix.
