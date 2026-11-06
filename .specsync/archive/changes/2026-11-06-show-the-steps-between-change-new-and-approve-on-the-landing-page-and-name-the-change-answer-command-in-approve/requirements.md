---
change: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve
artifact: requirements
---

# Requirements

- REQ-change-133 (added, `deltas/change.md`): approve's refusals for a missing acceptance criterion or an unfinished interview name the `specsync change answer` command that supplies the answer; the guard refuses exactly what it refused before.

Human intent: DOCS-3 (the landing page shows the steps between `change new` and approve) and DOCS-3.a (if one is skipped anyway, approval's refusal names the command that finishes it), with DOCS-1 and DOCS-1.a (the examples run as written, and CI fails when one stops working). Leif approved them on 2026-11-05 at 20:22 MST, relayed by orc t18.
