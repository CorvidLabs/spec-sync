---
kind: session
id: 2026-11-13-476-action-expected-sha256
aliases: [476-action-expected-sha256]
objective: The GitHub Action checks the release archive against a SHA-256 the consumer pins, one per platform (#476)
intent_scope: ""
modules: [github]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
pr: ""
criteria: [CI-1.b]
requirements: []
cases: []
design: []
files:
  - action.yml
  - .github/scripts/verify-release-digest.sh
  - .github/scripts/test-verify-release-digest.sh
  - .github/workflows/ci.yml
  - fledge.toml
observations:
  - 2026-11-13-at-9786779ec-feat-476-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-at-9786779ec-verify-step-5-release-candidate-test-failed-3
  - 2026-11-13-at-7f98a979a-after-the-docs-fix-release-candidate-test-67
  - 2026-11-13-red-first-control-on-main-bb7682a78-the-action-has-no-expected
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
    holds: accept
    drives: review
    opened: 2026-11-13T18:39:21Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# The GitHub Action checks the release archive against a SHA-256 the consumer pins, one per platform (#476)

## Goal

The GitHub Action checks the release archive against a SHA-256 the consumer pins, one per platform (#476)

## Where it stands

- Merged main, ran the verify lane, fixed the docs example the release validator refused, reran the remaining steps

## Decisions and why

- decided: authorization is Leif's own approval ('#476 Action checksum', 03:49Z, and 'One per platform (Recommended)', 04:04Z, coordinator chat, relayed by claude-code-t370); REQ-github-031 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form
- decided: the release-candidate failure evidence (2026-11-13-at-9786779ec-verify-step-5-…) points at the rerun's log, which passed after the fix; the failing output itself is in the verify log attached to 2026-11-13-at-9786779ec-feat-476-…, lines 7636-7744, because the log file was rewritten by the rerun
- decided: no new lesson for the github module, because the change follows the existing sidecar-checksum pattern

## What was verified

- [At 9786779ec (feat/476 with origin/main bb7682a78 merged in), fledge lanes run verify steps 1 to 4 passed: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 619 passed), release build and strict spec check.](evidence/2026-11-13-at-9786779ec-feat-476-with-origin-main-bb7682a78-merged-in.md)
- [At 9786779ec, verify step 5 release-candidate-test failed 3 tests: this change's docs example used CorvidLabs/spec-sync@<commit>, and the release validator requires every site Action ref to be @v6.3.0. Fixed in 7f98a979a in this session: the example is a with: fragment fenced as text, and the page says @v6.3.0 lacks the input. No bug record: found and fixed before review.](evidence/2026-11-13-at-9786779ec-verify-step-5-release-candidate-test-failed-3.md)
- [At 7f98a979a, after the docs fix: release-candidate-test (67 tests), ci-gate-test, linux-baseline-test, ci-trigger-paths-test, ci-attest-and-summary-test, clippy-all-targets-test, release-digest-test (the verifier's pass and refusal cases) and spec-check passed, and cargo test --test integration documented/action passed 17. Rust sources are unchanged since steps 1 to 4.](evidence/2026-11-13-at-7f98a979a-after-the-docs-fix-release-candidate-test-67.md)
- [Red-first control: on main bb7682a78 the Action has no expected-sha256 input and no verifier, so .github/scripts/test-verify-release-digest.sh fails at its first case. This change adds them; not a new bug.](evidence/2026-11-13-red-first-control-on-main-bb7682a78-the-action-has-no-expected.md)

## Open threads

## Timeline

- 2026-11-13T17:52:25Z · start · intent → intent · The GitHub Action checks the release archive against a SHA-256 the consumer pins, one per platform (#476) ^t-66a529
  - revision: 9786779ec78468bf0656990e3e21a5593c18ad39
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:27Z · advance · intent → design ·  ^t-ab8e1d
  - native_change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
  - revision: 9786779ec78468bf0656990e3e21a5593c18ad39
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:27Z · skip · design → plan · Docs, CI and CLI behaviour only: no visuals or interactions ^t-ad66fe
  - native_change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
  - revision: 9786779ec78468bf0656990e3e21a5593c18ad39
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:29Z · advance · plan → build · native:verifying ^t-1de7a5
  - native_change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
  - gate: native:verifying
  - revision: 9786779ec78468bf0656990e3e21a5593c18ad39
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:29Z · advance · build → test · native:verifying ^t-3428f5
  - native_change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
  - gate: native:verifying
  - revision: 9786779ec78468bf0656990e3e21a5593c18ad39
  - by: claude-code (hc t38p)
- 2026-11-13T18:39:21Z · advance · test → accept · native:verifying ^t-014bf1
  - native_change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
  - gate: native:verifying
  - revision: 7f98a979abe8422e8e33ea4f54e27ebb28a8c734
  - by: claude-code (hc t38p)
- 2026-11-13T18:39:21Z · gate-opened · accept → accept · Implementation review of the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform ^t-d17c94
  - native_change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
  - revision: 7f98a979abe8422e8e33ea4f54e27ebb28a8c734
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
