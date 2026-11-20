# Lesson bundle: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Name the path and reason for a refused hook read, and refuse a linked user-cache hashes.json in rehash
- **Kind**: BugFix
- **Specs**: hooks, hash_cache, rehash
- **Paths**: src/hooks.rs, src/hash_cache.rs, src/commands/rehash.rs, tests/integration
- **Acceptance**: With an installed CLAUDE.md grown past 8 MiB, or not valid UTF-8, hooks uninstall --claude exits 1 naming CLAUDE.md and the reason and leaves the file unchanged; hooks install --claude exits 1 naming it; hooks status prints the reason for that target instead of not installed (REQ-hooks-007).
With the Git hooks folder reached through a symbolic link, hooks uninstall --precommit exits 1 naming the path; in a folder that is not a Git repository, or whose hooks folder does not exist, it still reports nothing to remove and exits 0 (REQ-hooks-009).
With the user-cache hashes.json a symbolic link or a directory, rehash exits 1 with the reason check's warning gives, writing nothing: the link is still a link and its target unchanged; with a regular or absent cache, rehash still rebuilds it (REQ-hash-cache-007).

## Evidence

- Verification commit: `4f80d8304f7e50a8c601e18b7f8d8eadc8e57b48`
- Base commit: `7527da3a7aea52436404766df845bbc8fb2f19b6`
- Verified by: `specsync check --spec hash_cache --spec hooks --spec rehash`

## From the change's context.md

# Context

The reviews of #1133 (the 6.4.0 fix at `7527da3a`) found two places where the code falls short of
requirements already approved:

- **Hooks (Codex P2 on `src/hooks.rs:625`, corvid-agent's must-fix).** `is_installed` turns a
  refused or failed bounded read of a hook target into "not installed". With `CLAUDE.md` past
  8 MiB, `hooks uninstall --claude` printed "No hooks to remove." and exited 0, and `hooks status`
  said "not installed". REQ-hooks-007 requires such a read to be handled as that read's other
  errors, naming the path and the reason. `uninstall_hook` for the pre-commit hook did the same
  with a hooks folder reached through a link (REQ-hooks-009).
- **Rehash (Codex P2 on `src/hash_cache.rs:124`).** `HashCache::save` renamed a new cache over a
  user-cache `hashes.json` that is a symbolic link, so `rehash` exited 0 where REQ-hash-cache-007
  says `save` refuses. Nothing was written through the link.

Leif approved this as change 9 ("Yes, change 9 (Recommended)", about 15:43Z on 2026-11-20).
A third finding, hooks in a linked worktree with no shared hooks folder, goes to 6.4.1.

## From the change's testing.md

# Testing

| Case | Required behavior | Test |
|------|-------------------|------|
| Installed `CLAUDE.md` past 8 MiB | `uninstall --claude` exits 1 naming `CLAUDE.md` and the bound; file unchanged; `status` prints the reason; `install --claude` exits 1 | `tests/integration/project_reads.rs`: `hooks_name_an_oversized_hook_file_instead_of_calling_it_not_installed` |
| `CLAUDE.md` not valid UTF-8 | `uninstall --claude` exits 1 naming the path and UTF-8 | `hooks_name_a_hook_file_that_is_not_utf8` |
| Hooks folder reached through a link | `uninstall --precommit` exits 1 naming the path | `tests/integration/project_writes.rs`: `hooks_uninstall_precommit_refuses_a_linked_hooks_folder` |
| Not a Git repository | `uninstall --precommit` exits 0, nothing to remove | `hooks_uninstall_precommit_outside_git_has_nothing_to_remove` |
| User-cache `hashes.json` a link or a directory | `rehash` exits 1 with `check`'s reason; link and target unchanged | `rehash_refuses_a_linked_user_cache_hashes_file` |

## Where these lessons go

- `specs/hooks/hooks.3md`
- `specs/hash_cache/hash_cache.3md`
- `specs/rehash/rehash.3md`
