---
change: when-i-want-a-pull-request-one-command-archives-the-change-and-prepares-it
artifact: design
---

# Design

`change pr` calls the ship path without `--push` when the change is not archived, so the archive commit happens once. It then pushes the current branch, setting `origin` as upstream when there is none. `gh pr view` reads the pull request for that branch. `gh pr create` runs only when that view fails. The printed next step is to wait for green checks and then merge. Nothing in the command calls `gh pr merge`.
