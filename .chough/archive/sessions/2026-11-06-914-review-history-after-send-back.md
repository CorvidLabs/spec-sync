---
kind: session
id: 2026-11-06-914-review-history-after-send-back
aliases: [914-review-history-after-send-back]
objective: A reviewed change that is sent back can be reviewed again (#914)
intent_scope: CHANGE
modules: [change]
stage: done
stage_status: active
iteration: 1
callers: []
native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
pr: ""
criteria: [CHANGE-12, CHANGE-12.d]
requirements: [REQ-change-123]
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "New cycle from test after reset: Chough stopped at build but native SpecSync shows let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval archived, merged in #944 (d0a98a0b). advance refuses build->next on an archived change, so reset to accept and let the native:archived gate close the cycle. No human decision recorded.. Create or record the native change (specsync change new, then chough checkpoint --native-change <id>) and set the next action."
closed: 2026-11-06
outcome: done
---
# A reviewed change that is sent back can be reviewed again (#914)

## Goal

A reviewed change that is sent back can be reviewed again (#914)

## Where it stands

- Red first: a_change_reviewed_before_its_send_back_is_reviewed_again_and_finalizes fails on main cd8cab5e ('Review: recorded (unreadable)' after the send-back)
- Implemented and tested in a throwaway clone (patch kept): history binds any recorded approval; 15 focused unit + 4 integration tests green; the new unit test fails on main code at 'the review history stays readable after a send-back'
- Fix applied at c2830a2e: history binds any recorded definition approval, currency only a post-send-back one; a_change_reviewed_before_its_send_back_is_reviewed_again_and_finalizes (integration) and a_review_recorded_before_a_send_back_is_history_and_a_fresh_one_counts (unit) green, 15 send-back/scoped-review unit tests green
- Merged origin/main 918b6c56 (#899) at 4dd598ad; lesson moved to plane z=17 so it never duplicates #901's z=16
- Currency gate restored: the unit test now asserts the earlier review is Stale for the send-back reason; 15 send-back/scoped-review unit tests and the #914 integration test green; lane lint clean on 1.89

## Decisions and why

- orc t18 (2026-11-05, for Leif overnight): fix #914 in its own worktree ss-914 and PR, test-first; #901 waits untouched at 36ad7d5e until #914 merges
- decided: history binds any recorded definition approval, currency binds only a post-send-back approval, because a pre-send-back review was valid evidence when recorded (CHANGE-12.c) and must never let finalize through (CHANGE-12.b)
- rejected: dropping pre-send-back attempts at send-back, because CHANGE-12.c and the append-only committed history forbid losing evidence
- failed: an explicit send-back check in review currency, because it is unreachable (a byte-identical re-approval records a live approval with the same digest; any other re-scope already fails the contract check); removed, and the definition no longer claims a pre-send-back review never counts after an identical re-approval
- Leif decided "Leif via orc t18: approve intent CHANGE-12.d and the scope of let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval": "Ok yes I approve all" (orc tab, 2026-11-06 06:1x MST, in reply to the orc's morning brief; relayed by orc t18, who recorded the native approval as 0xLeif); recorded by ss-914 Claude agent; a relayed claim, not authenticated
- orc t18 (2026-11-06): #901 first, #914 next, #902 after; never record a SpecSync review as Leif unless he reviewed on GitHub; send 'ready for Leif's GitHub review #N at <head>'
- decided (verify, 2026-11-06): keep CHANGE-12.b explicit in scoped_review_currency, checked before the contract digest so it is reachable and tested; the earlier removal was of an unreachable check placed after the digest check, and leaving none made definition_approver_for_review_contract dead code (clippy -D dead-code caught it)

## What was verified

## Open threads

## Timeline

- 2026-11-06T06:43:37Z · gate-opened · intent → intent · Leif via orc t18: approve intent CHANGE-12.d and the scope of let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval ^t-8bf0bb
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - revision: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
- 2026-11-06T13:18:13Z · gate-closed · intent → intent · Leif via orc t18: approve intent CHANGE-12.d and the scope of let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval ^t-1f652d
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - revision: ef057270c9523f2c5bfd3af188b5c5b9911262e6
  - relayed_by:
    - decided_by: Leif
    - words: Ok yes I approve all
    - where: orc tab, 2026-11-06 06:1x MST, in reply to the orc's morning brief; relayed by orc t18, who recorded the native approval as 0xLeif
    - recorded_by: ss-914 Claude agent
- 2026-11-06T13:18:18Z · advance · intent → design ·  ^t-fa4edc
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - revision: ef057270c9523f2c5bfd3af188b5c5b9911262e6
- 2026-11-06T13:18:27Z · skip · design → plan · CLI and lifecycle-record change; no visuals or interactions change ^t-825af8
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - revision: ef057270c9523f2c5bfd3af188b5c5b9911262e6
- 2026-11-06T13:18:28Z · advance · plan → build · native:approved ^t-c6b531
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - gate: native:approved
  - revision: ef057270c9523f2c5bfd3af188b5c5b9911262e6
- 2026-11-06T18:03:52Z · advance · build → test · native change archived by #944 (d0a98a0b) ^t-6eac63
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - gate: native:archived
  - revision: 29b6dd3feb5c2aa4cc6db7a2f10a86c6b2c7c8b2
- 2026-11-06T18:04:07Z · reset · test → accept · Chough stopped at build but native SpecSync shows let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval archived, merged in #944 (d0a98a0b). advance refuses build->next on an archived change, so reset to accept and let the native:archived gate close the cycle. No human decision recorded. ^t-dc0fd1
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - gate: native:archived
  - revision: 29b6dd3feb5c2aa4cc6db7a2f10a86c6b2c7c8b2
  - refresh: accept: native verification and the human implementation review
- 2026-11-06T18:04:20Z · advance · accept → done · native change archived by #944 (d0a98a0b) ^t-8aafb0
  - native_change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
  - gate: native:archived
  - revision: 29b6dd3feb5c2aa4cc6db7a2f10a86c6b2c7c8b2

Closed 2026-11-06: done.

## Receipts

```yaml
receipts: []
```
