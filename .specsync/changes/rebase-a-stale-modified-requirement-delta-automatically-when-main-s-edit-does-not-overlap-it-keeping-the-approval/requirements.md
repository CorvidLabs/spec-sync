---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: requirements
---

# Requirements

The delta adds REQ-change-161 (`deltas/change.md`) and REQ-cmd-change-038 (`deltas/cmd_change.md`), and modifies REQ-change-122 so the #843 refusal names the rebase as the one exception; its remedy and every other criterion stay. Intent: CHANGE-6.a (an approval is bound to what it approved, so the rebase is a recorded, re-checkable merge, never a silent edit) and CHANGE-6.c (getting on with the implementation costs no fresh approval). No gate is loosened: overlapping edits, sections, REMOVED items, workflow v1, CI and an unrecoverable base keep today's refusal.
