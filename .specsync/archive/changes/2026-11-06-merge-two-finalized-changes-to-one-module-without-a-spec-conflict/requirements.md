---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: requirements
---

# Requirements

The confirmed sentences, unchanged:

- MERGE-1: Two independent changes to the same module, each finalized on its own branch, merge cleanly in either order with no conflict on that module's `.spec.md` (`version:`, change-log row, `files:`) or on `requirements.md`.
- MERGE-2: A fixture with two such branches proves it with `git merge-tree` (failing first, then green).
- MERGE-3: After a merge that leaves the same requirement ID twice, both `check` and `change audit` refuse and name the duplicate ID.
- MERGE-4: Existing strictness stays: no looser gates, timeouts, or weakened assertions.

Module requirements that state the same behavior: REQ-merge-004, REQ-merge-005, REQ-cmd-merge-003, REQ-validator-045, REQ-change-125.
