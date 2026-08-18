---
change: CHG-0148-a-reopened-change-must-be-closeable-again
artifact: tasks
---

# Tasks

- [x] Establish why the refusal appears at finalize and not at reopen — it comes
      from a walk over committed history, not from the command doing the move.
- [x] Confirm that re-running review cannot clear it, since the transition is
      already committed.
- [x] Admit the reopen direction explicitly rather than loosening the check.
- [x] Confirm a move to a third location is still refused.
- [x] Gate drill 049 self-flips: pending=2 -> 12/0.
- [x] Pin drill 013 inverted by hand (below 044, does not self-flip), and its
      assertion rewritten: a REPAIRED finalize archives the change and clears the
      workspace, so the stranded-state check it inherited no longer applies.
- [x] Confirm both drills discriminate against a binary built from a separate
      checkout of the unfixed tree.
- [x] Whole board: 50/6, the six remaining reds unchanged.
