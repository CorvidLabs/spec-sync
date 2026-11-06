---
hi: 1
families: [MERGE]
---

# Merging two changes to one module

## Intent

Leif confirmed these four sentences on 2026-11-05 for issue #862. They are what he wants from a merge of two independent changes to the same module.

## Criteria

- **MERGE-1**  Two independent changes to the same module, each finalized on its own branch, merge cleanly in either order with no conflict on that module's `.spec.md` (`version:`, change-log row, `files:`) or on `requirements.md`.
- **MERGE-2**  A fixture with two such branches proves it with `git merge-tree` (failing first, then green).
- **MERGE-3**  After a merge that leaves the same requirement ID twice, both `check` and `change audit` refuse and name the duplicate ID.
- **MERGE-4**  Existing strictness stays: no looser gates, timeouts, or weakened assertions.
