---
kind: session
id: 2026-11-19-972-escape-relay-names
aliases: [972-escape-relay-names]
objective: The relayed decider-mismatch error escapes both names (#972)
intent_scope: CHANGE
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-relayed-decider-mismatch-error-escapes-both-names
pr: ""
criteria: [CHANGE-13.d]
requirements: []
cases: []
design: []
files: []
observations: [2026-11-19-972-red-control-on-main-3bebe8b0, 2026-11-19-972-verify-lane-at-bf0014ad]
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-relayed-decider-mismatch-error-escapes-both-names
    holds: accept
    drives: review
    opened: 2026-11-19T08:25:50Z
run_consents: []
next: Pushed on fix/relays-998-972-974, no PR until ss-lead says. Then Leif's review, relayed by hc t4c through ss-lead; record it with chough change ship and his words. Record no review before that.
---
# The relayed decider-mismatch error escapes both names (#972)

## Goal

The relayed decider-mismatch error escapes both names (#972)

## Where it stands

- Merged the 11-13 branch onto main 3bebe8b0 on branch fix/relays-998-972-974; one conflict, the CHANGELOG section, both kept
- Refreshed onto main 3bebe8b0 with #998 and #974; one conflict, the CHANGELOG section, both kept; re-checked; red control on main and the verify lane recorded

## Decisions and why

- decided: authorization is Leif's own decision ('#972 escape names', coordinator chat 2026-11-13T03:57Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form on 2026-11-13 because that is when he gave it; this session only refreshes the change onto main
- decided: this change had no Chough session because its 11-13 branch was cut before the Chough 0.6.0 migration (#1073) reached main; ss-builder (hc t59) opened this one on 2026-11-19 for the batch 6 refresh

## What was verified

- [Red-first control: on main 3bebe8b0 with the three 11-13 failing-test commits applied (890c154a, 5d86b391, 2e32795b: the tests, plus 890c154a's compile scaffolding, a 'within' field that is always empty and a 'place' method that keeps main's wording), the_decider_mismatch_error_escapes_both_names fails: the error for a relayed decision that names another decider prints a raw escape sequence from the name. This is the defect #972 reports, fixed by this change; not a new bug.](evidence/2026-11-19-972-red-control-on-main-3bebe8b0.md)
- [At bf0014ad (#998, #972 and #974 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 19m36s on a machine also running another lane: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 627 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes the_decider_mismatch_error_escapes_both_names.](evidence/2026-11-19-972-verify-lane-at-bf0014ad.md)

## Open threads

## Timeline

- 2026-11-19T07:25:26Z · start · intent → intent · The relayed decider-mismatch error escapes both names (#972) ^t-0adaa3
  - revision: 92ece2cd0e21f05c4279e909b08da9e82e1d53b4
  - by: claude-code (hc t59)
- 2026-11-19T07:42:09Z · advance · intent → design ·  ^t-347b3c
  - native_change: the-relayed-decider-mismatch-error-escapes-both-names
  - revision: 2a61f3c6c7e9555c49a945210cda3604f6630ba6
  - by: claude-code (hc t59)
- 2026-11-19T07:42:09Z · skip · design → plan · No visuals or interactions: lifecycle records and CLI output only ^t-be66a1
  - native_change: the-relayed-decider-mismatch-error-escapes-both-names
  - revision: 2a61f3c6c7e9555c49a945210cda3604f6630ba6
  - by: claude-code (hc t59)
- 2026-11-19T07:42:14Z · advance · plan → build · native:verifying ^t-ecf223
  - native_change: the-relayed-decider-mismatch-error-escapes-both-names
  - gate: native:verifying
  - revision: 2a61f3c6c7e9555c49a945210cda3604f6630ba6
  - by: claude-code (hc t59)
- 2026-11-19T08:25:46Z · advance · build → test · native:verifying ^t-599eb3
  - native_change: the-relayed-decider-mismatch-error-escapes-both-names
  - gate: native:verifying
  - revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
  - by: claude-code (hc t59)
- 2026-11-19T08:25:50Z · advance · test → accept · native:verifying ^t-cf1d5a
  - native_change: the-relayed-decider-mismatch-error-escapes-both-names
  - gate: native:verifying
  - revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
  - by: claude-code (hc t59)
- 2026-11-19T08:25:50Z · gate-opened · accept → accept · Implementation review of the-relayed-decider-mismatch-error-escapes-both-names ^t-a76cd2
  - native_change: the-relayed-decider-mismatch-error-escapes-both-names
  - revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
