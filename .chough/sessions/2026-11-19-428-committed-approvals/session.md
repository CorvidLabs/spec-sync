---
kind: session
id: 2026-11-19-428-committed-approvals
aliases: [428-committed-approvals]
objective: finalize, ship and archive refuse an approvals ledger that Git history does not hold (#428)
intent_scope: CHANGE
modules: [change, cmd_change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
pr: ""
criteria: [CHANGE-16, CHANGE-16.a, CHANGE-16.b, CHANGE-16.c]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-428-red-on-main-at-ba481a7e
  - 2026-11-19-428-red-integration-on-main
  - 2026-11-19-428-verify-lane-at-d8ad737f
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
    holds: accept
    drives: review
    opened: 2026-11-19T17:15:06Z
run_consents: []
next: "REQ-change-166's fourth criterion waits on Leif (reports/428-delete-recreate-wording.md: a deleted-and-recreated workspace is refused by the older workflow-version guard first). Then Leif's review, after the v6.4.0 tag; no PR until ss-lead says"
---
# finalize, ship and archive refuse an approvals ledger that Git history does not hold (#428)

## Goal

finalize, ship and archive refuse an approvals ledger that Git history does not hold (#428)

## Where it stands

- Built red first from main (3bebe8b0, then 2cc0897f merged in): failing tests ba481a7e, fix 54427418, existing tests 4e62db84, rule tests 2cc29610, docs; lane 10/10 at d8ad737f; change check materialized REQ-change-166 and REQ-cmd-change-041 equal to the frozen definition

## Decisions and why

- 0xLeif decided "Intent approval of #428 (CHANGE-16 and its three sub-criteria)": "Approve as written" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:17Z, on the frozen ss-lead/gates/428-definition.md, its intent sentences shown to him as the builder's draft; relayed by hc t4c through ss-lead (hc t58). CHANGE-16 to CHANGE-16.c are the wording he chose with these words, not words he wrote); recorded by claude-code-t59; a relayed claim, not authenticated
- 0xLeif decided "Approve the scope of finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold": "Approve as written" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:17Z, on the frozen ss-lead/gates/428-definition.md (the requirements REQ-change-166 and REQ-cmd-change-041, the six change criteria and the choices in its section 6), its intent sentences shown to him as the builder's draft; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold as 0xLeif

## What was verified

- [Red first on main 3bebe8b0: at ba481a7e (the tests alone) seven unit tests fail: finalize archives a ledger edited since HEAD, one whose working copy only adds records, one not committed at HEAD, a committed rewrite, a rewrite merged in from a side branch and a recreated change; an accepted change archives with an extra or changed approval. Five pass, as they must before and after. This is the defect #428 reports; fixed in 54427418.](evidence/2026-11-19-428-red-on-main-at-ba481a7e.md)
- [Red first on main: finalize_ship_and_ship_status_refuse_a_ledger_the_working_tree_changed fails: ship-status calls a change ready to finalize after its approvals.json was edited in the working tree.](evidence/2026-11-19-428-red-integration-on-main.md)
- [At d8ad737f (main 2cc0897f merged in; the fix, the rule tests and the existing-test commits), fledge lanes run verify passed 10/10 steps in 8m29s, run alone: unit 3110 passed (2 ignored), integration 630 passed (ship-status exit 1 on the ledger blocker included), strict specs 62/62. The neutral git config was a prefix on the lane command only (#1092)](evidence/2026-11-19-428-verify-lane-at-d8ad737f.md)

## Open threads

## Timeline

- 2026-11-19T14:23:31Z · start · intent → intent · finalize, ship and archive refuse an approvals ledger that Git history does not hold (#428) ^t-022b1f
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t59)
- 2026-11-19T14:23:56Z · gate-opened · intent → intent · Intent approval of #428 (CHANGE-16 and its three sub-criteria) ^t-a023a2
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: intent
  - drives: none
  - by: claude-code (hc t59)
- 2026-11-19T14:23:56Z · gate-closed · intent → intent · Intent approval of #428 (CHANGE-16 and its three sub-criteria) ^t-2a0853
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve as written
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:17Z, on the frozen ss-lead/gates/428-definition.md, its intent sentences shown to him as the builder's draft; relayed by hc t4c through ss-lead (hc t58). CHANGE-16 to CHANGE-16.c are the wording he chose with these words, not words he wrote
    - recorded_by: claude-code-t59
  - by: claude-code (hc t59)
- 2026-11-19T14:25:07Z · advance · intent → design ·  ^t-6ec41c
  - revision: cf28b4c5052afb72ce01454a353d2e99c83b3222
  - by: claude-code (hc t59)
- 2026-11-19T14:29:21Z · skip · design → plan · No visuals or interactions: lifecycle refusals and CLI output only ^t-474e86
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: cf28b4c5052afb72ce01454a353d2e99c83b3222
  - by: claude-code (hc t59)
- 2026-11-19T14:29:23Z · gate-opened · plan → plan · Approve the scope of finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold ^t-8419c6
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: cf28b4c5052afb72ce01454a353d2e99c83b3222
  - holds: plan
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-19T14:29:23Z · gate-closed · plan → plan · Approve the scope of finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold ^t-0fc03d
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: cf28b4c5052afb72ce01454a353d2e99c83b3222
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve as written
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:17Z, on the frozen ss-lead/gates/428-definition.md (the requirements REQ-change-166 and REQ-cmd-change-041, the six change criteria and the choices in its section 6), its intent sentences shown to him as the builder's draft; relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-19T14:29:33Z · advance · plan → build · native:approved ^t-9f95c0
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:approved
  - revision: cf28b4c5052afb72ce01454a353d2e99c83b3222
  - by: claude-code (hc t59)
- 2026-11-19T17:15:05Z · advance · build → test · native:verifying ^t-697047
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: b70c28033539420c937ddb44d64a12ddcb35b63d
  - by: claude-code (hc t59)
- 2026-11-19T17:15:06Z · advance · test → accept · native:verifying ^t-c7ee33
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: b70c28033539420c937ddb44d64a12ddcb35b63d
  - by: claude-code (hc t59)
- 2026-11-19T17:15:06Z · gate-opened · accept → accept · Implementation review of finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold ^t-5446d7
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: b70c28033539420c937ddb44d64a12ddcb35b63d
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
- 2026-11-19T17:34:23Z · back · accept → build · ss-lead's review at 35090439: the Git-read refusals name the way out ^t-ba1bfe
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: cb0b4fc3273a3ca9a62110362773cf386a9f6601
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 3 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
