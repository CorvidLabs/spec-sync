---
kind: session
id: 2026-11-06-960-fresh-review-after-send-back
aliases: [960-fresh-review-after-send-back]
objective: After a send-back a change needs a fresh review before it can ship, even when its scope comes back unchanged (#960)
intent_scope: CHANGE
modules: [change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
pr: ""
criteria: [CHANGE-12.e]
requirements: [REQ-change-123]
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "check --commit, audit --strict, pre-push, push, PR (Fixes #960), verify, ready for Leif's GitHub review"
closed: 2026-11-06
outcome: closed-without-done
---
# After a send-back a change needs a fresh review before it can ship, even when its scope comes back unchanged (#960)

## Goal

After a send-back a change needs a fresh review before it can ship, even when its scope comes back unchanged (#960)

## Where it stands

- Red first: a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged fails on main 8a251d7a (status scoped_review_current true, next action finalize)
- Implemented: SendBackRecord.cancelled_review_attempts written on every new send-back; scoped_review_currency refuses a review recorded before the latest send-back (timestamp fallback for legacy send-backs); #960 integration test and 2 new unit tests green; 17 send-back/scoped-review unit and 5 send-back/review integration tests green

## Decisions and why

- Leif via orc t18 (2026-11-06, 'Capture both'): capture CHANGE-12.e verbatim; fix #960 as its own PR, test-first
- decided: a send-back records cancelled_review_attempts beside cancelled_approvals, and currency refuses a review recorded before the latest send-back; a send-back without the count uses a strict timestamp rule, which changes none of the 4 archives on main that have send-backs
- rejected: timestamps alone (same-second ambiguity in scripted flows), a per-review approval index (needs a review schema change plus a fallback anyway), dropping earlier attempts (CHANGE-12.c)
- Leif decided "Leif via orc t18: approve the scope of require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged": "Leif approved #960 (relayed)" (orc tab, 2026-11-06; relayed by orc t18, who recorded the native approval as 0xLeif at 71e0080d); recorded by ss-960 Claude agent; a relayed claim, not authenticated

## What was verified

## Open threads

## Timeline

- 2026-11-06T18:30:13Z · gate-opened · intent → intent · Leif via orc t18: approve the scope of require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged ^t-bed7a6
  - native_change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
  - revision: 8a251d7a553f5c98dc1f2f505262f4b0d055e3f7
- 2026-11-06T18:36:50Z · gate-closed · intent → intent · Leif via orc t18: approve the scope of require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged ^t-913573
  - native_change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
  - revision: 8daa19b31435446e235c65d158cb762f8ef4b894
  - relayed_by:
    - decided_by: Leif
    - words: Leif approved #960 (relayed)
    - where: orc tab, 2026-11-06; relayed by orc t18, who recorded the native approval as 0xLeif at 71e0080d
    - recorded_by: ss-960 Claude agent
- 2026-11-06T18:36:50Z · advance · intent → design ·  ^t-403374
  - native_change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
  - revision: 8daa19b31435446e235c65d158cb762f8ef4b894
- 2026-11-06T18:36:50Z · advance · design → plan ·  ^t-5e58b9
  - native_change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
  - revision: 8daa19b31435446e235c65d158cb762f8ef4b894
- 2026-11-06T18:36:50Z · advance · plan → build · native:approved ^t-142bfa
  - native_change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
  - gate: native:approved
  - revision: 8daa19b31435446e235c65d158cb762f8ef4b894

Closed 2026-11-06: closed-without-done.

## Receipts

```yaml
receipts: []
```
