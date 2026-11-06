---
change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
artifact: context
---

# Context

#833: archived changes left in state `accepted` have no supported repair, and one of them blocks every later
`change finalize`.

- **Corvidinho:** 46 such packages.
- **CorvidLabs/site:** 41 changes merged and never archived (the 38 to close plus 3 drafts). CHG-0026 blocks
  every finalize. Its documented recovery, `reopen`, overflows a bounded Git read (#976, fixed separately).
  The 15 v1 records cannot finish through `accept`. Site #616 sits in the partial-finalize state.

Leif's direction (relayed by orc t18, 2026-11-06) supersedes a standalone `close-late` command and a new
`closed_late` state. Build `specsync doctor`. It lists every problem with its fix, `--fix` applies the safe
ones, `--dry-run` shows them, and decisions need `--actor`/`--reason`. The late close is one doctor fix,
stored where SpecSync 6.x never looks. It ships in 6.3.0, and the site and Corvidinho CI pin 6.0.0.

Intent, captured verbatim with Leif's source:

- SHIP-12 and SHIP-12.a to SHIP-12.f ("Capture all + approve design");
- SHIP-12.g, RECOVER-3 and RECOVER-3.a to RECOVER-3.d ("Capture + recommended design").

Decisions D2 to D5 are recorded in the constraints answer. Requirement IDs were claimed by the orc:
REQ-change-143 (reworded), REQ-cmd-change-035 and REQ-cli-args-025.

The never-approved draft `close-a-change-that-merged-but-was-never-archived-…` is removed in this commit. Its
approval was refused with "semantic delta modules must exactly match affected specs".
