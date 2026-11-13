---
kind: session
id: 2026-11-06-977-same-second-review-test
aliases: [977-same-second-review-test]
objective: "A failing test always means something is actually wrong, never that it ran at an unlucky moment: make the #969 same-second review flake deterministic, then fix the assertion (#977)"
intent_scope: CI
modules: [change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
pr: ""
criteria: [CI-7]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Prove ship in a throwaway clone with test actors, then ready for approval; after re-approval: check --commit, verify, Leif re-reviews, ship"
closed: 2026-11-06
outcome: closed-without-done
---
# A failing test always means something is actually wrong, never that it ran at an unlucky moment: make the #969 same-second review flake deterministic, then fix the assertion (#977)

## Goal

A failing test always means something is actually wrong, never that it ran at an unlucky moment: make the #969 same-second review flake deterministic, then fix the assertion (#977)

## Where it stands

- Red first: with PinnedTestClock the #969 unit test fails 3/3 runs at assert_ne!(fresh, earlier); every earlier assertion passes in the same second
- Fixed: the assertion checks the attempt ledger grew by one and its last entry is the fresh current review; 3/3 pinned and 1 unpinned run green; 17 send-back/scoped-review unit tests green; merged main through #969 (squash) at 2a647ae6, keeping only #977's own changes

## Decisions and why

- Leif via orc t18 (2026-11-06, 'Broad'): capture CI-7 'A failing test always means something is actually wrong, never that it ran at an unlucky moment.'; #977 as its own test-only change up to ready for approval
- decided: a cfg(test) thread-local clock pin (PinnedTestClock) so the same second is a fixture; fix by asserting the attempt ledger grew by one and its last entry is current; branch stacks on #969's archive tip 5181fb2f until #969 merges
- Leif decided "Leif via orc t18: approve the scope of make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality": "Leif approved #977 (relayed)" (orc tab, 2026-11-06; relayed by orc t18, who recorded the native approval as 0xLeif at 8590976e); recorded by ss-977 Claude agent; a relayed claim, not authenticated
- orc t18 relaying Leif (2026-11-06): option (a), send-back recorded as 0xLeif; re-scope affected_specs [change], content unchanged
- decided: KEEP the uncommitted review-attempts.json/review.json, not discard: the send-back recorded cancelled_review_attempts=1 counting that attempt, so deleting it would put the fresh review at index 0 and make it never current; CHANGE-12.c and the append-only history also keep it
- decided: affected_specs [change] with no_spec_change kept true (acceptance owners come from affected_specs; no delta needed), so the content stays unchanged
- Leif decided "Leif via orc t18: re-approve the re-scoped make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality": "Re-approval recorded as 0xLeif at d429dda1" (Herdr orc tab, 2026-11-06, relayed by orc t18 (re-approval recorded as 0xLeif at d429dda1)); recorded by ss-s3 Claude agent; a relayed claim, not authenticated

## What was verified

## Open threads

## Timeline

- 2026-11-06T19:17:25Z · gate-opened · intent → intent · Leif via orc t18: approve the scope of make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality ^t-3b993e
  - native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
  - revision: 5181fb2f6952ef086eed6cb0dff976b41123cd77
- 2026-11-06T20:56:11Z · gate-closed · intent → intent · Leif via orc t18: approve the scope of make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality ^t-754693
  - native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
  - revision: 2a647ae6354e0cf5976422c35cfc0cc8eec2fba2
  - relayed_by:
    - decided_by: Leif
    - words: Leif approved #977 (relayed)
    - where: orc tab, 2026-11-06; relayed by orc t18, who recorded the native approval as 0xLeif at 8590976e
    - recorded_by: ss-977 Claude agent
- 2026-11-06T20:56:12Z · advance · intent → design ·  ^t-41fea6
  - native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
  - revision: 2a647ae6354e0cf5976422c35cfc0cc8eec2fba2
- 2026-11-06T20:56:12Z · advance · design → plan ·  ^t-1c9e43
  - native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
  - revision: 2a647ae6354e0cf5976422c35cfc0cc8eec2fba2
- 2026-11-06T20:56:13Z · advance · plan → build · native:approved ^t-3d5c97
  - native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
  - gate: native:approved
  - revision: 2a647ae6354e0cf5976422c35cfc0cc8eec2fba2
- 2026-11-06T21:51:49Z · gate-opened · build → build · Leif via orc t18: re-approve the re-scoped make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality ^t-1a3303
  - native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
  - revision: 86367d1b2cd0610d7aba9d0fa8520fe7ddd2cc62
- 2026-11-06T23:35:35Z · gate-closed · build → build · Leif via orc t18: re-approve the re-scoped make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality ^t-6ba37a
  - native_change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
  - revision: 4d1febe1cd34a4365829026657b691a2a0704fa6
  - relayed_by:
    - decided_by: Leif
    - words: Re-approval recorded as 0xLeif at d429dda1
    - where: Herdr orc tab, 2026-11-06, relayed by orc t18 (re-approval recorded as 0xLeif at d429dda1)
    - recorded_by: ss-s3 Claude agent

Closed 2026-11-06: closed-without-done.

## Receipts

```yaml
receipts: []
```
