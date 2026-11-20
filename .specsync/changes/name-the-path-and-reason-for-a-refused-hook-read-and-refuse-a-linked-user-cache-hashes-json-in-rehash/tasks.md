---
change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
artifact: tasks
---

# Tasks

- [ ] Red: integration tests for an oversized and a non-UTF-8 `CLAUDE.md` (`hooks uninstall --claude`, `hooks install --claude`, `hooks status`), for a hooks folder reached through a link (`hooks uninstall --precommit`), for `hooks uninstall --precommit` outside a Git repository, and for `rehash` with a linked or directory user-cache `hashes.json`.
- [ ] `hooks`: `installed` returns the read error with the path and the reason; `install_hook`, `uninstall_hook` and `cmd_status` use it; `git_hooks_dir` tells a missing repository or hooks folder from a refusal; `cmd_uninstall` exits 1 on any error but the Claude Code hook's manual-removal note.
- [ ] `hash_cache`: `save` refuses, writing nothing, when `hashes.json` in the local state folder is a symbolic link or not a regular file, in `refusal`'s words.
- [ ] Green on macOS (verify lane) and Linux, through one-lane; `change audit` passes.
