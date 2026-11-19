---
kind: session
id: 2026-11-13-1074-literal-baseline-history-probe
aliases: [1074-literal-baseline-history-probe]
objective: The workflow-v2 baseline history probe reads the baseline path literally, so a pathspec-magic project folder keeps its history (#1074)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
pr: ""
criteria: [SPEED-1.a.2]
requirements: []
cases: []
design: []
files: [src/change/validate_loaded_change.rs, src/change_tests/fast_archived_reads.rs]
observations:
  - 2026-11-13-at-e21aefacf-fix-1074-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-a
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
    holds: accept
    drives: review
    opened: 2026-11-13T19:22:48Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# The workflow-v2 baseline history probe reads the baseline path literally, so a pathspec-magic project folder keeps its history (#1074)

## Goal

The workflow-v2 baseline history probe reads the baseline path literally, so a pathspec-magic project folder keeps its history (#1074)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#1074 + #1075 (new)', coordinator chat 2026-11-13T04:39Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the change is narrow and an existing lesson covers it

## What was verified

- [At e21aefacf (fix/1074 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_never_committed_baseline_is_not_read_as_deleted_in_a_pathspec_magic_project_directory.](evidence/2026-11-13-at-e21aefacf-fix-1074-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, a_never_committed_baseline_is_not_read_as_deleted_in_a_pathspec_magic_project_directory fails. The failure is the defect #1074 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-a.md)

## Open threads

## Timeline

- 2026-11-13T18:40:17Z · start · intent → intent · The workflow-v2 baseline history probe reads the baseline path literally, so a pathspec-magic project folder keeps its history (#1074) ^t-cf6c9b
  - revision: e21aefacff8900fcf8fddce0d403f58746dc8333
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:19Z · advance · intent → design ·  ^t-f33202
  - native_change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
  - revision: e21aefacff8900fcf8fddce0d403f58746dc8333
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:19Z · skip · design → plan · CLI text and Git read behaviour only: no visuals or interactions ^t-e37132
  - native_change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
  - revision: e21aefacff8900fcf8fddce0d403f58746dc8333
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:21Z · advance · plan → build · native:verifying ^t-8f54b5
  - native_change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
  - gate: native:verifying
  - revision: e21aefacff8900fcf8fddce0d403f58746dc8333
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:21Z · advance · build → test · native:verifying ^t-df7f8b
  - native_change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
  - gate: native:verifying
  - revision: e21aefacff8900fcf8fddce0d403f58746dc8333
  - by: claude-code (hc t38p)
- 2026-11-13T19:22:47Z · advance · test → accept · native:verifying ^t-344a17
  - native_change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
  - gate: native:verifying
  - revision: e21aefacff8900fcf8fddce0d403f58746dc8333
  - by: claude-code (hc t38p)
- 2026-11-13T19:22:47Z · gate-opened · accept → accept · Implementation review of the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally ^t-a45678
  - native_change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
  - revision: e21aefacff8900fcf8fddce0d403f58746dc8333
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
