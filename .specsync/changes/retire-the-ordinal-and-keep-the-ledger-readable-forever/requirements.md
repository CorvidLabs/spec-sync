---
change: retire-the-ordinal-and-keep-the-ledger-readable-forever
artifact: requirements
---

# Requirements

## REQ-change-086 (new)

A change identity SHALL be minted from its description alone, and identity uniqueness SHALL be
enforced directly rather than as a side effect of allocating a number.

## Deliberately unchanged

Every historical identity. Archives keep their `CHG-NNNN-slug` IDs and directory names
permanently; nothing is renamed or renumbered, and no digest moves.

The sequence ledger as a readable artifact. It stops growing and stays reconcilable forever, for
the 120 archives that signed it and the 11 whose acknowledged collisions live only there.

See `deltas/change.md` for the canonical delta.
