---
kind: session
id: 2026-11-19-1012-merge-keeps-quoted-timestamps
aliases: [1012-merge-keeps-quoted-timestamps]
objective: A clean merge-driver union keeps a quoted YAML date quoted (#1012)
intent_scope: MERGE
modules: [merge]
stage: done
stage_status: active
iteration: 1
callers: []
native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
pr: "https://github.com/CorvidLabs/spec-sync/pull/1151"
criteria: [MERGE-5.a]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-1012-red-control-on-main-3bebe8b0
  - 2026-11-19-1012-verify-lane-at-f8b341c5
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Merge PR #1151 when the archived revision passes the required checks and review is complete."
---
# A clean merge-driver union keeps a quoted YAML date quoted (#1012)

## Goal

A clean merge-driver union keeps a quoted YAML date quoted (#1012)

## Where it stands

- Merged the 11-13 branch onto main 3bebe8b0 on branch fix/cli-432-475-1012; no conflict
- Refreshed onto main 3bebe8b0 with #432 and #475 on fix/cli-432-475-1012; no conflict; its CHANGELOG entry moved under Fixed after Changed; re-checked; red control on main and the verify lane recorded

## Decisions and why

- decided: authorization is Leif's own decision ('#1012 quoted dates', coordinator chat 2026-11-13T03:57Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form on 2026-11-13 because that is when he gave it; this session only refreshes the change onto main
- decided: this change had no Chough session because its 11-13 branch was cut before the Chough 0.6.0 migration (#1073) reached main; ss-builder (hc t59) opened this one on 2026-11-19 for the batch 10 refresh
- 0xLeif said in the November 23 SpecSync 6.4.2 release conversation: I inspected the remaining PRs and accept archival when qualified, including both ID renewals and #1114. Native review is recorded in .specsync/archive/changes/2026-11-23-a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted/review.json. Withdraw the duplicated session gate after native finalization.

## What was verified

- [Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, merge_driver::tests::quoted_yaml_timestamps_stay_strings_after_a_union fails: the quoted 2026-11-07 loses its quotes in the merged list. This is the defect #1012 reports, fixed by this change; not a new bug.](evidence/2026-11-19-1012-red-control-on-main-3bebe8b0.md)
- [At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes merge_driver::tests::quoted_yaml_timestamps_stay_strings_after_a_union.](evidence/2026-11-19-1012-verify-lane-at-f8b341c5.md)

## Open threads

## Timeline

- 2026-11-19T08:44:40Z · start · intent → intent · A clean merge-driver union keeps a quoted YAML date quoted (#1012) ^t-053ffe
  - revision: d8a3ba6c77873600e87b269625bd2d4d4c57613d
  - by: claude-code (hc t59)
- 2026-11-19T08:46:31Z · advance · intent → design ·  ^t-278490
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - revision: acfa0a65669ce37dff6cda489246ca91ead95dbe
  - by: claude-code (hc t59)
- 2026-11-19T08:46:31Z · skip · design → plan · No visuals or interactions: a merge driver's output only ^t-1e7d6b
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - revision: acfa0a65669ce37dff6cda489246ca91ead95dbe
  - by: claude-code (hc t59)
- 2026-11-19T08:46:33Z · advance · plan → build · native:verifying ^t-2235d0
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - gate: native:verifying
  - revision: acfa0a65669ce37dff6cda489246ca91ead95dbe
  - by: claude-code (hc t59)
- 2026-11-19T08:59:58Z · advance · build → test · native:verifying ^t-e8fdc1
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - gate: native:verifying
  - revision: f8b341c574e0b61113983999a8dad19506f9eb24
  - by: claude-code (hc t59)
- 2026-11-19T08:59:59Z · advance · test → accept · native:verifying ^t-8106bf
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - gate: native:verifying
  - revision: f8b341c574e0b61113983999a8dad19506f9eb24
  - by: claude-code (hc t59)
- 2026-11-19T08:59:59Z · gate-opened · accept → accept · Implementation review of a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted ^t-59ce84
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - revision: f8b341c574e0b61113983999a8dad19506f9eb24
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
- 2026-11-24T00:12:27Z · gate-closed · accept → accept · Implementation review of a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted ^t-b6b956
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - revision: da57656d17af44e7945dbb4fa1c2267402838647
  - by: unknown agent (hc t14v)
- 2026-11-24T00:12:31Z · advance · accept → done · Native scoped review and finalization for PR #1151 are recorded in the archive. ^t-bfb969
  - native_change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
  - gate: native:archived
  - revision: da57656d17af44e7945dbb4fa1c2267402838647
  - by: unknown agent (hc t14v)
