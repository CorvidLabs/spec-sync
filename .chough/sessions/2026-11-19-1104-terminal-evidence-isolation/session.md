---
kind: session
id: 2026-11-19-1104-terminal-evidence-isolation
aliases: [1104-terminal-evidence-isolation]
objective: One change whose records fail validation no longer decides another change's terminal evidence (#1104)
intent_scope: CHANGE
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
pr: ""
criteria: [CHANGE-7, CHANGE-8]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-1104-red-s1-s2-at-418bd2ec
  - 2026-11-19-1104-red-dependency-tests-before-the-fix
  - 2026-11-19-1104-verify-lane-at-b3c2cc0b
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
    holds: accept
    drives: review
    opened: 2026-11-19T20:14:25Z
run_consents: []
next: "Lane 10/10 at b3c2cc0b; evidence recorded against the existing CHANGE-7 and CHANGE-8. #1104 is for 6.5.0: rebuild onto the corrected main with #428 before it goes further, and merge after the v6.4.0 tag. Leif's review, relayed by hc t4c through ss-lead, is recorded with chough change ship and his words."
---
# One change whose records fail validation no longer decides another change's terminal evidence (#1104)

## Goal

One change whose records fail validation no longer decides another change's terminal evidence (#1104)

## Where it stands

## Decisions and why

- 0xLeif decided "Approve the scope of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence": "approve 1104 as written" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 15:12Z, on the frozen ss-lead/gates/1104-definition.md, with its three choices as recommended (a bug fix with no new requirement text; an unreadable state.json counts against a successor search only; no warning line in other changes' status); relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence as 0xLeif
- #1104's evidence is recorded against existing criteria, not a new one: CHANGE-7 ('At any moment I can ask what state a change is in and be told the single next thing to do'), since the bug was one change's status reporting another change's broken history, and CHANGE-8 ('I can see every change in flight at once') for change list. Leif approved #1104 as needing no new text, so no criterion is captured; directed by ss-lead (hc t58), 2026-11-19, as #1083 referenced SETUP-2.b

## What was verified

- [Red first, before the fix: with the tests of 418bd2ec on 880237c8, an archive committed before another change's bad commit (tester2's S1) reads corrupt-history with that other change's 'duplicates state at commit' reason; the broken change's own failure test passes, as it must. Fixed in 71ea5bac.](evidence/2026-11-19-1104-red-s1-s2-at-418bd2ec.md)
- [The dependency tests of 06ac04fe, written after the fix, fail against the pre-fix sources: an accepted change with unchanged inputs and an unreadable state elsewhere does not read exact, and a failed candidate is not named in a successor search.](evidence/2026-11-19-1104-red-dependency-tests-before-the-fix.md)
- [At b3c2cc0b (checked and committed), fledge lanes run verify passed 10/10 steps in 11m42s through one-lane: unit 3099 passed (2 ignored), integration 630, spec check 166/166 files. Includes the S1 integration test through the binary (status, change list, ship-status, audit --archives, a third change shipped). The neutral git config was a prefix on the lane command only (#1092).](evidence/2026-11-19-1104-verify-lane-at-b3c2cc0b.md)

## Open threads

## Timeline

- 2026-11-19T17:17:14Z · start · intent → intent · One change whose records fail validation no longer decides another change's terminal evidence (#1104) ^t-d0a1b6
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T17:22:12Z · advance · intent → design ·  ^t-ded7b1
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T17:23:12Z · skip · design → plan · No visuals or interactions: status verdicts only ^t-70af62
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T17:23:12Z · gate-opened · plan → plan · Approve the scope of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence ^t-70b309
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - holds: plan
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-19T17:23:12Z · gate-closed · plan → plan · Approve the scope of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence ^t-67e23d
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - relayed_by:
    - decided_by: 0xLeif
    - words: approve 1104 as written
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 15:12Z, on the frozen ss-lead/gates/1104-definition.md, with its three choices as recommended (a bug fix with no new requirement text; an unreadable state.json counts against a successor search only; no warning line in other changes' status); relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-19T17:23:19Z · advance · plan → build · native:approved ^t-94f997
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - gate: native:approved
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T20:14:07Z · advance · build → test · native:verifying ^t-7fcc37
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - gate: native:verifying
  - revision: b3c2cc0b4c72e55195a205a841bc00afe35d7db4
  - by: claude-code (hc t59)
- 2026-11-19T20:14:15Z · advance · test → accept · native:verifying ^t-68d7c0
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - gate: native:verifying
  - revision: b3c2cc0b4c72e55195a205a841bc00afe35d7db4
  - by: claude-code (hc t59)
- 2026-11-19T20:14:25Z · gate-opened · accept → accept · Implementation review of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence ^t-c7eb62
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: b3c2cc0b4c72e55195a205a841bc00afe35d7db4
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
