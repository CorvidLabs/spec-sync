---
kind: session
id: 2026-11-13-475-check-fix-json-stdout
aliases: [475-check-fix-json-stdout]
objective: check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: check-fix-keeps-stdout-one-json-document-in-machine-formats
pr: ""
criteria: [AGENT-2]
requirements: []
cases: []
design: []
files: [src/commands/check.rs, tests/integration/fix.rs]
observations:
  - 2026-11-13-at-75030f60a-fix-475-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-fix
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of check-fix-keeps-stdout-one-json-document-in-machine-formats
    holds: accept
    drives: review
    opened: 2026-11-13T17:51:21Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475)

## Goal

check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#475 clean JSON', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the fix is a narrow correction an existing lesson already covers

## What was verified

- [At 75030f60a (fix/475 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including fix::fix_with_json_output and fix::fix_dry_run_with_json_output_is_one_json_document.](evidence/2026-11-13-at-75030f60a-fix-475-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, fix::fix_with_json_output and fix::fix_dry_run_with_json_output_is_one_json_document fails. The failure is the defect #475 reports, fixed by this change; it is not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-fix.md)

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
