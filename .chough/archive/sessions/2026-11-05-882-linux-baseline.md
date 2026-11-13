---
kind: session
id: 2026-11-05-882-linux-baseline
aliases: [882-linux-baseline]
objective: Linux release binaries start on a documented glibc baseline (spec-sync#882, audit D6, plan row S7)
intent_scope: ""
modules: []
stage: done
stage_status: active
iteration: 1
callers: []
native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
pr: ""
criteria: [INSTALL-1, INSTALL-1.a, INSTALL-1.b, INSTALL-1.c]
requirements: [REQ-github-025]
cases:
  - TC-github-linux-baseline-lockstep
  - TC-github-linux-arm64-glibc-download-starts-on-baseline
  - TC-github-linux-arm64-musl-download-starts-anywhere
design: []
files: []
observations:
  - OBS-tc-github-linux-baseline-lockstep-8d3fbcb70a07759eebbe4095
  - OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-c9cfc037fd1400b5587015c3
  - OBS-tc-github-linux-arm64-musl-download-starts-anywhere-4accc748063adeedf526da3e
  - OBS-tc-github-linux-baseline-lockstep-0e1525ce22afcbc2fa4ac991
  - OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-b49ab60d64c36c856935f453
  - OBS-tc-github-linux-arm64-musl-download-starts-anywhere-f8d8bed59851d38bba47323b
  - OBS-product-test-199fa922a987064b62f203cb
  - OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-e830c9d0f1421775e2356bdb
  - OBS-tc-github-linux-arm64-musl-download-starts-anywhere-515683671907229ea5e79126
  - OBS-tc-github-linux-baseline-lockstep-a0a16bc0ce1d0ef7aa9439d9
bugs:
  - BUG-linux-arm64-release-download-does-not-start-on-debian-12-or-ubuntu-22-0-23eb0fe0f3eb7815d6dc3284
blockers: []
waiting_on_human: []
run_consents: []
next: "Push the fix, wait for CI, send the fix for review (changes reviewed content); merge waits for Leif (overnight: orc cannot act in his name)"
closed: 2026-11-06
outcome: done
---
# Linux release binaries start on a documented glibc baseline (spec-sync#882, audit D6, plan row S7)

## Goal

Linux release binaries start on a documented glibc baseline (spec-sync#882, audit D6, plan row S7)

## Where it stands

- Profiled locally (macOS, 20 cores, nextest 0.9.151, main 67aa7dc2): all 3242 tests pass in 811s wall vs ~2000s with cargo test; sys 4364s > user 2398s
- 152 of 3242 tests carry 92% of test time; the slowest (integration eight_workflow_v2_changes_finalize...) takes 343s alone and spawns over 13,000 git processes (929 rev-parse, 186 ls-files, 164 show, 149 rev-list)
- CI rc.2 macos-15 qualify: test build 4m, unit tests 2025s, integration 338s, release build 5m (ubuntu: unit 214s, integration 58s)
- Native definition approval recorded by 0xLeif (via orc t18) at a22d6240
- Repository-fact memo (git_utils) scoped to CLI change commands, project lock and read scopes; invalidated by git writes, verification commands, pre-push gate and gh; live-read guard for race checks
- Batched cat-file --batch reader for show <rev>:<path> blobs and single-path ls-tree; equality tests incl. binary/empty/missing
- Spawn ceilings per command pass: new 13, answer 15, approve 29, check 82, status 50, review 128, finalize 239 (baseline 63/71/89/388/66/323/739); slowest test 5,535 spawns vs 15,563 (35.6%) before the batch reader
- test-nextest task, release-candidate lane, pinned cargo-nextest 0.9.151 install in release.yml (published sha256 verified)
- change check --commit verified (575ddcd9); fledge verify steps pass on pinned 1.89; pre-push green
- Shipped: archived at 2bc2ba92
- Captured approved INSTALL-1, 1.a, 1.b, 1.c in hi/install.md (Leif's approval relayed by orc t18, 20:35 MST)
- Failing test first: .github/scripts/linux-baseline.py + test-linux-baseline.py written; red recorded in the change testing.md (v6.2.0 ARM64 rejected with GLIBC_2.39 on ubuntu:22.04 and debian:12; 15 structural failures on the unmodified tree); positive control passes on new-recipe ARM64 archives
- Defined native change build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose (operations, spec github, REQ-github-025 delta, 11 acceptance criteria, all artifacts complete)
- Failing tests: d90cbc8292c6a5c07e3ca34155628eaeb9a8b2a6
- Implemented #882 within the widened approved scope; merged origin/main 560af9f1 (CHANGELOG union)
- Green: three cases pass on new-recipe ARM64 archives built from merged tree 4547d9c0; bug verified; verify lane passed (OBS-product-test-199fa922a987064b62f203cb); change check materialized REQ-github-025
- Shipped: change finalized and archived at 815c1f43 (2026-11-06-build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose)
- CI green on archive tip 558a8144 (27 pass, attestation skipped as designed), including all four linux-baseline jobs and the v6.2.0 negative control on the hosted runners
- Fixed Codex P2 on #910 (truncated smoke run now reports the container exit, the last step and its output) with a failing-first test; cases rerun green

## Decisions and why

- Agent finding, not a human decision: opt-level 2 test build did not speed the slowest test (386s vs 343s), so an optimized test profile is not the lever; git process spawning is
- Leif (2026-11-05 09:21 MST, Herdr orc tab, relayed by orc t18): approved intent SPEED-2.b.1 and SPEED-2.b.2 and the approach: cut SpecSync git process spawns plus cargo nextest in the qualify lane
- Rejected: optimized test profile as the main lever (opt-level 2 measured 386s vs 343s on the slowest test); larger runner (not needed if spawns drop)
- Leif (2026-11-05 09:35 MST, orc tab): approved the four-part scope; qualify timeout returns to 45 in a one-line follow-up after the next RC's macOS qualify measures well under 45, not in this change
- Leif (2026-11-05 18:23 MST, orc tab, relayed by orc t18): "Record and merge" for PR #869; review recorded as 0xLeif (pass) at 113893d8; acceptance granted
- Agent finding, not a human decision: v6.2.0 Linux gnu binaries (aarch64 and x86_64) need GLIBC_2.39 only through weak pidfd_spawnp/pidfd_getpid bound on ubuntu-latest 24.04; the same source built on Ubuntu 22.04 needs at most GLIBC_2.34 and passes version/help/check on ubuntu:22.04 and debian:12
- Leif (2026-11-05 20:35 MST, orc tab, relayed by orc t18): "Both arches, glibc 2.35, + musl": intent INSTALL-1/1.a/1.b/1.c as drafted; fix x86_64 and ARM64 gnu builds on Ubuntu 22.04 (floor glibc 2.35 = Ubuntu 22.04 / Debian 12+); add specsync-linux-aarch64-musl as the fallback; a release whose Linux download wouldn't start on the floor is stopped (CI runs the built binaries in ubuntu:22.04 and debian:12). No tag or publish.
- rejected: ARM64-only scope and documenting cargo install as the only ARM64 fallback because Leif chose both arches plus an aarch64 musl download
- decided: record the release baseline as REQ-github-024 in the github spec because the release-matrix contract has lived there since CHG-0075 and the Windows drop, and Chough evidence needs a native contract
- rejected: a no-spec change like CHG-0048 because the github spec owns the release distribution contract
- decided: renumber the release-baseline requirement from REQ-github-024 to REQ-github-025 because spec-sync#886 (branch fix/886-comment-readonly-token, approved) already holds 024 (orc t18); scope unchanged
- Leif (2026-11-05, orc tab, relayed by orc t18): 'Approve' — scope approval of build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose at 83458275, recorded natively by the orc as 0xLeif; instruction: build, verify, PR (Fixes #882), 'ready for review'; no tag or publish
- Leif (2026-11-05 21:28 MST, orc tab, relayed by orc t18): "Yes, widen" #882 to add .github/scripts/test-required-ci-gate.py and site/src/content/docs/quickstart.md; send-back recorded by the agent as 'claude (ss-linux-arm64)' quoting this decision, never 0xLeif; Leif's re-approval to be recorded by the orc
- Leif (2026-11-05, orc tab, relayed by orc t18): re-approval of the widened build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose recorded natively by the orc at 6abc70e9 ('Yes, widen'); instruction: build, verify, PR, 'ready for review'
- orc t18 (2026-11-05 23:3x MST), deciding as Leif under his overnight authorization ('decide as him where 95% confident'): review recorded natively as 0xLeif (pass) at ebbbfb25; instruction: ship, push the archive tip, merge pinned on green including the new linux-baseline job; no tag or publish

## What was verified

- Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: failed (exit 1). — 860234f21921 ^2026-11-06-tc-github-linux-baseline-lockstep
- Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: failed (exit 1). — 3cdbbaec56f5 ^2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline
- Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: failed (exit 2). — 9652fecf70fb ^2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere
- Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: passed (exit 0). — 28ef48376243 ^2026-11-06-tc-github-linux-baseline-lockstep-4afe7b
- Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: passed (exit 0). — d31f3335566b ^2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline-fea482
- Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: passed (exit 0). — 89f47822b12c ^2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere-65ac20
- PATH=$HOME/.cargo/bin:$PATH fledge lanes run verify at 4547d9c0118b6a463a2344aa08ed0c17f580fb32 (merged with origin/main 560af9f1; cargo 1.89.0): 7 steps passed in 35m23s: fmt, clippy -D warnings, cargo check, cargo test (2823 + 483 passed, 2 ignored as on main, 0 failed), release build, spec-check 62/62 at 100% coverage, release-candidate-test, ci-gate-test and linux-baseline-test (34 tests). ^2026-11-06-path-home-cargo-bin-path-fledge-lanes-run-verify-at
- Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: passed (exit 0). — d31f3335566b ^2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline-b7f0e4
- Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: passed (exit 0). — 89f47822b12c ^2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere-a7cf42
- Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: passed (exit 0). — 177be7a28663 ^2026-11-06-tc-github-linux-baseline-lockstep-84c264

## Open threads

## Timeline

- 2026-11-05T16:20:45Z · gate-opened · intent → intent · Intent approval for #861 speed work (criteria drafted, not captured in hi yet) ^t-2a1839
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:21:39Z · gate-closed · intent → intent · Intent approval for #861 speed work (criteria drafted, not captured in hi yet) ^t-c8521e
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:22:17Z · advance · intent → design ·  ^t-7f2cb6
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:22:17Z · skip · design → plan · CLI and CI change; no visuals or interactions ^t-434a77
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:33:57Z · gate-opened · plan → plan · Scope approval of cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos (specsync change approve --actor 0xLeif), including whether the qualify timeout returns to 45 in this change or in a follow-up after a measured candidate run ^t-5951b7
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:37:53Z · gate-closed · plan → plan · Scope approval of cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos (specsync change approve --actor 0xLeif), including whether the qualify timeout returns to 45 in this change or in a follow-up after a measured candidate run ^t-7ee015
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: a22d6240cbce18e4807840bfd89d9ec81a06a339
- 2026-11-05T16:37:54Z · advance · plan → build · native:approved ^t-56ad04
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - gate: native:approved
  - revision: a22d6240cbce18e4807840bfd89d9ec81a06a339
- 2026-11-06T03:26:30Z · reset · build → intent · New cycle for spec-sync#882 (audit D6, plan row S7): #861's change is archived; Linux ARM64 release binary does not start on Debian 12 ^t-bb389a
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
  - refresh: intent: HI criteria and FRD; append new criteria, never rewrite approved ones, design: canonical prototype and companion; their human approval covers the old version, plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, native: the definition is archived; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos`; Chough does not change native state.
- 2026-11-06T03:26:47Z · reset · intent → intent · New cycle for spec-sync#882 (audit D6, plan row S7): #861 change is archived ^t-1bc7ec
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
  - refresh: intent: HI criteria and FRD; append new criteria, never rewrite approved ones, native: the definition is archived; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos`; Chough does not change native state.
- 2026-11-06T03:33:21Z · gate-opened · intent → intent · Intent decision for spec-sync#882 (INSTALL-1 draft + approach, Q1 scope x86_64, Q2 floor glibc 2.35, Q3 aarch64-musl fallback) ^t-1b2858
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
- 2026-11-06T03:35:32Z · gate-closed · intent → intent · Intent decision for spec-sync#882 (INSTALL-1 draft + approach, Q1 scope x86_64, Q2 floor glibc 2.35, Q3 aarch64-musl fallback) ^t-f007a8
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
- 2026-11-06T03:36:59Z · advance · intent → design ·  ^t-fe4503
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
- 2026-11-06T03:37:00Z · skip · design → plan · CI/release workflow and docs change; no visuals or interactions ^t-ed961f
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
- 2026-11-06T03:55:54Z · gate-opened · plan → plan · Scope approval of build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose (via orc t18) ^t-3a11f5
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
- 2026-11-06T04:01:17Z · gate-closed · plan → plan · Scope approval of build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose (via orc t18) ^t-63f7a7
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
- 2026-11-06T04:01:20Z · advance · plan → build · native:approved ^t-2132fc
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - gate: native:approved
  - revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
- 2026-11-06T04:29:41Z · gate-opened · build → build · Re-approval of build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose after the widen send-back (via orc t18) ^t-66935d
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: c0d8d287bae079bbeddf954d8c5334b225170a4e
- 2026-11-06T04:44:20Z · gate-closed · build → build · Re-approval of build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose after the widen send-back (via orc t18) ^t-d31013
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 08519bdf37cd78f6d932a0e34ffb07fd75adb86b
- 2026-11-06T05:35:48Z · advance · build → test · native:verifying ^t-916c1b
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - gate: native:verifying
  - revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
- 2026-11-06T05:36:39Z · advance · test → accept · native:verifying ^t-bd00a2
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - gate: native:verifying
  - revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
- 2026-11-06T05:36:39Z · gate-opened · accept → accept · Implementation review of build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose (via orc t18, recorded natively by the orc) ^t-2776da
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
- 2026-11-06T06:36:39Z · gate-closed · accept → accept · Implementation review of build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose (via orc t18, recorded natively by the orc) ^t-b734f9
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - revision: 815c1f43e63e62b4cb2af1e7015b45d028acd20b
- 2026-11-06T14:39:40Z · advance · accept → done · native change archived by #910 (e6b1ce11) ^t-06d2bb
  - native_change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
  - gate: native:archived
  - revision: e6b1ce1162b27630445e650ac0e8e1be2bfd57b7

Closed 2026-11-06: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-06-tc-github-linux-baseline-lockstep
    aliases: OBS-tc-github-linux-baseline-lockstep-8d3fbcb70a07759eebbe4095
    claim: "Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: failed (exit 1)."
    result: failed
    command: python3 .github/scripts/test-linux-baseline.py
    revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
    environment: local-macos-arm64
    content: 977afd1b9b79ec04e6a39d27ccbd096a4f916606617a2869d2f31bb1569adf03
    fingerprint: sha256:860234f219219c7e69d7f6976d0dcdc51b7145943895691c2f1f94dba9dc2bb1
    case: TC-github-linux-baseline-lockstep
    case_digest: de0e92375a286f581f072544efb66b2b2df78cfb5dd821a50e0293aa9a788f03
    observed: 2026-11-06T04:04:17Z
    duration_seconds: 24
  - id: 2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline
    aliases: OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-c9cfc037fd1400b5587015c3
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: failed (exit 1)."
    result: failed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0
    revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
    environment: local-macos-arm64-docker
    content: 977afd1b9b79ec04e6a39d27ccbd096a4f916606617a2869d2f31bb1569adf03
    fingerprint: sha256:3cdbbaec56f5a6d0be5a952eee3e0a02a928b48772dfa45ae34d753563616608
    case: TC-github-linux-arm64-glibc-download-starts-on-baseline
    case_digest: 6c1458891f6ec19102613f5b61efd0e208d94a14c0a036f4bc247c882825e908
    observed: 2026-11-06T04:04:33Z
    duration_seconds: 8
  - id: 2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere
    aliases: OBS-tc-github-linux-arm64-musl-download-starts-anywhere-4accc748063adeedf526da3e
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: failed (exit 2)."
    result: failed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0
    revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
    environment: local-macos-arm64-docker
    content: 977afd1b9b79ec04e6a39d27ccbd096a4f916606617a2869d2f31bb1569adf03
    fingerprint: sha256:9652fecf70fb8afa26301511bb86637958fb5a77d18aa98583812bae674d8e2b
    case: TC-github-linux-arm64-musl-download-starts-anywhere
    case_digest: 9b63920d3e9575e64918c797332f155b597e92179b987c70f28ce5d1993f1ec9
    observed: 2026-11-06T04:04:39Z
    duration_seconds: 0
  - id: 2026-11-06-tc-github-linux-baseline-lockstep-4afe7b
    aliases: OBS-tc-github-linux-baseline-lockstep-0e1525ce22afcbc2fa4ac991
    claim: "Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: passed (exit 0)."
    result: passed
    command: python3 .github/scripts/test-linux-baseline.py
    revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
    environment: local-macos-arm64
    content: 40102202888fdf3bd6b11480d008fc0643523f192cfb58b692f16c3da48e70ae
    fingerprint: sha256:28ef48376243d93dc9b30cd4d800d7cd1a8abc3f9ce02b56e7c9bc1bb2241d43
    case: TC-github-linux-baseline-lockstep
    case_digest: de0e92375a286f581f072544efb66b2b2df78cfb5dd821a50e0293aa9a788f03
    observed: 2026-11-06T04:57:58Z
    duration_seconds: 5
  - id: 2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline-fea482
    aliases: OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-b49ab60d64c36c856935f453
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: passed (exit 0)."
    result: passed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0
    revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
    environment: local-macos-arm64-docker
    content: 40102202888fdf3bd6b11480d008fc0643523f192cfb58b692f16c3da48e70ae
    fingerprint: sha256:d31f3335566bcc44fbc4d10389d049ae7bb8c957e3f2ee7c56215a736b6f7ee6
    case: TC-github-linux-arm64-glibc-download-starts-on-baseline
    case_digest: 6c1458891f6ec19102613f5b61efd0e208d94a14c0a036f4bc247c882825e908
    observed: 2026-11-06T04:58:03Z
    duration_seconds: 4
  - id: 2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere-65ac20
    aliases: OBS-tc-github-linux-arm64-musl-download-starts-anywhere-f8d8bed59851d38bba47323b
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: passed (exit 0)."
    result: passed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0
    revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
    environment: local-macos-arm64-docker
    content: 40102202888fdf3bd6b11480d008fc0643523f192cfb58b692f16c3da48e70ae
    fingerprint: sha256:89f47822b12c94514c628e202d2dd9d79bf49d4eb189b95189430d69a27b635a
    case: TC-github-linux-arm64-musl-download-starts-anywhere
    case_digest: 9b63920d3e9575e64918c797332f155b597e92179b987c70f28ce5d1993f1ec9
    observed: 2026-11-06T04:58:10Z
    duration_seconds: 5
  - id: 2026-11-06-path-home-cargo-bin-path-fledge-lanes-run-verify-at
    aliases: OBS-product-test-199fa922a987064b62f203cb
    claim: "PATH=$HOME/.cargo/bin:$PATH fledge lanes run verify at 4547d9c0118b6a463a2344aa08ed0c17f580fb32 (merged with origin/main 560af9f1; cargo 1.89.0): 7 steps passed in 35m23s: fmt, clippy -D warnings, cargo check, cargo test (2823 + 483 passed, 2 ignored as on main, 0 failed), release build, spec-check 62/62 at 100% coverage, release-candidate-test, ci-gate-test and linux-baseline-test (34 tests)."
    result: passed
    revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
    environment: local-macos-arm64
    content: 4502bef3fab227583b3a20f5afe3ba74287808d5e538405a771c0cbb5fba9fd7
    observed: 2026-11-06T05:35:15Z
  - id: 2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline-b7f0e4
    aliases: OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-e830c9d0f1421775e2356bdb
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: passed (exit 0)."
    result: passed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0
    revision: 558a8144e8a74250566fd5b2a8fe2331f693204c
    environment: local-macos-arm64-docker
    content: 9bae963b22a44963853a24e6517d0df13ef146060e52133f5781d14e8f4c0935
    fingerprint: sha256:d31f3335566bcc44fbc4d10389d049ae7bb8c957e3f2ee7c56215a736b6f7ee6
    case: TC-github-linux-arm64-glibc-download-starts-on-baseline
    case_digest: 6c1458891f6ec19102613f5b61efd0e208d94a14c0a036f4bc247c882825e908
    observed: 2026-11-06T06:57:53Z
    duration_seconds: 2
  - id: 2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere-a7cf42
    aliases: OBS-tc-github-linux-arm64-musl-download-starts-anywhere-515683671907229ea5e79126
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: passed (exit 0)."
    result: passed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0
    revision: 558a8144e8a74250566fd5b2a8fe2331f693204c
    environment: local-macos-arm64-docker
    content: 9bae963b22a44963853a24e6517d0df13ef146060e52133f5781d14e8f4c0935
    fingerprint: sha256:89f47822b12c94514c628e202d2dd9d79bf49d4eb189b95189430d69a27b635a
    case: TC-github-linux-arm64-musl-download-starts-anywhere
    case_digest: 9b63920d3e9575e64918c797332f155b597e92179b987c70f28ce5d1993f1ec9
    observed: 2026-11-06T06:57:59Z
    duration_seconds: 4
  - id: 2026-11-06-tc-github-linux-baseline-lockstep-84c264
    aliases: OBS-tc-github-linux-baseline-lockstep-a0a16bc0ce1d0ef7aa9439d9
    claim: "Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: passed (exit 0)."
    result: passed
    command: python3 .github/scripts/test-linux-baseline.py
    revision: 558a8144e8a74250566fd5b2a8fe2331f693204c
    environment: local-macos-arm64
    content: 9bae963b22a44963853a24e6517d0df13ef146060e52133f5781d14e8f4c0935
    fingerprint: sha256:177be7a286638d977a8717e8d9e50e65cb1f388c075a397c48292a224d5ebe96
    case: TC-github-linux-baseline-lockstep
    case_digest: de0e92375a286f581f072544efb66b2b2df78cfb5dd821a50e0293aa9a788f03
    observed: 2026-11-06T06:58:08Z
    duration_seconds: 7
```
