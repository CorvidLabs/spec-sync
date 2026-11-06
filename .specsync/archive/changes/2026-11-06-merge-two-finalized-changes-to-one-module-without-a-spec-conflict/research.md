---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: research
---

# Research

Replayed the conflict with git 2.43. When both branches write the same `version: 6`, that line merges by itself. The conflict is the new `files:` item, the new change-log row, and the new requirement block. When the version numbers differ, git groups `version:` into the same hunk as `files:`.

`git merge-tree --write-tree` runs a configured merge driver. With `merge=specsync` set and no driver command, a conflicting file still conflicts and a non-overlapping edit still merges. A driver command that exits non-zero turns even a non-overlapping edit into a conflict, so the driver must merge the whole file, not only the hunks.

GitHub's pull-request merge does not run a custom merge driver. Local `git merge` and `git merge-tree` do, once `specsync merge --install` has recorded the command.
