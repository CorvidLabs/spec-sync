---
change: CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar
artifact: requirements
---

# Requirements

## REQ-cmd-change-014 (new)

`change ship-status` SHALL name a next action the same binary will accept, and SHALL resolve a
change's verification and review evidence from wherever that change currently lives.

See `deltas/cmd_change.md` for the canonical delta.

## `change` module — Public API only

`find_change_dir` becomes an exported item of the `change` module, so it gains a row in that
module's Public API section. No `change`-module requirement changes: the resolver's behaviour is
unchanged, only its visibility. See `deltas/change.md`.

## Deliberately not changed

The `Blocker:` lines, the `blockers[]` JSON array, and the stage ladder's contents. This change
alters which string the `Next:` line carries and where evidence is read from; it does not change
what counts as a blocker or what the stages are.

The post-merge behaviour of the archived stage ladder is also unchanged and is stated as a known
limit in `research.md`: `verified` depends on commit ancestry, which a squash merge breaks.
