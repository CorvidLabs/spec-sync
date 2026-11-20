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
  - 2026-11-19-428-verify-lane-at-cb0b4fc3
  - 2026-11-19-428-red-ways-out-at-4ebc7bea
  - 2026-11-19-428-verify-lane-at-e869bf11
  - 2026-11-20-428-rework-lane-at-f444f569
  - 2026-11-20-428-review-round-1-at-a0b997db
  - 2026-11-20-428-migrate-regression-at-c7d16a2b
  - 2026-11-20-428-ship-order-at-5ef1af6a
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Red first: option 3's message, then the finalize-level superseded_approval relay test"
---
# finalize, ship and archive refuse an approvals ledger that Git history does not hold (#428)

## Goal

finalize, ship and archive refuse an approvals ledger that Git history does not hold (#428)

## Where it stands

- Built red first from main (3bebe8b0, then 2cc0897f merged in): failing tests ba481a7e, fix 54427418, existing tests 4e62db84, rule tests 2cc29610, docs; lane 10/10 at d8ad737f; change check materialized REQ-change-166 and REQ-cmd-change-041 equal to the frozen definition

## Decisions and why

- 0xLeif decided "Intent approval of #428 (CHANGE-16 and its three sub-criteria)": "Approve as written" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:17Z, on the frozen ss-lead/gates/428-definition.md, its intent sentences shown to him as the builder's draft; relayed by hc t4c through ss-lead (hc t58). CHANGE-16 to CHANGE-16.c are the wording he chose with these words, not words he wrote); recorded by claude-code-t59; a relayed claim, not authenticated
- 0xLeif decided "Approve the scope of finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold": "Approve as written" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:17Z, on the frozen ss-lead/gates/428-definition.md (the requirements REQ-change-166 and REQ-cmd-change-041, the six change criteria and the choices in its section 6), its intent sentences shown to him as the builder's draft; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold as 0xLeif
- 0xLeif decided "Leif's relayed approval of #428's option 3, its two reworded sentences and its three declared test files (relayed by hc t4c through ss-lead t58)": ""All as recommended" for option 3; Leif's six rules, rule 6 (a) and (c), for the two sentences; "yes to all 3" for the three test files" (Option 3: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 19:43Z, on the frozen ss-lead/gates/428-criterion4.md (the older check's new message, REQ-change-166's new fourth criterion, the change's new second criterion and the added path src/change/validate_loaded_change.rs), with the builder running the send-back; relayed through ss-lead (hc t58). The two sentences (REQ-change-166's still-finalizes criterion and the change's third criterion): a standing OK in Leif's six rules, relayed by hc t4c on 2026-11-19 at about 14:35Z through ss-lead (hc t58): (a) a wording fix that only makes an approved sentence match tested code, and (c) the send-back it needs; applied by ss-lead to these exact texts on 2026-11-19; these are not his words about these texts. The three test files: his one-time yes, "yes to all 3", in a Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 01:00Z, answering item 1, '#1104 and #428 edit four test files their scope doesn't list. May the builder add those paths in the send-backs already planned? No wording or behaviour changes.'; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold as 0xLeif
- History rewritten onto the corrected main. The commit ids this session's notes and the change's verification records name map to: abbbc279 -> 5e2f0f07, 232d0fd8 -> b492c69d, 54427418 -> 0012db50, 4e62db84 -> 36c2e472, 2cc29610 -> 7574561f, aa2fa4b5 -> ad07a67d, d8ad737f -> 91b8496a, 392ead1d -> e9732782, d619032f -> a16e36e9, b70c2803 -> 6aa692dc, 35090439 -> 6b4fe21f, 67657421 -> ffdb5111, cb0b4fc3 -> ec6c0aa6, 38d52971 -> 0e1f712b, 45b8f9dc -> 6bfa0d76, 999f55a3 -> e8e9ef5b, 4ebc7bea -> 2e923bd5, 88cb2857 -> 0159d72e, f723893d -> f4787775, e869bf11 -> bf2c60da, ff59ae9e -> af209abe; and main 2cc0897f -> 7d8370f4. cf28b4c5, df54b34a, ba481a7e and 21c11a77 kept their ids.

## What was verified

- [Red first on main 3bebe8b0: at ba481a7e (the tests alone) seven unit tests fail: finalize archives a ledger edited since HEAD, one whose working copy only adds records, one not committed at HEAD, a committed rewrite, a rewrite merged in from a side branch and a recreated change; an accepted change archives with an extra or changed approval. Five pass, as they must before and after. This is the defect #428 reports; fixed in 54427418.](evidence/2026-11-19-428-red-on-main-at-ba481a7e.md)
- [Red first on main: finalize_ship_and_ship_status_refuse_a_ledger_the_working_tree_changed fails: ship-status calls a change ready to finalize after its approvals.json was edited in the working tree.](evidence/2026-11-19-428-red-integration-on-main.md)
- [At d8ad737f (main 2cc0897f merged in; the fix, the rule tests and the existing-test commits), fledge lanes run verify passed 10/10 steps in 8m29s, run alone: unit 3110 passed (2 ignored), integration 630 passed (ship-status exit 1 on the ledger blocker included), strict specs 62/62. The neutral git config was a prefix on the lane command only (#1092)](evidence/2026-11-19-428-verify-lane-at-d8ad737f.md)
- [At cb0b4fc3 (the Git-read refusals name the way out, red first in the commit before), fledge lanes run verify passed 10/10 steps in 12m18s, run alone. The neutral git config was a prefix on the lane command only (#1092)](evidence/2026-11-19-428-verify-lane-at-cb0b4fc3.md)
- [Red first for tester2's interim findings at 999f55a3: at 4ebc7bea (the tests alone) three fail: an ignored ledger is told to run check --commit, which cannot commit it; a re-created ledger's way out names only the re-creation; a missing working ledger gives finalize a bare 'No such file or directory'. Fixed in 88cb2857.](evidence/2026-11-19-428-red-ways-out-at-4ebc7bea.md)
- [At e869bf11 (the three ways out fixed, re-checked), fledge lanes run verify passed 10/10 steps in 8m18s through one-lane: unit 3113 passed (2 ignored), integration 630. The neutral git config was a prefix on the lane command only (#1092)](evidence/2026-11-19-428-verify-lane-at-e869bf11.md)
- [#428's rework at f444f569 (main a28e3042 merged in, sent back, option 3, the two reworded sentences and the three declared tests approved on the relayed sources, checked and committed): red first at c97746ba (the whole-workspace refusal named no commit or way out); option 3 at 41494bcf; the finalize-level relay removed and restored in a reopening's superseded_approval passes. With this tree's release build first on PATH, fledge lanes run verify passed 10/10 in 9m07s through one-lane (unit 3120 passed, 2 ignored, committed_approvals 20 of 20; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict against origin/main a28e3042 passes.](evidence/2026-11-20-428-rework-lane-at-f444f569.md)
- [#1120's first review round (Codex's five, corvid-agent's review) and tester2's way-out finding: red first at b00511c5 (5 of 5 fail); fixed at 5f57f347 and 41fa601f; checked at a0b997db. With this tree's release build first on PATH, fledge lanes run verify passed 10/10 in 9m03s through one-lane (unit 3126 passed, 2 ignored, committed_approvals 26 of 26; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes.](evidence/2026-11-20-428-review-round-1-at-a0b997db.md)
- [tester2's regression on #1120 (P-428-guard-migrate-digests): red first at 768e71b6 (a reopening committed without the migrate 5.0 digests was refused as not a ledger); fixed, each committed version must be an object whose approvals, reopenings and send_backs are lists; checked at c7d16a2b. fledge lanes run verify 10/10 in 7m43s through one-lane (unit 3127 passed, 2 ignored, committed_approvals 27 of 27; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix only (#1092); change audit --strict passes.](evidence/2026-11-20-428-migrate-regression-at-c7d16a2b.md)
- [Codex round 2 P1 and corvid-agent's blocker on #1120: with verification stale too, ship-status puts a ledger refusal whose way out is not check --commit first and makes that way out ship_next, so ship names the restore and never check --commit. Red first through the binary at 8e3eb2ee; a refusal that check --commit itself resolves keeps its place (review_and_ship_refuse_a_verification_commit_that_lacks_the_work, which the first fix broke at 9f2a8f18). Checked at 5ef1af6a: fledge lanes run verify 10/10 in 7m19s through one-lane (unit 3127, 2 ignored; integration 646; spec check 62/62, 168/168 files), the neutral git config as a prefix only (#1092); change audit --strict passes.](evidence/2026-11-20-428-ship-order-at-5ef1af6a.md)

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
- 2026-11-19T17:34:29Z · advance · build → test · native:verifying ^t-7ae329
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: 45b8f9dcc745b1ef6b50a9bf304e1c73b455adfc
  - by: claude-code (hc t59)
- 2026-11-19T17:34:30Z · advance · test → accept · native:verifying ^t-d2f9ab
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: 45b8f9dcc745b1ef6b50a9bf304e1c73b455adfc
  - by: claude-code (hc t59)
- 2026-11-19T19:34:20Z · back · accept → build · tester2's interim run at 999f55a3: the ways out for an ignored, a re-created and a missing ledger ^t-fffc25
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: 88cb285784d5c4b8ea0f975afed386f2af4434aa
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 4 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T19:43:10Z · advance · build → test · native:verifying ^t-a3223e
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: e869bf117949ca8f1454eb07832899f24b77940a
  - by: claude-code (hc t59)
- 2026-11-19T19:43:12Z · advance · test → accept · native:verifying ^t-86c804
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: e869bf117949ca8f1454eb07832899f24b77940a
  - by: claude-code (hc t59)
- 2026-11-20T03:40:25Z · back · accept → build · Rework before review: option 3 on Leif's relayed word (the older check names the commit and the two ways out; REQ-change-166's fourth criterion, the change's second criterion, src/change/validate_loaded_change.rs); two sentences under standing OK rule 6 (a)+(c); three existing test files declared on Leif's one-time yes; a finalize-level test for a relay in a reopening's superseded_approval. History rewritten onto the corrected main, then main a28e3042 merged in. ^t-66a289
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - gate: native:verifying
  - revision: af209abec904cd2c3aed86f4d9e5ee82d8fad585
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 6 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-20T03:40:25Z · gate-closed · build → build · Implementation review of finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold ^t-0665cb
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: af209abec904cd2c3aed86f4d9e5ee82d8fad585
  - by: claude-code (hc t59)
- 2026-11-20T03:53:12Z · gate-opened · build → build · Leif's relayed approval of #428's option 3, its two reworded sentences and its three declared test files (relayed by hc t4c through ss-lead t58) ^t-69da8f
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: 9d893560396022373d820948cc02dd35d9a75aef
  - holds: build
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-20T03:53:12Z · direct-native-write · build → build · finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state verifying → draft ^t-e5a564
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: 9d893560396022373d820948cc02dd35d9a75aef
  - direct_write:
    - change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: verifying
    - after_state: draft
  - by: claude-code (hc t59)
- 2026-11-20T03:53:12Z · gate-closed · build → build · Leif's relayed approval of #428's option 3, its two reworded sentences and its three declared test files (relayed by hc t4c through ss-lead t58) ^t-5b4a8e
  - native_change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
  - revision: 9d893560396022373d820948cc02dd35d9a75aef
  - relayed_by:
    - decided_by: 0xLeif
    - words: "\"All as recommended\" for option 3; Leif's six rules, rule 6 (a) and (c), for the two sentences; \"yes to all 3\" for the three test files"
    - where: Option 3: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 19:43Z, on the frozen ss-lead/gates/428-criterion4.md (the older check's new message, REQ-change-166's new fourth criterion, the change's new second criterion and the added path src/change/validate_loaded_change.rs), with the builder running the send-back; relayed through ss-lead (hc t58). The two sentences (REQ-change-166's still-finalizes criterion and the change's third criterion): a standing OK in Leif's six rules, relayed by hc t4c on 2026-11-19 at about 14:35Z through ss-lead (hc t58): (a) a wording fix that only makes an approved sentence match tested code, and (c) the send-back it needs; applied by ss-lead to these exact texts on 2026-11-19; these are not his words about these texts. The three test files: his one-time yes, "yes to all 3", in a Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 01:00Z, answering item 1, '#1104 and #428 edit four test files their scope doesn't list. May the builder add those paths in the send-backs already planned? No wording or behaviour changes.'; relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
    - actor: 0xLeif
  - by: claude-code (hc t59)
