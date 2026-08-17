---
change: CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent
artifact: requirements
---

# Requirements

Two requirements are added as semantic deltas, one per affected module. The delta files are the
source; `specs/` is materialized from them rather than hand-edited.

## `deltas/cmd_change.md` — REQ-cmd-change-011

The command surface. Commands that enumerate active changes must distinguish an empty project
from one whose workspaces could not be read, and must never present a partial roster as complete.

It sits beside REQ-cmd-change-009, which already requires text `show`/`status` to fail closed
before emitting a successful lifecycle projection when a correction ledger is invalid. This is
the same principle applied one level out: not "do not project from bad data" but "do not report
absence you did not measure."

## `deltas/change.md` — REQ-change-068

The domain surface. Enumeration must return what could be read and what could not as separate
facts, so no caller can mistake an unreadable workspace for an absent one.

Stated at the roster level rather than the command level because the four defective callers were
not all commands — the pull-request diff base and sibling-in-flight reporting are internal, and a
requirement written only about `list` and `status` would not have covered them.

## Explicitly retained behaviour

Both requirements carry an acceptance criterion for the case that must **not** change: a project
with genuinely no active changes still prints the empty-project line and exits 0. Without it, a
change that simply started refusing every tree would satisfy everything else.

The plain record list used by digest, ledger and successor computation also keeps failing closed
on any unreadable workspace. That is the historical contract its eleven callers were written
against, and a silently short roster is more dangerous there than a hard error.
