---
kind: session
id: 2026-11-13-990-literal-cutoff-archive-root
aliases: [990-literal-cutoff-archive-root]
objective: The cutoff check probes the archive root with a literal pathspec, so a project folder named like pathspec magic is not reported unreadable (#990)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
pr: ""
criteria: [SPEED-1.a.2]
requirements: []
cases: []
design: []
files: [src/change/validate_loaded_change.rs, src/change_tests/fast_archived_reads.rs]
observations:
  - 2026-11-13-at-a5f25b136-fix-990-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
    holds: accept
    drives: review
    opened: 2026-11-13T16:58:19Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# The cutoff check probes the archive root with a literal pathspec, so a project folder named like pathspec magic is not reported unreadable (#990)

## Goal

The cutoff check probes the archive root with a literal pathspec, so a project folder named like pathspec magic is not reported unreadable (#990)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#990 odd folder names', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the fix is a narrow correction an existing lesson already covers

## What was verified

- [At a5f25b136 (fix/990 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory.](evidence/2026-11-13-at-a5f25b136-fix-990-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, change::tests::a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory fails. The failure is the defect #990 reports, fixed by this change; it is not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

## Open threads

## Timeline

- 2026-11-13T16:16:48Z · start · intent → intent · The cutoff check probes the archive root with a literal pathspec, so a project folder named like pathspec magic is not reported unreadable (#990) ^t-e55af1
  - revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:06Z · advance · intent → design ·  ^t-112c2f
  - native_change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
  - revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:06Z · skip · design → plan · Internal Git read path only: no visuals or interactions ^t-513e31
  - native_change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
  - revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:08Z · advance · plan → build · native:verifying ^t-0150d0
  - native_change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
  - gate: native:verifying
  - revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:08Z · advance · build → test · native:verifying ^t-d338c0
  - native_change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
  - gate: native:verifying
  - revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
  - by: claude-code (hc t38p)
- 2026-11-13T16:58:19Z · advance · test → accept · native:verifying ^t-39050a
  - native_change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
  - gate: native:verifying
  - revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
  - by: claude-code (hc t38p)
- 2026-11-13T16:58:19Z · gate-opened · accept → accept · Implementation review of the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path ^t-c0ee83
  - native_change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
  - revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
