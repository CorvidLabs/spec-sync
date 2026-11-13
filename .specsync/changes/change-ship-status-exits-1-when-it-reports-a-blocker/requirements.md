---
change: change-ship-status-exits-1-when-it-reports-a-blocker
artifact: requirements
---

# Requirements

ADDED REQ-cmd-change-039 (`deltas/cmd_change.md`): `change ship-status` exits 1 whenever its
report names a blocker, after printing the whole report, for one change and for the roster; JSON
stays one document; a report with no blocker exits 0.

Wording: claude-code-t370 checked it against Leif's 2026-11-13T14:12Z pick "Exit 1 on a blocker
(Recommended)" (option text 'So scripts and CI can act on it.') and reported it ALIGNED; the
number was allocated by claude-code-t370.

Evidence: `change::ship_status_exits_one_when_it_reports_a_blocker`
(`tests/integration/change.rs`). It fails on main, where every form exits 0 (red run kept), and
passes with the fix.
