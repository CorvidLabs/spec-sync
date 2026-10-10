---
change: make-it-obvious-on-a-pull-request-when-a-change-is-still-open-and-when-it-is-archived
artifact: context
---

# Context

A pull request with a verified open change and a pull request whose change is archived both pass the lifecycle gate. The difference shows up only if you look in `.specsync/changes/` or `.specsync/archive/`.

`change ship` finalizes and then tells you to commit, push, wait, and merge. The archive commit is a second step. The next-action line mentions merging even when a change is still open.

The Archive check is the sentence on the pull request. When the change is ready, `change ship` archives it and commits that archive. When it is not ready, status and ship name one reason and one command.
