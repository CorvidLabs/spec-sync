## MODIFIED

### REQUIREMENT REQ-hooks-009

`hooks install`, `uninstall` and `status` SHALL reach the hooks folder Git names (`git rev-parse --git-path hooks`, which honors `core.hooksPath`) one component at a time without following any symbolic link, from the canonical Git directory when the folder lies in it, from the canonical work-tree top when it lies there, and otherwise from the canonical parent of that folder. Unlike the previous text, which made a missing folder outside both trees for any config, a folder outside both SHALL be made, when missing, only when `git config --show-scope` says `core.hooksPath` comes from the `global` or `system` config; set anywhere else, including a `.git/config` an archive ships, a missing folder SHALL be an error and nothing SHALL be made there. They SHALL read `pre-commit` without following a link or blocking, checked on the handle and bounded (8 MiB); replace it through the open, checked handle; create it only with an exclusive create; make it executable through the handle; and remove it by unlinking its name. A link on the way or at `pre-commit` SHALL be refused with the path named, and nothing SHALL be written or made executable outside the folder.

Acceptance Criteria
- With `core.hooksPath` inside the work tree, a committed hooks folder link and a dangling `pre-commit` link are refused, and nothing is created or made executable at their targets.
- With a repository-local `core.hooksPath` outside the work tree, a missing folder is not made.
- A real hooks folder inside the work tree gets an executable `pre-commit`.
