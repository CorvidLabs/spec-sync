---
change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
artifact: context
---

# Context

Four follow-ups to #1081's lost-requirements check (REQ-change-169, REQ-cmd-change-042), filed from Codex's third and fourth review rounds on #1108 and kept out of that PR under the two-round cap: #1112 (the files read at the reference come from the working tree's scope too), #1113 (`change ship-status` lists the blocker for archived and retired changes), #1116 (`change audit` skips the check for a change whose approval or delta does not validate) and #1117 (an unreadable requirements directory is reported as a loss).

Definition: defs/1112-1113-1116-1117-definition.md (the specsync agent's draft). Confirmed by t4c under Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after') at about 14:50Z: the recommended choices taken, no new refusal, all four requirements ADDED. Stacked on #1108 (`ed258f58`), like #1122. #1105 (REQ-change-174) later runs the comparison once for the whole project; the #1116 test stays valid under it.
