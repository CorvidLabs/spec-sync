---
kind: session
id: 2026-11-20-922-933-938-ship-status
aliases: [922-933-938-ship-status]
objective: ship-status names reopen only when reopen runs (#922), change ship finishes an interrupted finalization (#933), and ship-status reads the ledger freeze once per report (#938)
intent_scope: ""
modules: []
stage: build
stage_status: active
iteration: 1
callers: []
native_change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
pr: ""
criteria: [SHIP-4.a, SHIP-5, SHIP-9.a]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-20-1b-red-unit-at-79c8f630
  - 2026-11-20-1b-red-through-the-binary
  - 2026-11-20-1b-verify-lane-at-fd3581a8
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Failing tests first: #922, #933, #938 and the #1038 assertion"
---
# ship-status names reopen only when reopen runs (#922), change ship finishes an interrupted finalization (#933), and ship-status reads the ledger freeze once per report (#938)

## Goal

ship-status names reopen only when reopen runs (#922), change ship finishes an interrupted finalization (#933), and ship-status reads the ledger freeze once per report (#938)

## Where it stands

## Decisions and why

- 0xLeif decided "Leif's relayed approval of the batch 1b definition (#922, #933, #938) (relayed by hc t4c through ss-lead t58)": "Approve as written" (A Claude Code chat answer to his planning session (hc t4c), 2026-11-19 about 14:14Z, item 4 of items 4 to 7, on exactly ss-lead/gates/batch1b-922-933-938-definition.md: in t4c's words, "#922 + #933 + #938 definition (gates/batch1b-922-933-938-definition.md, the no-spec bug fix with #933 option 1, the text-status sentence left out)". Relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads as 0xLeif

## What was verified

- [Red first at 79c8f630, cargo test --bin specsync through one-lane: 7 of the new tests fail on the code before the fix (#922: an archived change told to reopen after unrelated drift and with an unreadable verification.json, a stale accepted or archived change's ship_next without reopen; #933: an interrupted finalization not ready, and change ship refusing it once the task is ticked; #938: two freeze evaluations per report). The guard (an accepted change finalize would not resume is not ready) and the #1038 assertion pass. The neutral git config as a prefix only (#1092).](evidence/2026-11-20-1b-red-unit-at-79c8f630.md)
- [Through the binary, on the code before the fix (the fix set aside, the tests of db652292): #922 told to reopen after unrelated drift and with an unreadable archived verification.json; #933 a finalization stopped before the archive move (its destination pre-made) not ready. The #938 output guard passes before and after. cargo test --test integration through one-lane.](evidence/2026-11-20-1b-red-through-the-binary.md)
- [At fd3581a8 (checked and committed), with this tree's release build first on PATH: fledge lanes run verify 10/10 in 8m27s through one-lane (unit 3108, 2 ignored; integration 648, the 4 binary tests and a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it, which the first lane at a596265d caught and 48e448ce fixed; spec check 62/62, 167/167 files). change audit --strict fails only on tests/integration.rs and tests/integration/ship_status_reopen_and_resume.rs, which the definition does not declare.](evidence/2026-11-20-1b-verify-lane-at-fd3581a8.md)

## Open threads

## Timeline

- 2026-11-20T12:25:10Z · start · intent → intent · ship-status names reopen only when reopen runs (#922), change ship finishes an interrupted finalization (#933), and ship-status reads the ledger freeze once per report (#938) ^t-45870b
  - revision: a28e30420c12627a384a685cd5d808fc3585b70a
  - by: claude-code (hc t59)
- 2026-11-20T12:28:37Z · gate-opened · intent → intent · Leif's relayed approval of the batch 1b definition (#922, #933, #938) (relayed by hc t4c through ss-lead t58) ^t-630de0
  - native_change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
  - revision: a28e30420c12627a384a685cd5d808fc3585b70a
  - holds: intent
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-20T12:28:37Z · gate-closed · intent → intent · Leif's relayed approval of the batch 1b definition (#922, #933, #938) (relayed by hc t4c through ss-lead t58) ^t-bc55c8
  - native_change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
  - revision: a28e30420c12627a384a685cd5d808fc3585b70a
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve as written
    - where: A Claude Code chat answer to his planning session (hc t4c), 2026-11-19 about 14:14Z, item 4 of items 4 to 7, on exactly ss-lead/gates/batch1b-922-933-938-definition.md: in t4c's words, "#922 + #933 + #938 definition (gates/batch1b-922-933-938-definition.md, the no-spec bug fix with #933 option 1, the text-status sentence left out)". Relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-20T12:28:48Z · advance · intent → design · The definition is approved as written and recorded; its design and plan are the approved definition ^t-2887e5
  - native_change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
  - revision: f1b901397ea408ae402681bc8982bb3c0470533a
  - by: claude-code (hc t59)
- 2026-11-20T12:28:48Z · advance · design → plan · The definition is approved as written and recorded; its design and plan are the approved definition ^t-54bf35
  - native_change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
  - revision: f1b901397ea408ae402681bc8982bb3c0470533a
  - by: claude-code (hc t59)
- 2026-11-20T12:28:48Z · advance · plan → build · The definition is approved as written and recorded; its design and plan are the approved definition ^t-95dd80
  - native_change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
  - gate: native:approved
  - revision: f1b901397ea408ae402681bc8982bb3c0470533a
  - by: claude-code (hc t59)
