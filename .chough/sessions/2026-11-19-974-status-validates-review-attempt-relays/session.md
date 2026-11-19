---
kind: session
id: 2026-11-19-974-status-validates-review-attempt-relays
aliases: [974-status-validates-review-attempt-relays]
objective: status and show check every review attempt's relay before showing the claims (#974)
intent_scope: CHANGE
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
pr: ""
criteria: [CHANGE-13.b]
requirements: []
cases: []
design: []
files: []
observations: [2026-11-19-974-red-control-on-main-3bebe8b0, 2026-11-19-974-verify-lane-at-bf0014ad]
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
    holds: accept
    drives: review
    opened: 2026-11-19T08:25:57Z
run_consents: []
next: Pushed on fix/relays-998-972-974, no PR until ss-lead says. Then Leif's review, relayed by hc t4c through ss-lead; record it with chough change ship and his words. Record no review before that.
---
# status and show check every review attempt's relay before showing the claims (#974)

## Goal

status and show check every review attempt's relay before showing the claims (#974)

## Where it stands

- Merged the 11-13 branch onto main 3bebe8b0 on branch fix/relays-998-972-974; one conflict, the CHANGELOG line, both kept
- Refreshed onto main 3bebe8b0 with #998 and #972; one conflict, the CHANGELOG line, both kept; re-checked; red control on main and the verify lane recorded

## Decisions and why

- decided: authorization is Leif's own decision ('#974 check old reviews', coordinator chat 2026-11-13T03:57Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form on 2026-11-13 because that is when he gave it; this session only refreshes the change onto main
- decided: this change had no Chough session because its 11-13 branch was cut before the Chough 0.6.0 migration (#1073) reached main; ss-builder (hc t59) opened this one on 2026-11-19 for the batch 6 refresh

## What was verified

- [Red-first control: on main 3bebe8b0 with the three 11-13 failing-test commits applied (890c154a, 5d86b391, 2e32795b: the tests, plus 890c154a's compile scaffolding, a 'within' field that is always empty and a 'place' method that keeps main's wording), recorded_claims_refuse_a_damaged_earlier_review_attempt_relay fails: the claims are shown although an earlier review attempt's relay is damaged. This is the defect #974 reports, fixed by this change; not a new bug.](evidence/2026-11-19-974-red-control-on-main-3bebe8b0.md)
- [At bf0014ad (#998, #972 and #974 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 19m36s on a machine also running another lane: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 627 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes recorded_claims_refuse_a_damaged_earlier_review_attempt_relay.](evidence/2026-11-19-974-verify-lane-at-bf0014ad.md)

## Open threads

## Timeline

- 2026-11-19T07:25:44Z · start · intent → intent · status and show check every review attempt's relay before showing the claims (#974) ^t-5af0e6
  - revision: ecffb2b24331d9bcda56e0732f933eaae18764d8
  - by: claude-code (hc t59)
- 2026-11-19T07:42:20Z · advance · intent → design ·  ^t-ffe701
  - native_change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
  - revision: 2a61f3c6c7e9555c49a945210cda3604f6630ba6
  - by: claude-code (hc t59)
- 2026-11-19T07:42:20Z · skip · design → plan · No visuals or interactions: lifecycle records and CLI output only ^t-2cda50
  - native_change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
  - revision: 2a61f3c6c7e9555c49a945210cda3604f6630ba6
  - by: claude-code (hc t59)
- 2026-11-19T07:42:25Z · advance · plan → build · native:verifying ^t-55d1fd
  - native_change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
  - gate: native:verifying
  - revision: 2a61f3c6c7e9555c49a945210cda3604f6630ba6
  - by: claude-code (hc t59)
- 2026-11-19T08:25:54Z · advance · build → test · native:verifying ^t-fb7239
  - native_change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
  - gate: native:verifying
  - revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
  - by: claude-code (hc t59)
- 2026-11-19T08:25:57Z · advance · test → accept · native:verifying ^t-00a3f4
  - native_change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
  - gate: native:verifying
  - revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
  - by: claude-code (hc t59)
- 2026-11-19T08:25:57Z · gate-opened · accept → accept · Implementation review of status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims ^t-2b46ed
  - native_change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
  - revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
