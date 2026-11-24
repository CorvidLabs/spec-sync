---
change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
artifact: context
---

# Context

Issue #1104. `terminal_evidence_summary` (`src/change/consider_accepted_evidence_anchor.rs`) answers
"is this change's evidence still good?" for `change status`, `change show`, `change list` and
ship-status. In each of its three paths it first loads the whole roster with
`list_all_changes_checked`, which validates every change, the history of every archive included. If
any one change fails, that error becomes every change's verdict: `corrupt-history` for each archived
change and `stale` for each open one, with the broken change's reason.

tester2 reproduced it on SpecSync 6.3.0 and on a copy of starling (`tester2/prs/g298/notes.md`, S1
and S2): a merge left one change both open and archived at one commit, and 8 to 13 other archived
changes then read `corrupt-history` with that change's reason, their own evidence intact. `check`,
`review` and `finalize` were never stopped by it.

Leif approved the definition as written on 2026-11-19 at about 15:12Z ("approve 1104 as written"),
relayed by his planning session (hc t4c) through ss-lead (hc t58), with its three choices as
recommended: a bug fix with no new requirement text, an unreadable `state.json` counting against a
successor search only, and no warning line in other changes' status. Frozen copy:
`ss-lead/gates/1104-definition.md`.
