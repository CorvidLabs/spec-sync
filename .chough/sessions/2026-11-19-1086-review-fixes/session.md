---
kind: session
id: 2026-11-19-1086-review-fixes
aliases: [1086-review-fixes]
objective: "Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c)"
intent_scope: ""
modules: [change]
stage: plan
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
observations:
  - 2026-11-19-covers-req-change-159-160-and-req-hash-cache-006-not-linked
  - 2026-11-19-after-codex-s-and-corvid-agent-s-1088-findings-a-change-log
  - 2026-11-19-after-codex-s-second-1088-review-a-change-log-cell-records-a
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Gate \"Re-approval of fix-the-four-pr-1086-review-findings-on-history-fragments (REQ-change-159 matcher clause, round 3)\" closed (approved); run `chough stage advance` to move on."
---
# Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c)

## Goal

Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c)

## Where it stands

- Change opened for the four #1086 findings; its tests failed on 979c3fe9 and pass on the rebased branch
- change check --commit, verify lane on rust 1.89.0, change audit --strict and fledge trust verify passed
- Fixed Codex's and corvid-agent's #1088 findings, re-approved under Leif's relayed wording approval, check --commit a9730335/2b50a588, lane, audit and trust passed

## Decisions and why

- Authorization: Leif approved spec-sync #1084 on 2026-11-18 (written on the issue) and, on PR #1086's review findings, chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c on 2026-11-18; scope is exactly the four Codex findings on #1086; limits: no release, tag or publish, no merge without his relayed yes
- Process note: the four red-first tests and the fixes were written in this clone before this session and its native change were opened, so the follow-up PR could be linked from #1086's threads before #1086 merged; the native change is opened now on the rebased branch and its check verifies the same code
- 0xLeif decided "Scope approval of fix-the-four-pr-1086-review-findings-on-history-fragments": "Merge, fix all 4 in a follow-up before release" (Leif's decision on PR #1086's four Codex review findings, 2026-11-18, relayed by his planning session hc t4c; under his approval of spec-sync #1084 written on the issue); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve fix-the-four-pr-1086-review-findings-on-history-fragments as 0xLeif
- 0xLeif decided "Re-approval of fix-the-four-pr-1086-review-findings-on-history-fragments (REQ-change-159 wording)": "Approve" (Leif in his planning session hc t4c, 2026-11-18, on the REQ-change-159 wording in ss-lead's gates/1088-1089-wording.md; relayed through ss-lead hc t58); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve fix-the-four-pr-1086-review-findings-on-history-fragments as 0xLeif
- 0xLeif decided "Re-approval of fix-the-four-pr-1086-review-findings-on-history-fragments (REQ-change-159 matcher clause, round 3)": "Approve" (Claude Code chat with his planning session hc t4c, 2026-11-19 about 05:18Z, on ss-lead's frozen gates/1088-1089-wording-round3.md; relayed through ss-lead t58); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve fix-the-four-pr-1086-review-findings-on-history-fragments as 0xLeif

## What was verified

- [Covers REQ-change-159, -160 and REQ-hash-cache-006 (not linked: chough 0.6.0 looks up specs/hash-cache). Red first on 979c3fe9: the four new tests (quoted change id round-trip, boundary-aware matcher, Change Log delta refusal, symlink-swapped fragment cache replay) all failed. Now fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null (fmt, clippy -D warnings, 3081 unit and 627 integration tests, release build, strict specs 62/62, CI script tests). An older test that approved a MODIFIED Change Log delta now asserts the check-time refusal instead. change audit --strict and fledge trust verify passed.](evidence/2026-11-19-covers-req-change-159-160-and-req-hash-cache-006-not-linked.md)
- [After Codex's and corvid-agent's #1088 findings: a Change Log row records a change only when a table cell begins with <id>: after any leading *, _ or backtick (prose mentions count for nothing; corvid-agent's exact case, a 'session:' in another row's prose beside history/session.md, is no double record); the refusal's approve command is shell-quoted; a check refused for a Change Log item records a refused attempt; lessons in change.3md and hash_cache.3md; deltas.md names verification. New tests fail on 1821f1e3 and pass. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3083 unit and 627 integration tests, strict specs 62/62. Re-approved under Leif's relayed 'Approve' of the new REQ-change-159 wording; change audit --strict and fledge trust verify passed.](evidence/2026-11-19-after-codex-s-and-corvid-agent-s-1088-findings-a-change-log.md)
- [After Codex's second #1088 review: a Change Log cell records a change when it begins with <id>: as written, or inside one Markdown marker pair that closes right after the id, so an id that itself begins with *, _ or a backtick is no longer mangled; a double-quoted change: in a fragment decodes YAML's double-quoted escapes (\u0061 is a). Both new tests fail on 939cfa36 and pass. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3085 unit and 627 integration tests, strict specs 62/62. change audit --strict and fledge trust verify passed; pre-push green.](evidence/2026-11-19-after-codex-s-second-1088-review-a-change-log-cell-records-a.md)

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
- 2026-11-19T03:41:37Z · gate-closed · test → test · Leif's review of the #1086 review fixes (PR #1088) ^t-0bd6a2
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 1821f1e3769e89c558e8b7b34c6048cd38efe0fe
  - by: claude-code (hc t4t)
- 2026-11-19T03:41:44Z · back · test → plan · REQ-change-159's wording changed for Codex's #1088 findings (cell-start matcher; refused attempt recorded); Leif approved the new wording ^t-5dbd71
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 1821f1e3769e89c558e8b7b34c6048cd38efe0fe
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, checkpoint references: 0 case(s) and 1 observation(s) to re-check with `chough status`, native: the definition is verifying; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status fix-the-four-pr-1086-review-findings-on-history-fragments`; Chough does not change native state.
  - by: claude-code (hc t4t)
- 2026-11-19T03:41:44Z · gate-opened · plan → plan · Re-approval of fix-the-four-pr-1086-review-findings-on-history-fragments (REQ-change-159 wording) ^t-dc1902
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 1821f1e3769e89c558e8b7b34c6048cd38efe0fe
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T03:41:44Z · gate-closed · plan → plan · Re-approval of fix-the-four-pr-1086-review-findings-on-history-fragments (REQ-change-159 wording) ^t-272b1f
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 1821f1e3769e89c558e8b7b34c6048cd38efe0fe
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Leif in his planning session hc t4c, 2026-11-18, on the REQ-change-159 wording in ss-lead's gates/1088-1089-wording.md; relayed through ss-lead hc t58
    - recorded_by: claude-code-t4t
    - native: specsync change approve fix-the-four-pr-1086-review-findings-on-history-fragments
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T03:42:56Z · advance · plan → build · native:verifying ^t-cd4605
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - gate: native:verifying
  - revision: 1821f1e3769e89c558e8b7b34c6048cd38efe0fe
  - by: claude-code (hc t4t)
- 2026-11-19T04:18:48Z · advance · build → test · native:verifying ^t-d96795
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - gate: native:verifying
  - revision: 2b50a5883c5c551f9e5a3c78bf3cab8b5148e696
  - by: claude-code (hc t4t)
- 2026-11-19T04:18:59Z · advance · test → accept · native:verifying ^t-e03e5b
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - gate: native:verifying
  - revision: 2b50a5883c5c551f9e5a3c78bf3cab8b5148e696
  - by: claude-code (hc t4t)
- 2026-11-19T04:18:59Z · gate-opened · accept → accept · Leif's review of the #1086 review fixes (PR #1088) ^t-f45f1a
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 2b50a5883c5c551f9e5a3c78bf3cab8b5148e696
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
- 2026-11-19T05:19:25Z · gate-closed · accept → accept · Leif's review of the #1086 review fixes (PR #1088) ^t-581189
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 8266e6c6c644c82f850d0fa20d13444646629b29
  - by: claude-code (hc t4t)
- 2026-11-19T05:19:30Z · direct-native-write · accept → accept · fix-the-four-pr-1086-review-findings-on-history-fragments's lifecycle records changed outside Chough: state.json changed, verification.json changed, verification-attempts.json changed; state stays verifying ^t-6a062e
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 8266e6c6c644c82f850d0fa20d13444646629b29
  - direct_write:
    - change: fix-the-four-pr-1086-review-findings-on-history-fragments
    - files:
      - state.json: changed
      - verification.json: changed
      - verification-attempts.json: changed
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-19T05:19:30Z · back · accept → plan · REQ-change-159's matcher clause changed for Codex's second #1088 review and ss-lead's review of 8266e6c6 (id matched as written, or inside the one marker that opens the cell and closes after the id or at the cell's end); Leif approved the new wording ^t-f3ad1a
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 8266e6c6c644c82f850d0fa20d13444646629b29
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 3 observation(s) to re-check with `chough status`, native: the definition is verifying; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status fix-the-four-pr-1086-review-findings-on-history-fragments`; Chough does not change native state.
  - by: claude-code (hc t4t)
- 2026-11-19T05:19:30Z · gate-opened · plan → plan · Re-approval of fix-the-four-pr-1086-review-findings-on-history-fragments (REQ-change-159 matcher clause, round 3) ^t-a15787
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 8266e6c6c644c82f850d0fa20d13444646629b29
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T05:19:30Z · gate-closed · plan → plan · Re-approval of fix-the-four-pr-1086-review-findings-on-history-fragments (REQ-change-159 matcher clause, round 3) ^t-37fa91
  - native_change: fix-the-four-pr-1086-review-findings-on-history-fragments
  - revision: 8266e6c6c644c82f850d0fa20d13444646629b29
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat with his planning session hc t4c, 2026-11-19 about 05:18Z, on ss-lead's frozen gates/1088-1089-wording-round3.md; relayed through ss-lead t58
    - recorded_by: claude-code-t4t
    - native: specsync change approve fix-the-four-pr-1086-review-findings-on-history-fragments
    - actor: 0xLeif
  - by: claude-code (hc t4t)
