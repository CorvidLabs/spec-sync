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

Re-scope after the first review: Leif approved and reviewed this change at 2e65c8a8 and 45772452. `change ship`
then refused it with "acceptance input `src/agents.rs` is production source without deterministic canonical
ownership", because the approved affected paths named all of `src/`. Acceptance gives every production file in
scope an owner among the affected specs, and 127 of the 164 files under `src/` belong to other modules. Leif sent
it back (recorded by orc t18; option A). The affected paths now name exactly the files the change touches. `cli`
joins the affected specs because it owns `src/main.rs`, which gains the doctor dispatch arm, and its Purpose now
says that it routes `specsync doctor`. The code, tests and acceptance criteria are unchanged. The review the
send-back cancelled stays in the append-only history.

