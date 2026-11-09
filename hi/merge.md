---
hi: 1
families: [MERGE]
---

# Merging two changes to one module

## Intent

Leif confirmed these four sentences on 2026-11-05 for issue #862. They are what he wants from a merge of two independent changes to the same module.

MERGE-5 and MERGE-5.a to MERGE-5.d were drafted by the agent from issues #872, #875, #876 and #870 (merge-driver follow-ups from the #865 review), where the driver reported a clean merge whose result was wrong. Leif approved them as drafted in the Herdr orc tab on 2026-11-06 ("Capture MERGE-5 + cases"), relayed by the orc (t18). Recorded by the ss-testgaps agent.

## Criteria

- **MERGE-1**  Two independent changes to the same module, each finalized on its own branch, merge cleanly in either order with no conflict on that module's `.spec.md` (`version:`, change-log row, `files:`) or on `requirements.md`.
- **MERGE-2**  A fixture with two such branches proves it with `git merge-tree` (failing first, then green).
- **MERGE-3**  After a merge that leaves the same requirement ID twice, both `check` and `change audit` refuse and name the duplicate ID.
- **MERGE-4**  Existing strictness stays: no looser gates, timeouts, or weakened assertions.
- **MERGE-5**  When the merge driver calls a merge clean, the result keeps everything both sides wrote and means what they meant; when it cannot, it leaves a conflict for me.
  - **MERGE-5.a**  A value quoted in a contract's frontmatter list is still a string after a merge.
  - **MERGE-5.b**  Whatever follows the change log, prose or another table, stays where it was after a merge.
  - **MERGE-5.c**  When both sides bump the version and annotate it differently, I get the same result whichever way I merge, and neither note is lost.
  - **MERGE-5.d**  A merge never numbers a new change-log row below one already there.
