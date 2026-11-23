---
change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
artifact: testing
---

# Testing

| Case | Required behavior | Test |
|------|-------------------|------|
| Installed `CLAUDE.md` past 8 MiB | `uninstall --claude` exits 1 naming `CLAUDE.md` and the bound; file unchanged; `status` prints the reason; `install --claude` exits 1 | `tests/integration/project_reads.rs`: `hooks_name_an_oversized_hook_file_instead_of_calling_it_not_installed` |
| `CLAUDE.md` not valid UTF-8 | `uninstall --claude` exits 1 naming the path and UTF-8 | `hooks_name_a_hook_file_that_is_not_utf8` |
| Hooks folder reached through a link | `uninstall --precommit` exits 1 naming the path | `tests/integration/project_writes.rs`: `hooks_uninstall_precommit_refuses_a_linked_hooks_folder` |
| Not a Git repository | `uninstall --precommit` exits 0, nothing to remove | `hooks_uninstall_precommit_outside_git_has_nothing_to_remove` |
| User-cache `hashes.json` a link or a directory | `rehash` exits 1 with `check`'s reason; link and target unchanged | `rehash_refuses_a_linked_user_cache_hashes_file` |
