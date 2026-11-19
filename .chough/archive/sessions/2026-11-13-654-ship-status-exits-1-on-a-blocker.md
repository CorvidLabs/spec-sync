---
kind: session
id: 2026-11-13-654-ship-status-exits-1-on-a-blocker
aliases: [654-ship-status-exits-1-on-a-blocker]
objective: change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)
modules: [cmd_change]
stage: done
stage_status: archived
iteration: 1
native_change: change-ship-status-exits-1-when-it-reports-a-blocker
criteria: [SHIP-9.a]
files:
  - src/commands/change.rs
  - tests/integration/change.rs
  - site/src/content/docs/cli.md
  - docs/ADOPTING.md
  - CHANGELOG.md
observations:
  - 2026-11-13-at-81877688e-654-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
  - 2026-11-19-red-first-control-on-main-3bebe8b0-with-only-the-new-tests
  - 2026-11-19-654-verify-lane-at-4d439b39
  - 2026-11-19-654-verify-lane-at-612d255d
  - 2026-11-19-654-verify-lane-at-5f5cdf8d
  - 2026-11-19-654-verify-lane-at-9564c7a3
closed: 2026-11-19
outcome: done
---
# change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Goal

change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 on branch fix/ship-status-654-1038-937 with #1038 and #937: merged the 11-13 branch, kept main's cmd_change spec files, and let change check apply REQ-cmd-change-039 and write specs/cmd_change/history/change-ship-status-exits-1-when-it-reports-a-blocker.md instead of the old version bump and Change Log row; added the CHANGELOG line; red control on main and the verify lane recorded
- PR #1093 opened; re-checked at 612d255d after the review fixes for #1038, with the verify lane recorded there

## Decisions and why

- decided: authorization is Leif's own decision ('Exit 1 on a blocker (Recommended)', option text 'So scripts and CI can act on it.', coordinator chat 2026-11-13T14:12Z, relayed by claude-code-t370 and passed on by claude-code-t38c); REQ-cmd-change-039 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it
- decided: the approved REQ-cmd-change-039 text is unchanged and the 11-13 approval stands because Leif said 'Yes, they stand' (relayed by his planning session hc t4c through ss-lead hc t58, 2026-11-18)
- decided: one review answer from Leif on the batch PR counts as the scoped review of each change in it because he said 'Yes, one answer per batch' (relayed by hc t4c through ss-lead hc t58, 2026-11-18); nothing is recorded until it comes
- 0xLeif decided "Implementation review of change-ship-status-exits-1-when-it-reports-a-blocker": "all as recommended, builder runs the send-back" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:32Z, on ss-lead/gates/1093-review-request.md at PR #1093 head 762adcaf, one answer for the three changes of the PR; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change review change-ship-status-exits-1-when-it-reports-a-blocker as 0xLeif with verdict pass

## What was verified

- At 81877688e (#654 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039; chough cannot resolve cmd_change requirement ids, so the id is named here). — 194974f47b87 ^2026-11-13-at-81877688e-654-with-origin-main-bb7682a78-merged-in-fledge
- Red-first control: on main 18385ef5e with only the new test, change::ship_status_exits_one_when_it_reports_a_blocker fails: ship-status printed its Blocker line and exited 0. The failure is the defect #654 reports, fixed by this change; not a new bug. — e59482574dfd ^2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
- 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
- Red-first control: on main 3bebe8b0 with only the new tests, change::ship_status_exits_one_when_it_reports_a_blocker fails (ship-status printed its Blocker line and exited 0), and the three existing tests that now expect exit 1 from a blocked ship-status fail the same way. The failure is the defect #654 reports, fixed by this change; not a new bug. — ed377209c613 ^2026-11-19-red-first-control-on-main-3bebe8b0-with-only-the-new-tests
- At 4d439b39 (#654, #1038 and #937 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m07s: fmt, lint and check-types, cargo test (unit 3079 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests, including change::ship_status_exits_one_when_it_reports_a_blocker and the three tests that now expect exit 1 from a blocked ship-status (REQ-cmd-change-039). The lane's processes ran with a neutral global git config: with the machine's own config, which signs tags, four changelog unit tests wait on an editor for a tag message. — e8ef6ed8b7a7 ^2026-11-19-654-verify-lane-at-4d439b39
- At 612d255d (PR #1093 after the review fixes for #1038 in c697bbde, each change re-checked there), fledge lanes run verify passed 10/10 steps in 29m34s on a busy machine: fmt, lint and check-types, cargo test (unit 3081 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039); this change's source is unchanged since 4d439b39. — 8885c3e2fa2d ^2026-11-19-654-verify-lane-at-612d255d
- At 5f5cdf8d (PR #1093 after the archive-side fix e54bc636, each change re-checked there), fledge lanes run verify passed 10/10 steps in 9m42s: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039); this change's source is unchanged since 4d439b39. — 9c02647bf5c0 ^2026-11-19-654-verify-lane-at-5f5cdf8d
- At 9564c7a3 (after the send-back, the new criterion and Leif's relayed re-approval of #1038's change, with all three changes re-checked), fledge lanes run verify passed 10/10 steps in 11m60s, run alone: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092) — 890731c11cad ^2026-11-19-654-verify-lane-at-9564c7a3

## Bugs

None.

## Open threads

## Timeline

- 2026-11-13T21:52:39Z · start · intent → intent · change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654) ^t-f6681b
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:41Z · advance · intent → design ·  ^t-cdcc2d
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:41Z · skip · design → plan · CLI behaviour only: no visuals or interactions ^t-4cd0d8
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:43Z · advance · plan → build · native:verifying ^t-527d9f
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - gate: native:verifying
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:43Z · advance · build → test · native:verifying ^t-d862de
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - gate: native:verifying
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T22:39:05Z · advance · test → accept · native:verifying ^t-c8ebc6
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - gate: native:verifying
  - revision: 81877688e5eb0e3749d3db30e55fa7f0d1347302
  - by: claude-code (hc t38p)
- 2026-11-13T22:39:05Z · gate-opened · accept → accept · Implementation review of change-ship-status-exits-1-when-it-reports-a-blocker ^t-671531
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - revision: 81877688e5eb0e3749d3db30e55fa7f0d1347302
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
- 2026-11-19T14:37:59Z · gate-closed · accept → accept · Implementation review of change-ship-status-exits-1-when-it-reports-a-blocker ^t-8c484d
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - revision: 079a56233dbd982044c64720dc4a563041942b89
  - relayed_by:
    - decided_by: 0xLeif
    - words: all as recommended, builder runs the send-back
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 14:32Z, on ss-lead/gates/1093-review-request.md at PR #1093 head 762adcaf, one answer for the three changes of the PR; relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change review change-ship-status-exits-1-when-it-reports-a-blocker
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-19T14:38:04Z · advance · accept → done · native:archived ^t-5869bd
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - gate: native:archived
  - revision: 079a56233dbd982044c64720dc4a563041942b89
  - by: claude-code (hc t59)

Closed 2026-11-19: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-13-at-81877688e-654-with-origin-main-bb7682a78-merged-in-fledge
    claim: "At 81877688e (#654 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039; chough cannot resolve cmd_change requirement ids, so the id is named here)."
    result: passed
    revision: 81877688e5eb0e3749d3db30e55fa7f0d1347302
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: 66f48f900b24f56ef8e63696fbd06379614040e7d87d9be76423f81f3339dfbe
    fingerprint: sha256:194974f47b87b073d0cbf8624f57446e93338260de2b9e01581bc931f213e895
    observed: 2026-11-13T22:39:01Z
    actor: claude-code (hc t38p)
  - id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
    claim: "Red-first control: on main 18385ef5e with only the new test, change::ship_status_exits_one_when_it_reports_a_blocker fails: ship-status printed its Blocker line and exited 0. The failure is the defect #654 reports, fixed by this change; not a new bug."
    result: failed
    revision: 81877688e5eb0e3749d3db30e55fa7f0d1347302
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: 66f48f900b24f56ef8e63696fbd06379614040e7d87d9be76423f81f3339dfbe
    fingerprint: sha256:e59482574dfd6474faf45cc03d6d339098e08a67919e7447ac4ed230cc14a2e7
    observed: 2026-11-13T22:39:02Z
    actor: claude-code (hc t38p)
  - id: 2026-11-19-red-first-control-on-main-3bebe8b0-with-only-the-new-tests
    claim: "Red-first control: on main 3bebe8b0 with only the new tests, change::ship_status_exits_one_when_it_reports_a_blocker fails (ship-status printed its Blocker line and exited 0), and the three existing tests that now expect exit 1 from a blocked ship-status fail the same way. The failure is the defect #654 reports, fixed by this change; not a new bug."
    result: failed
    revision: 4d439b395c78eb20987f11815fd46ee859bde087
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: 1b34863182f9a288aa5a2899fb3dad4140972adb3abe0d9516b44c695af88683
    fingerprint: sha256:ed377209c613487d1e02dfb5d820168fad6abee797c88d85415adca933101b26
    observed: 2026-11-19T04:55:17Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-654-verify-lane-at-4d439b39
    claim: "At 4d439b39 (#654, #1038 and #937 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m07s: fmt, lint and check-types, cargo test (unit 3079 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests, including change::ship_status_exits_one_when_it_reports_a_blocker and the three tests that now expect exit 1 from a blocked ship-status (REQ-cmd-change-039). The lane's processes ran with a neutral global git config: with the machine's own config, which signs tags, four changelog unit tests wait on an editor for a tag message."
    result: passed
    revision: 4d439b395c78eb20987f11815fd46ee859bde087
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: 1b34863182f9a288aa5a2899fb3dad4140972adb3abe0d9516b44c695af88683
    fingerprint: sha256:e8ef6ed8b7a702cb6e37b4c11a19c9843b99852b1989eba8c8335a8bc3c63fc7
    observed: 2026-11-19T04:55:32Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-654-verify-lane-at-612d255d
    claim: "At 612d255d (PR #1093 after the review fixes for #1038 in c697bbde, each change re-checked there), fledge lanes run verify passed 10/10 steps in 29m34s on a busy machine: fmt, lint and check-types, cargo test (unit 3081 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039); this change's source is unchanged since 4d439b39."
    result: passed
    revision: 612d255dc6c463fdabdb7fe329f74dcb78de5824
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: d6b777e99a4073e1fc42bb05499eea290730b7e3889e0a0d0ac54d84e020b7d6
    fingerprint: sha256:8885c3e2fa2d07ac511a9853adb64e574aa3909dfe8e0161defe13476995bd1b
    observed: 2026-11-19T05:56:46Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-654-verify-lane-at-5f5cdf8d
    claim: "At 5f5cdf8d (PR #1093 after the archive-side fix e54bc636, each change re-checked there), fledge lanes run verify passed 10/10 steps in 9m42s: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039); this change's source is unchanged since 4d439b39."
    result: passed
    revision: 5f5cdf8d41eb8ca4e8f3c3b229e8c5f24f10c5c7
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: bf638a95bab9091bab5a1880c42b353e868c47abad1ea4614ebc0ac89c46416e
    fingerprint: sha256:9c02647bf5c0ab235eddbf4832713a82e16bc50fc24ceaca983648aaf36cd305
    observed: 2026-11-19T06:20:44Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-654-verify-lane-at-9564c7a3
    claim: "At 9564c7a3 (after the send-back, the new criterion and Leif's relayed re-approval of #1038's change, with all three changes re-checked), fledge lanes run verify passed 10/10 steps in 11m60s, run alone: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092)"
    result: passed
    revision: 9564c7a3e165e3b6cf44e3a8f3125b6c39f97725
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: d5ab9d38b8f4a4e03d340c239338f5cc5e0a1f048f0bbbef37ce5e32e5ebc8ad
    fingerprint: sha256:890731c11cadfdc8f7fd6085d95ed1623baf5b6b11a77fb05ea741927832c7b8
    observed: 2026-11-19T14:21:37Z
    actor: claude-code (hc t59)
```
