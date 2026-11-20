## MODIFIED
### REQUIREMENT REQ-hooks-009

`hooks install`, `uninstall` and `status` SHALL reach the hooks folder Git names (`git rev-parse --git-path hooks`, which honors `core.hooksPath`) one component at a time without following any symbolic link, from the canonical Git directory, or in a linked worktree the canonical common Git directory, when the folder lies in either, from the canonical work-tree top when it lies there, and otherwise from the canonical parent of that folder. A folder outside these trees SHALL be used, and made when missing, only when `git config --show-scope` says `core.hooksPath` comes from the `global` or `system` config; set anywhere else, including a `.git/config` an archive ships, nothing SHALL be read, written or made there, and the command SHALL say why. Each path SHALL be compared after its longest existing ancestor is resolved, so a path spelled through a link (macOS's `/var` for `/private/var`) is placed where it lies. The `git rev-parse` call SHALL run without `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` in its environment. They SHALL read `pre-commit` without following a link or blocking, checked on the handle and bounded (8 MiB); replace it through the open, checked handle; create it only with an exclusive create; make it executable through the handle; and remove it by unlinking its name. A link on the way or at `pre-commit` SHALL be refused with the path named, and nothing SHALL be written or made executable outside the folder.

Acceptance Criteria
- With `core.hooksPath` inside the work tree, a committed hooks folder link and a dangling `pre-commit` link are refused, and nothing is created or made executable at their targets.
- With a repository-local `core.hooksPath` outside the work tree, a missing folder is not made.
- A real hooks folder inside the work tree gets an executable `pre-commit`.
- With a repository-local `core.hooksPath` naming an existing folder outside the work tree, no hook is written there and the command fails.
- An absolute `core.hooksPath` spelled through a link to the work tree is inside it.
- In a linked worktree whose main repository has no `hooks` folder, `install --precommit` makes it and installs the hook.
