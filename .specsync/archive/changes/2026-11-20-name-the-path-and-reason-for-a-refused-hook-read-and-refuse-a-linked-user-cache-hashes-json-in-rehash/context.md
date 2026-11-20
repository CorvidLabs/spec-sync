---
change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
artifact: context
---

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
