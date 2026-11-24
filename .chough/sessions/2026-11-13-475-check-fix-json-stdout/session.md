---
kind: session
id: 2026-11-13-475-check-fix-json-stdout
aliases: [475-check-fix-json-stdout]
objective: check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475)
intent_scope: ""
modules: [change]
stage: done
stage_status: active
iteration: 1
callers: []
native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
pr: "https://github.com/CorvidLabs/spec-sync/pull/1151"
criteria: [AGENT-2]
requirements: []
cases: []
design: []
files: [src/commands/check.rs, tests/integration/fix.rs]
observations:
  - 2026-11-13-at-75030f60a-fix-475-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-fix
  - 2026-11-19-475-red-control-on-main-3bebe8b0
  - 2026-11-19-475-verify-lane-at-f8b341c5
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Merge PR #1151 when the archived revision passes the required checks and review is complete."
---
# check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475)

## Goal

check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 with #432 and #1012 on fix/cli-432-475-1012; no conflict; a CHANGELOG Fixed line added; re-checked; red control on main and the verify lane recorded

## Decisions and why

- decided: authorization is Leif's own scope approval ('#475 clean JSON', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the fix is a narrow correction an existing lesson already covers
- 0xLeif said in the November 23 SpecSync 6.4.2 release conversation: I inspected the remaining PRs and accept archival when qualified, including both ID renewals and #1114. Native review is recorded in .specsync/archive/changes/2026-11-23-check-fix-keeps-stdout-one-json-document-in-machine-formats/review.json. Withdraw the duplicated session gate after native finalization.

## What was verified

- [At 75030f60a (fix/475 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including fix::fix_with_json_output and fix::fix_dry_run_with_json_output_is_one_json_document.](evidence/2026-11-13-at-75030f60a-fix-475-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, fix::fix_with_json_output and fix::fix_dry_run_with_json_output_is_one_json_document fails. The failure is the defect #475 reports, fixed by this change; it is not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-fix.md)
- [Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, fix::fix_with_json_output and fix::fix_dry_run_with_json_output_is_one_json_document fail: stdout is not one JSON document, a per-spec 'added 1 export(s)' or 'would add 1 export(s)' line comes before it. This is the defect #475 reports, fixed by this change; not a new bug.](evidence/2026-11-19-475-red-control-on-main-3bebe8b0.md)
- [At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes fix::fix_with_json_output and fix::fix_dry_run_with_json_output_is_one_json_document.](evidence/2026-11-19-475-verify-lane-at-f8b341c5.md)

## Open threads

## Timeline

- 2026-11-13T16:59:21Z · start · intent → intent · check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475) ^t-ba5ae1
  - revision: 75030f60aa1313747e49e386fa59690e049d13a4
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:25Z · advance · intent → design ·  ^t-4e9834
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - revision: 75030f60aa1313747e49e386fa59690e049d13a4
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:25Z · skip · design → plan · CLI or CI behaviour only: no visuals or interactions ^t-f2c9b6
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - revision: 75030f60aa1313747e49e386fa59690e049d13a4
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:29Z · advance · plan → build · native:verifying ^t-5def69
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - gate: native:verifying
  - revision: 75030f60aa1313747e49e386fa59690e049d13a4
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:29Z · advance · build → test · native:verifying ^t-7410bc
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - gate: native:verifying
  - revision: 75030f60aa1313747e49e386fa59690e049d13a4
  - by: claude-code (hc t38p)
- 2026-11-13T17:51:21Z · advance · test → accept · native:verifying ^t-94c615
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - gate: native:verifying
  - revision: 75030f60aa1313747e49e386fa59690e049d13a4
  - by: claude-code (hc t38p)
- 2026-11-13T17:51:21Z · gate-opened · accept → accept · Implementation review of check-fix-keeps-stdout-one-json-document-in-machine-formats ^t-da75fd
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - revision: 75030f60aa1313747e49e386fa59690e049d13a4
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
- 2026-11-19T08:44:56Z · back · accept → build · Batch 10 refresh onto main 3bebe8b0 (ss-builder, hc t59): the change gets its CHANGELOG Fixed line and is re-checked on the merged branch ^t-941f83
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - gate: native:verifying
  - revision: d8a3ba6c77873600e87b269625bd2d4d4c57613d
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 2 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T08:59:53Z · advance · build → test · native:verifying ^t-66adb9
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - gate: native:verifying
  - revision: f8b341c574e0b61113983999a8dad19506f9eb24
  - by: claude-code (hc t59)
- 2026-11-19T08:59:54Z · advance · test → accept · native:verifying ^t-fcdda5
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - gate: native:verifying
  - revision: f8b341c574e0b61113983999a8dad19506f9eb24
  - by: claude-code (hc t59)
- 2026-11-24T00:12:19Z · gate-closed · accept → accept · Implementation review of check-fix-keeps-stdout-one-json-document-in-machine-formats ^t-e3d8b7
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - revision: da57656d17af44e7945dbb4fa1c2267402838647
  - by: unknown agent (hc t14v)
- 2026-11-24T00:12:23Z · advance · accept → done · Native scoped review and finalization for PR #1151 are recorded in the archive. ^t-98da4c
  - native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
  - gate: native:archived
  - revision: da57656d17af44e7945dbb4fa1c2267402838647
  - by: unknown agent (hc t14v)
