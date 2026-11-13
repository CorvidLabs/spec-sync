---
kind: session
id: 2026-11-13-945-961-upstream-archive-lookup
aliases: [945-961-upstream-archive-lookup]
objective: "The upstream archive lookup reads tree entries as bytes and knows an archive HEAD already holds after an upstream merge (#945, #961)"
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
pr: ""
criteria: [SHIP-3.c]
requirements: []
cases: []
design: []
files: [src/change/consider_accepted_evidence_anchor.rs, src/change_tests/upstream_archive.rs]
observations:
  - 2026-11-13-at-57bc249e1-fix-945-961-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests-an
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
    holds: accept
    drives: review
    opened: 2026-11-13T19:22:35Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# The upstream archive lookup reads tree entries as bytes and knows an archive HEAD already holds after an upstream merge (#945, #961)

## Goal

The upstream archive lookup reads tree entries as bytes and knows an archive HEAD already holds after an upstream merge (#945, #961)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#945 + #961 (one PR)', coordinator chat 2026-11-13T04:39Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the change is narrow and an existing lesson covers it

## What was verified

- [At 57bc249e1 (fix/945-961 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3072 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including the upstream_archive tests for a non-UTF-8 package and an upstream rename HEAD already holds.](evidence/2026-11-13-at-57bc249e1-fix-945-961-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new tests, an_unrelated_non_utf8_archive_package_does_not_stop_the_upstream_lookup and an_upstream_rename_of_an_archive_head_already_contains_is_not_reported fail. These are the defects #945 and #961 report, fixed by this change; not new bugs.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests-an.md)

## Open threads

## Timeline

- 2026-11-13T18:40:12Z · start · intent → intent · The upstream archive lookup reads tree entries as bytes and knows an archive HEAD already holds after an upstream merge (#945, #961) ^t-137757
  - revision: 57bc249e1a12a47a465304ddd64de2cc942f8b01
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:14Z · advance · intent → design ·  ^t-4be702
  - native_change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
  - revision: 57bc249e1a12a47a465304ddd64de2cc942f8b01
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:15Z · skip · design → plan · CLI text and Git read behaviour only: no visuals or interactions ^t-aaa9ce
  - native_change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
  - revision: 57bc249e1a12a47a465304ddd64de2cc942f8b01
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:16Z · advance · plan → build · native:verifying ^t-6ab36d
  - native_change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
  - gate: native:verifying
  - revision: 57bc249e1a12a47a465304ddd64de2cc942f8b01
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:17Z · advance · build → test · native:verifying ^t-df7875
  - native_change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
  - gate: native:verifying
  - revision: 57bc249e1a12a47a465304ddd64de2cc942f8b01
  - by: claude-code (hc t38p)
- 2026-11-13T19:22:35Z · advance · test → accept · native:verifying ^t-541aae
  - native_change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
  - gate: native:verifying
  - revision: 57bc249e1a12a47a465304ddd64de2cc942f8b01
  - by: claude-code (hc t38p)
- 2026-11-13T19:22:35Z · gate-opened · accept → accept · Implementation review of the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream ^t-b72bc2
  - native_change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
  - revision: 57bc249e1a12a47a465304ddd64de2cc942f8b01
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
