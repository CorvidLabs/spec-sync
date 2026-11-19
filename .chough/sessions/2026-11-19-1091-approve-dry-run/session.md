---
kind: session
id: 2026-11-19-1091-approve-dry-run
aliases: [1091-approve-dry-run]
objective: specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091)
intent_scope: CHANGE
modules: [change, cmd_change, cli_args]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
pr: ""
criteria: [CHANGE-15, CHANGE-15.a, CHANGE-15.b, CHANGE-15.c]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Continue build
---
# specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091)

## Goal

specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091)

## Where it stands

- Captured CHANGE-15, CHANGE-15.a, CHANGE-15.b and CHANGE-15.c in hi/change.md with their provenance
- Created the draft change with the approved title, specs, paths and interview answers; wrote the three deltas (REQ-change-163 with Leif's exception, REQ-cmd-change-040, REQ-cli-args-027), word for word against the approved text, and every selected artifact

## Decisions and why

- 0xLeif decided "Intent approval of #1091 (CHANGE-15 and its three sub-criteria)": "OK, with (a) fixed" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition; relayed by hc t4c through ss-lead (hc t58). CHANGE-15.a is the wording t4c put to him as that fix, which he chose with these words; he did not write it); recorded by claude-code-t59; a relayed claim, not authenticated
- decided: REQ-change-163's first-problem sentence carries the exception 'except when an interrupted lifecycle write is waiting to be recovered' because that is the exception Leif approved with 'OK, with (a) fixed' and t4c confirmed the sentence (relayed through ss-lead hc t58, 2026-11-19)
- 0xLeif decided "Scope approval of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing": "Approve" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition: the requirements, the criteria and the four choices (which checks are skipped, no lock, the interrupted write, the actor as its own check). REQ-change-163's first-problem sentence carries the exception he approved, confirmed by hc t4c. Relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-19T05:07:57Z · start · intent → intent · specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091) ^t-07b2c9
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t59)
- 2026-11-19T05:08:26Z · gate-opened · intent → intent · Intent approval of #1091 (CHANGE-15 and its three sub-criteria) ^t-b80379
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: intent
  - drives: none
  - by: claude-code (hc t59)
- 2026-11-19T05:08:26Z · gate-closed · intent → intent · Intent approval of #1091 (CHANGE-15 and its three sub-criteria) ^t-37a8d7
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: OK, with (a) fixed
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition; relayed by hc t4c through ss-lead (hc t58). CHANGE-15.a is the wording t4c put to him as that fix, which he chose with these words; he did not write it
    - recorded_by: claude-code-t59
  - by: claude-code (hc t59)
- 2026-11-19T05:08:31Z · advance · intent → design ·  ^t-124ef7
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t59)
- 2026-11-19T05:08:31Z · skip · design → plan · CLI behaviour only: no visuals or interactions ^t-153302
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t59)
- 2026-11-19T05:26:59Z · gate-opened · plan → plan · Scope approval of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing ^t-a6759a
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - revision: a5ec540b461ce962a7d05bb521a807dd10e6dcd8
  - holds: plan
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-19T05:26:59Z · gate-closed · plan → plan · Scope approval of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing ^t-cf7720
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - revision: a5ec540b461ce962a7d05bb521a807dd10e6dcd8
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition: the requirements, the criteria and the four choices (which checks are skipped, no lock, the interrupted write, the actor as its own check). REQ-change-163's first-problem sentence carries the exception he approved, confirmed by hc t4c. Relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-19T05:27:30Z · advance · plan → build · native:approved ^t-afa7b9
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - gate: native:approved
  - revision: a5ec540b461ce962a7d05bb521a807dd10e6dcd8
  - by: claude-code (hc t59)
