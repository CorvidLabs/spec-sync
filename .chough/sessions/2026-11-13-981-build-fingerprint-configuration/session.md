---
kind: session
id: 2026-11-13-981-build-fingerprint-configuration
aliases: [981-build-fingerprint-configuration]
objective: The build fingerprint frames the compile configuration as well as the sources, so each build keeps its own read cache (#981)
intent_scope: ""
modules: [hash_cache]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
pr: ""
criteria: [SPEED-1.c]
requirements: []
cases: []
design: []
files:
  - build.rs
  - src/build_fingerprint.rs
  - src/hash_cache.rs
  - src/main.rs
  - specs/hash_cache/hash_cache.spec.md
observations:
  - 2026-11-13-at-f4f8a0e75-fix-981-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
    holds: accept
    drives: review
    opened: 2026-11-13T20:54:52Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# The build fingerprint frames the compile configuration as well as the sources, so each build keeps its own read cache (#981)

## Goal

The build fingerprint frames the compile configuration as well as the sources, so each build keeps its own read cache (#981)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#981 cache per build', coordinator chat 2026-11-13T04:55Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At f4f8a0e75 (fix/981 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3073 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration.](evidence/2026-11-13-at-f4f8a0e75-fix-981-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration fails: the fingerprint covered the sources but not the build configuration. The failure is the defect #981 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

## Open threads

## Timeline

- 2026-11-13T20:09:04Z · start · intent → intent · The build fingerprint frames the compile configuration as well as the sources, so each build keeps its own read cache (#981) ^t-fc7416
  - revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:06Z · advance · intent → design ·  ^t-7fb4b4
  - native_change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
  - revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:06Z · skip · design → plan · Build, cache and CLI behaviour only: no visuals or interactions ^t-fa4c45
  - native_change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
  - revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:08Z · advance · plan → build · native:verifying ^t-3a2da8
  - native_change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
  - gate: native:verifying
  - revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:08Z · advance · build → test · native:verifying ^t-12949a
  - native_change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
  - gate: native:verifying
  - revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
  - by: claude-code (hc t38p)
- 2026-11-13T20:54:52Z · advance · test → accept · native:verifying ^t-1daec1
  - native_change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
  - gate: native:verifying
  - revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
  - by: claude-code (hc t38p)
- 2026-11-13T20:54:52Z · gate-opened · accept → accept · Implementation review of the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources ^t-e22ed9
  - native_change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
  - revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
