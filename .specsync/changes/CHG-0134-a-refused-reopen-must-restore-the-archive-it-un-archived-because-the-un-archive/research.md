---
change: CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive
artifact: research
---

# Research

The fix pattern was found by looking for how the file handles OTHER failable
moves, rather than by designing one. `archive_change`'s "source restored" path
for #540 is the same problem solved correctly one function away.

That is worth recording as a search strategy: when a codebase has the same
hazard in two places and one is handled, the handled one is a better source of
design than first principles — it already agrees with the surrounding
invariants.
