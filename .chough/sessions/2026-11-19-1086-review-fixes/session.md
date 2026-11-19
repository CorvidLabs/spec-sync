---
kind: session
id: 2026-11-19-1086-review-fixes
aliases: [1086-review-fixes]
objective: "Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c)"
intent_scope: ""
modules: [change]
stage: test
stage_status: active
iteration: 1
callers: []
native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
pr: ""
criteria: [MERGE-1]
requirements: []
cases: []
design: []
files: []
observations: [2026-11-19-covers-req-change-159-160-and-req-hash-cache-006-not-linked]
bugs: []
blockers: []
waiting_on_human:
  - gate: "Leif's review of the #1086 review fixes (PR #1088)"
    holds: accept
    drives: review
    opened: 2026-11-19T02:36:45Z
run_consents: []
next: Continue test
---
# Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c)

## Goal

Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c)

## Where it stands

- Change opened for the four #1086 findings; its tests failed on 979c3fe9 and pass on the rebased branch
- change check --commit, verify lane on rust 1.89.0, change audit --strict and fledge trust verify passed

## Decisions and why

- Authorization: Leif approved spec-sync #1084 on 2026-11-18 (written on the issue) and, on PR #1086's review findings, chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c on 2026-11-18; scope is exactly the four Codex findings on #1086; limits: no release, tag or publish, no merge without his relayed yes
- Process note: the four red-first tests and the fixes were written in this clone before this session and its native change were opened, so the follow-up PR could be linked from #1086's threads before #1086 merged; the native change is opened now on the rebased branch and its check verifies the same code
- 0xLeif decided "Scope approval of fix-the-four-pr-1086-review-findings-on-history-fragments": "Merge, fix all 4 in a follow-up before release" (Leif's decision on PR #1086's four Codex review findings, 2026-11-18, relayed by his planning session hc t4c; under his approval of spec-sync #1084 written on the issue); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve fix-the-four-pr-1086-review-findings-on-history-fragments as 0xLeif

## What was verified

- [Covers REQ-change-159, -160 and REQ-hash-cache-006 (not linked: chough 0.6.0 looks up specs/hash-cache). Red first on 979c3fe9: the four new tests (quoted change id round-trip, boundary-aware matcher, Change Log delta refusal, symlink-swapped fragment cache replay) all failed. Now fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null (fmt, clippy -D warnings, 3081 unit and 627 integration tests, release build, strict specs 62/62, CI script tests). An older test that approved a MODIFIED Change Log delta now asserts the check-time refusal instead. change audit --strict and fledge trust verify passed.](evidence/2026-11-19-covers-req-change-159-160-and-req-hash-cache-006-not-linked.md)

## Open threads

## Timeline

- 2026-11-19T02:10:42Z · start · intent → intent · Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c) ^t-228b7d
  - revision: 069e6eaa9565a8ebf4837b0fb1f4f95c51234d0d
  - by: claude-code (hc t4t)
- 2026-11-19T02:10:51Z · advance · intent → design ·  ^t-c7d545
  - revision: 069e6eaa9565a8ebf4837b0fb1f4f95c51234d0d
  - by: claude-code (hc t4t)
- 2026-11-19T02:10:52Z · skip · design → plan · Record and validation behaviour only: no visuals or interactions ^t-34c466
  - revision: 069e6eaa9565a8ebf4837b0fb1f4f95c51234d0d
  - by: claude-code (hc t4t)
- 2026-11-19T02:11:54Z · gate-opened · plan → plan · Scope approval of fix-the-four-pr-1086-review-findings-on-history-fragments ^t-639da2
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 069e6eaa9565a8ebf4837b0fb1f4f95c51234d0d
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T02:11:54Z · gate-closed · plan → plan · Scope approval of fix-the-four-pr-1086-review-findings-on-history-fragments ^t-4039cf
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 069e6eaa9565a8ebf4837b0fb1f4f95c51234d0d
  - relayed_by:
    - decided_by: 0xLeif
    - words: Merge, fix all 4 in a follow-up before release
    - where: Leif's decision on PR #1086's four Codex review findings, 2026-11-18, relayed by his planning session hc t4c; under his approval of spec-sync #1084 written on the issue
    - recorded_by: claude-code-t4t
    - native: specsync change approve fix-the-four-pr-1086-review-findings-on-history-fragments
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T02:36:25Z · advance · plan → build · native:verifying ^t-ff2e18
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - gate: native:verifying
  - revision: de6a78796022977c87002ea9610fb0911a406f8e
  - by: claude-code (hc t4t)
- 2026-11-19T02:36:45Z · advance · build → test · native:verifying ^t-ebb9b8
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - gate: native:verifying
  - revision: de6a78796022977c87002ea9610fb0911a406f8e
  - by: claude-code (hc t4t)
- 2026-11-19T02:36:45Z · gate-opened · test → test · Leif's review of the #1086 review fixes (PR #1088) ^t-e95803
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: de6a78796022977c87002ea9610fb0911a406f8e
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
