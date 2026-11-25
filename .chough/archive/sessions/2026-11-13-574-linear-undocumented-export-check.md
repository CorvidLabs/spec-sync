---
kind: session
id: 2026-11-13-574-linear-undocumented-export-check
aliases: [574-linear-undocumented-export-check]
objective: check names each undocumented export's file from one lookup, so its time grows linearly (#574)
modules: [validator]
stage: done
stage_status: archived
iteration: 1
native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
criteria: [SPEED-1.b]
files: [src/validator/consumed_by_re.rs, tests/integration/check_scaling.rs]
observations:
  - 2026-11-13-at-2ef24f40e-574-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
  - 2026-11-19-574-red-control-on-main-3bebe8b0
  - 2026-11-19-574-verify-lane-at-c2b19a91
  - 2026-11-19-574-verify-lane-at-76a66032
  - 2026-11-19-574-red-on-main-release
  - 2026-11-19-574-red-on-main-debug
  - 2026-11-19-574-green-at-the-tip-release
  - 2026-11-19-574-green-at-the-tip-debug
  - 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
closed: 2026-11-24
outcome: done
---
# check names each undocumented export's file from one lookup, so its time grows linearly (#574)

## Goal

check names each undocumented export's file from one lookup, so its time grows linearly (#574)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 with the other four; one conflict in tests/integration.rs, both test modules kept; re-checked; red control on main and the verify lane recorded
- The lead's and the tester's review points on f4de84fc are in (f52d7f3c, 6ffa715e): #574's timing test is interleaved and resized so it is red on main in release and debug; #1050's and #1060's CHANGELOG entries say what an upgrade does and what is left out; re-checked; lane 10/10 at 76a66032

## Decisions and why

- decided: authorization is Leif's own scope approval ('#574 fast check', coordinator chat 2026-11-13T14:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it
- 0xLeif decided "Implementation review of check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly": "I accept all four" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request headed 'Decision for you: do you accept each implementation?', which listed the four changes of the validation batch (#1071, #1076, #574 and #1060) with their native Verify lane at cb679a13, Trust, scoped checks and CLI dogfood against the 6.4.2 release binary. This change is the #574 one of those four.); recorded by claude-code-t14v; a relayed claim, not authenticated; the scoped review of check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly was already current, so no review ran

## What was verified

- At 2ef24f40e (574 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including check_scaling::check_time_grows_linearly_with_undocumented_exports. - 4f67a305861f ^2026-11-13-at-2ef24f40e-574-with-origin-main-bb7682a78-merged-in-fledge
- Red-first control: on main 18385ef5e with only the new test, check_scaling::check_time_grows_linearly_with_undocumented_exports fails. On main 8,000 undocumented exports took 0.55 s and 32,000 took 4.9 s, ratio 9.0, over the linear bound of 7. The failure is the defect #574 reports, fixed by this change; not a new bug. - 1b8b0888dcf6 ^2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
- Red-first control: on main 3bebe8b0 with only the new tests, check_scaling::check_time_grows_linearly_with_undocumented_exports fails: four times the undocumented exports took 9.3 times as long (0.84s against 7.83s on a busy machine); linear would be about 4. This is the defect #574 reports, fixed by this change; not a new bug. - ced0b3127d12 ^2026-11-19-574-red-control-on-main-3bebe8b0
- At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes check_scaling::check_time_grows_linearly_with_undocumented_exports. - 7080ba2d1534 ^2026-11-19-574-verify-lane-at-c2b19a91
- At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092) - 244325edaca8 ^2026-11-19-574-verify-lane-at-76a66032
- Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, release build: 14.3 times as long (0.56s, 7.95s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug. The first sizes, 8,000 against 32,000, were red in debug only (the tester measured 5.2 times in release). - 34aed61710ac ^2026-11-19-574-red-on-main-release
- Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, debug build: 14.9 times as long (7.06s, 104.9s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug. - 805e38d8683e ^2026-11-19-574-red-on-main-debug
- The resized timing test at the batch 3 tip, idle machine, release build: 3.0 times as long (0.12s, 0.35s), under the bound of 7. - bcb29f7df80c ^2026-11-19-574-green-at-the-tip-release
- The resized timing test at the batch 3 tip, idle machine, debug build: 3.8 times as long (0.65s, 2.45s), under the bound of 7. - d7b754223534 ^2026-11-19-574-green-at-the-tip-debug
- 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
- At cb679a13 (#1071, #1076, #1060 and #574 merged onto main a275ab44, the 6.4.2 release candidate commit, each change re-checked there), fledge lanes run verify passed 10/10 steps in 34m20s: fmt, lint and check-types, cargo test (unit 3357 passed, 2 ignored; integration 825 passed), the release build, the forced strict spec check at 100 percent coverage, and the six CI contract suites. On real CLI paths with the release binary that lane built (sha256 5c4426e0), against the 6.4.2 release binary, best of three runs of check --force on one module: 8,000 undocumented exports take 0.048 s and 32,000 take 0.089 s (1.85 times for four times the exports); 6.4.2 takes 0.086 s and 0.566 s (6.58 times). Every export is still named in the output. - 4f138e01c8d7 ^2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44

## Bugs

None.

## Open threads

## Timeline

- 2026-11-13T20:56:25Z · start · intent → intent · check names each undocumented export's file from one lookup, so its time grows linearly (#574) ^t-b1c6d3
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:29Z · advance · intent → design ·  ^t-004196
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:29Z · skip · design → plan · Read-path performance only: no visuals or interactions ^t-3f8943
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:33Z · advance · plan → build · native:verifying ^t-d67c28
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:33Z · advance · build → test · native:verifying ^t-2f65c1
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T21:51:56Z · advance · test → accept · native:verifying ^t-2de051
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T21:51:56Z · gate-opened · accept → accept · Implementation review of check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly ^t-0855bf
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
- 2026-11-19T07:23:20Z · back · accept → build · ss-lead's review of the batch 3 branch at f4de84fc: make the timing test harder for machine load to flip (interleaved runs, fastest of three), same 4x sizes and same bound ^t-539258
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: f4de84fc72a02214003261bb9e01648dbd9916c6
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 4 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T08:41:59Z · advance · build → test · native:verifying ^t-26ae16
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 76a6603257a1efec6ba25d83c603617ce7434c46
  - by: claude-code (hc t59)
- 2026-11-19T08:42:02Z · advance · test → accept · native:verifying ^t-2eca65
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 76a6603257a1efec6ba25d83c603617ce7434c46
  - by: claude-code (hc t59)
- 2026-11-24T21:02:01Z · direct-native-write · accept → accept · check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly's lifecycle records changed outside Chough: review.json added, review-attempts.json added; state stays verifying ^t-7649ed
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: c7c0f5f54d863fe09fdf8ac42c9032b40ce2c65b
  - direct_write:
    - change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
    - files:
      - review.json: added
      - review-attempts.json: added
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t14v)
- 2026-11-24T21:02:01Z · gate-closed · accept → accept · Implementation review of check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly ^t-3c058e
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: c7c0f5f54d863fe09fdf8ac42c9032b40ce2c65b
  - relayed_by:
    - decided_by: 0xLeif
    - words: I accept all four
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request headed 'Decision for you: do you accept each implementation?', which listed the four changes of the validation batch (#1071, #1076, #574 and #1060) with their native Verify lane at cb679a13, Trust, scoped checks and CLI dogfood against the 6.4.2 release binary. This change is the #574 one of those four.
    - recorded_by: claude-code-t14v
  - by: claude-code (hc t14v)
- 2026-11-24T21:02:03Z · advance · accept → done · native:archived ^t-07ea77
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:archived
  - revision: c7c0f5f54d863fe09fdf8ac42c9032b40ce2c65b
  - by: claude-code (hc t14v)

Closed 2026-11-24: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-13-at-2ef24f40e-574-with-origin-main-bb7682a78-merged-in-fledge
    claim: "At 2ef24f40e (574 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including check_scaling::check_time_grows_linearly_with_undocumented_exports."
    result: passed
    revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: 403b9ac0ba3a3c372edbc958342ac51185669a5fab2d0d2883ac4b8391d544dc
    fingerprint: sha256:4f67a305861f1b0b55da75174f129bf59146b6c11e7b17aeba6b19899d0d3491
    criteria: SPEED-1.b
    observed: 2026-11-13T21:51:53Z
    actor: claude-code (hc t38p)
  - id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
    claim: "Red-first control: on main 18385ef5e with only the new test, check_scaling::check_time_grows_linearly_with_undocumented_exports fails. On main 8,000 undocumented exports took 0.55 s and 32,000 took 4.9 s, ratio 9.0, over the linear bound of 7. The failure is the defect #574 reports, fixed by this change; not a new bug."
    result: failed
    revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: 403b9ac0ba3a3c372edbc958342ac51185669a5fab2d0d2883ac4b8391d544dc
    fingerprint: sha256:1b8b0888dcf6c22c142d40d96daba4a2db89ecbed36e9f2d1734c12c5bc335e5
    criteria: SPEED-1.b
    observed: 2026-11-13T21:51:54Z
    actor: claude-code (hc t38p)
  - id: 2026-11-19-574-red-control-on-main-3bebe8b0
    claim: "Red-first control: on main 3bebe8b0 with only the new tests, check_scaling::check_time_grows_linearly_with_undocumented_exports fails: four times the undocumented exports took 9.3 times as long (0.84s against 7.83s on a busy machine); linear would be about 4. This is the defect #574 reports, fixed by this change; not a new bug."
    result: failed
    revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: d51119b1914763f95f18c5a96371c2c554353568b4863d3b69f2f0234962b4d8
    fingerprint: sha256:ced0b3127d128007a5e43cf2d61e35b53e2f1230dd7a8ef05d4e0d6907a742ef
    criteria: SPEED-1.b
    observed: 2026-11-19T07:14:12Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-574-verify-lane-at-c2b19a91
    claim: "At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes check_scaling::check_time_grows_linearly_with_undocumented_exports."
    result: passed
    revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: d51119b1914763f95f18c5a96371c2c554353568b4863d3b69f2f0234962b4d8
    fingerprint: sha256:7080ba2d15341fbedda3803c248adfb3483329540dcdd1b6dedc63bf9649e810
    criteria: SPEED-1.b
    observed: 2026-11-19T07:14:13Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-574-verify-lane-at-76a66032
    claim: "At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092)"
    result: passed
    revision: 76a6603257a1efec6ba25d83c603617ce7434c46
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
    fingerprint: sha256:244325edaca85ce1dc0b4afe3c4feda6ea8378863e0a1c5dbf8ef4de18476554
    criteria: SPEED-1.b
    observed: 2026-11-19T08:41:54Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-574-red-on-main-debug
    claim: "Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, debug build: 14.9 times as long (7.06s, 104.9s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug."
    result: failed
    revision: 76a6603257a1efec6ba25d83c603617ce7434c46
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
    fingerprint: sha256:805e38d8683e6d48229b1a14f3bd89bcb0a73e4c6bd39319033c302531d38b07
    criteria: SPEED-1.b
    observed: 2026-11-19T08:42:55Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-574-red-on-main-release
    claim: "Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, release build: 14.3 times as long (0.56s, 7.95s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug. The first sizes, 8,000 against 32,000, were red in debug only (the tester measured 5.2 times in release)."
    result: failed
    revision: 76a6603257a1efec6ba25d83c603617ce7434c46
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
    fingerprint: sha256:34aed61710ac0ed18db68ec11bcd056f7cc2dc2d6ef2cb0c42ceaf8e2181d4b6
    criteria: SPEED-1.b
    observed: 2026-11-19T08:42:55Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-574-green-at-the-tip-debug
    claim: "The resized timing test at the batch 3 tip, idle machine, debug build: 3.8 times as long (0.65s, 2.45s), under the bound of 7."
    result: passed
    revision: 76a6603257a1efec6ba25d83c603617ce7434c46
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
    fingerprint: sha256:d7b754223534bfb540c557ef3a6e279761193ae38473dbd54e065c3bbb4c4e58
    criteria: SPEED-1.b
    observed: 2026-11-19T08:42:56Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-574-green-at-the-tip-release
    claim: "The resized timing test at the batch 3 tip, idle machine, release build: 3.0 times as long (0.12s, 0.35s), under the bound of 7."
    result: passed
    revision: 76a6603257a1efec6ba25d83c603617ce7434c46
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
    fingerprint: sha256:bcb29f7df80ceb822f461ade09ee2e686f457b534c2f6e037096645a95a9c4a4
    criteria: SPEED-1.b
    observed: 2026-11-19T08:42:56Z
    actor: claude-code (hc t59)
  - id: 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
    claim: "At cb679a13 (#1071, #1076, #1060 and #574 merged onto main a275ab44, the 6.4.2 release candidate commit, each change re-checked there), fledge lanes run verify passed 10/10 steps in 34m20s: fmt, lint and check-types, cargo test (unit 3357 passed, 2 ignored; integration 825 passed), the release build, the forced strict spec check at 100 percent coverage, and the six CI contract suites. On real CLI paths with the release binary that lane built (sha256 5c4426e0), against the 6.4.2 release binary, best of three runs of check --force on one module: 8,000 undocumented exports take 0.048 s and 32,000 take 0.089 s (1.85 times for four times the exports); 6.4.2 takes 0.086 s and 0.566 s (6.58 times). Every export is still named in the output."
    result: passed
    revision: cb679a13f1433afbfc0989666a234d05ae822f66
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads
    content: a309099a545c98c834a7d5a3bf4a0e3c684e607bb7325f90d97bed3d80c78d19
    fingerprint: sha256:4f138e01c8d72be3d4e938e9ffc65bee1cabd32f39df9b6c22c17b2470e05870
    criteria: SPEED-1.b
    observed: 2026-11-24T20:49:25Z
    actor: claude-code (hc t14v)
```
