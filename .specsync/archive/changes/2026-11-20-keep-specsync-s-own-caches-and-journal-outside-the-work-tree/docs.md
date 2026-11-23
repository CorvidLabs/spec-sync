---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: docs
---

# Docs

- **`CHANGELOG.md`, `### Security`:** one entry. It says:
  - where the cache, the journal and the remote cache now live;
  - that an in-tree journal stops lifecycle commands until it is deleted;
  - the enclosing-work-tree rule;
  - the hooks-folder rule;
  - the documented limits (F5, F8, F9, F10).
- **Specs:**
  - the requirements below, through the deltas;
  - the Public API rows for `local_state`;
  - invariants and error cases in `util`, `hash_cache`, `change`, `cmd_check`, `rehash`, `cmd_resolve`, `hooks` and `validator`;
  - `testing.md` and `tasks.md`;
  - a lesson in `util.3md`.
- **The private advisory additions** (outside this repository): the class fix, the documented limits, and F10.
