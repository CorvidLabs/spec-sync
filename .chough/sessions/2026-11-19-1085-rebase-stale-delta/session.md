---
kind: session
id: 2026-11-19-1085-rebase-stale-delta
aliases: [1085-rebase-stale-delta]
objective: When a MODIFIED delta is stale only because main edited the same requirement, change check merges base, main and the delta word by word; non-overlapping edits rebase the delta automatically, keep the approval and record the rebase; overlapping edits are refused as today (#1085, Leif's approval on the issue, relayed by hc t4c)
intent_scope: ""
modules: [change]
stage: plan
stage_status: active
iteration: 1
callers: []
native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
pr: ""
criteria: [CHANGE-6.a, CHANGE-6.c]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-covers-req-change-161-req-change-122-and-req-cmd-change-038
  - 2026-11-19-after-fable-lead-s-corvid-agent-s-and-codex-s-1089-findings
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Gate \"Re-approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval (REQ-change-161 and REQ-cmd-change-038, round 3)\" closed (approved); run `chough stage advance` to move on."
---
# When a MODIFIED delta is stale only because main edited the same requirement, change check merges base, main and the delta word by word; non-overlapping edits rebase the delta automatically, keep the approval and record the rebase; overlapping edits are refused as today (#1085, Leif's approval on the issue, relayed by hc t4c)

## Goal

When a MODIFIED delta is stale only because main edited the same requirement, change check merges base, main and the delta word by word; non-overlapping edits rebase the delta automatically, keep the approval and record the rebase; overlapping edits are refused as today (#1085, Leif's approval on the issue, relayed by hc t4c)

## Where it stands

- Change opened with deltas for change and cmd_change and its artifacts; tests/integration/rebase_stale_delta.rs fails on 99f61b2b: change check refuses the non-overlapping stale delta (the overlapping control passes, as it should)
- Rebased onto main 3bebe8b0 (CHANGELOG conflict resolved keeping both entries); change check --commit, verify lane on rust 1.89.0, audit --strict and trust verify passed
- Fixed the review findings, re-approved under Leif's relayed wording approval, check --commit 2ce4e060/5b2dfce4, lane, audit and trust passed

## Decisions and why

- Authorization: Leif approved spec-sync #1085 on 2026-11-18, written on the issue as 'Leif's decision (2026-11-18): approved.', relayed to this agent by hc t4c as his scope approval; limits: no release, tag or publish, no merge without his relayed yes
- Stacked on the #1084 branch (PR #1086) so this change's own check writes history fragments; it is rebased onto main once #1086 merges
- 0xLeif decided "Scope approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1085 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval as 0xLeif
- 0xLeif decided "Re-approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval (REQ-change-161, REQ-cmd-change-038 wording)": "Approve" (Leif in his planning session hc t4c, 2026-11-18, on the REQ-change-161 and REQ-cmd-change-038 wording in ss-lead's gates/1088-1089-wording.md; relayed through ss-lead hc t58); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval as 0xLeif
- 0xLeif decided "Re-approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval (REQ-change-161 and REQ-cmd-change-038, round 3)": "Approve" (Claude Code chat with his planning session hc t4c, 2026-11-19 about 05:18Z, on ss-lead's frozen gates/1088-1089-wording-round3.md; relayed through ss-lead t58); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval as 0xLeif

## What was verified

- [Covers REQ-change-161, REQ-change-122 and REQ-cmd-change-038 (the last not linked: chough 0.6.0 looks up specs/cmd-change). Red first on 99f61b2b: tests/integration/rebase_stale_delta.rs's discriminator failed (change check refused the non-overlapping stale delta); the overlapping control passed. Now fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null (fmt, clippy -D warnings, 3084 unit and 630 integration tests, release build, strict specs 62/62, CI script tests): 3 integration and 7 unit tests cover the rebase, status text and JSON, a tampered record, re-approval and acceptance after a rebase, and the refusing controls (overlap, spec section, workflow v1, no history, CI). change audit --strict and fledge trust verify passed.](evidence/2026-11-19-covers-req-change-161-req-change-122-and-req-cmd-change-038.md)
- [After fable-lead's, corvid-agent's and Codex's #1089 findings: main's block must be text the default branch held more recently than the approval-time base; rebases are staged and written only when no stale item is left; the merge anchors only on words unique in all three texts; status lists only rebases that recompute; rebases are keyed by approval index, time and digest; the stale refusal names git show origin/main:<file>. New tests (branch-side edit before the first check, after materialization and in a merge resolution; no origin; origin moved back; an older main text; the mixed two-item refusal byte-identical; repeated words; same-second re-approval; status of a non-recomputing record; an end-to-end branch-edit refusal) fail on 2c06cef4 and pass. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3091 unit and 631 integration tests, strict specs 62/62. Re-approved under Leif's relayed 'Approve' of the new REQ-change-161 and REQ-cmd-change-038 wording; change audit --strict and fledge trust verify passed.](evidence/2026-11-19-after-fable-lead-s-corvid-agent-s-and-codex-s-1089-findings.md)

## Open threads

## Timeline

- 2026-11-19T01:32:05Z · start · intent → intent · When a MODIFIED delta is stale only because main edited the same requirement, change check merges base, main and the delta word by word; non-overlapping edits rebase the delta automatically, keep the approval and record the rebase; overlapping edits are refused as today (#1085, Leif's approval on the issue, relayed by hc t4c) ^t-eec275
  - revision: 99f61b2b5667d60c3b0705902486f910c8e067c5
  - by: claude-code (hc t4t)
- 2026-11-19T01:32:14Z · advance · intent → design ·  ^t-67690d
  - revision: 99f61b2b5667d60c3b0705902486f910c8e067c5
  - by: claude-code (hc t4t)
- 2026-11-19T01:32:14Z · skip · design → plan · Record and CLI behaviour only: no visuals or interactions ^t-62eba3
  - revision: 99f61b2b5667d60c3b0705902486f910c8e067c5
  - by: claude-code (hc t4t)
- 2026-11-19T01:37:19Z · gate-opened · plan → plan · Scope approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval ^t-08ef0a
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 99f61b2b5667d60c3b0705902486f910c8e067c5
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T01:37:19Z · gate-closed · plan → plan · Scope approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval ^t-54901c
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 99f61b2b5667d60c3b0705902486f910c8e067c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's decision (2026-11-18): approved.
    - where: Written on spec-sync issue #1085 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue
    - recorded_by: claude-code-t4t
    - native: specsync change approve rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T01:37:27Z · advance · plan → build · native:approved ^t-50966e
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - gate: native:approved
  - revision: 32cc5e0295db17d38f6164b683214e7a719e8bbd
  - by: claude-code (hc t4t)
- 2026-11-19T02:47:30Z · advance · build → test · native:verifying ^t-2e1836
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - gate: native:verifying
  - revision: 101edcf006e0b8f366a031bf22d89452d6c89daa
  - by: claude-code (hc t4t)
- 2026-11-19T02:47:44Z · advance · test → accept · native:verifying ^t-31acae
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - gate: native:verifying
  - revision: 101edcf006e0b8f366a031bf22d89452d6c89daa
  - by: claude-code (hc t4t)
- 2026-11-19T02:47:44Z · gate-opened · accept → accept · Leif's review of #1085 ^t-f3d76a
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 101edcf006e0b8f366a031bf22d89452d6c89daa
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
- 2026-11-19T04:01:01Z · gate-closed · accept → accept · Leif's review of #1085 ^t-e10ae2
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 2c06cef49e88a660bdf145cabde79c705772e07d
  - by: claude-code (hc t4t)
- 2026-11-19T04:01:04Z · back · accept → plan · REQ-change-161 and REQ-cmd-change-038 wording changed for fable-lead's, corvid-agent's and Codex's #1089 findings; Leif approved the new wording ^t-589ce9
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 2c06cef49e88a660bdf145cabde79c705772e07d
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 1 observation(s) to re-check with `chough status`, native: the definition is verifying; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval`; Chough does not change native state.
  - by: claude-code (hc t4t)
- 2026-11-19T04:01:04Z · gate-opened · plan → plan · Re-approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval (REQ-change-161, REQ-cmd-change-038 wording) ^t-b767c6
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 2c06cef49e88a660bdf145cabde79c705772e07d
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T04:01:54Z · gate-closed · plan → plan · Re-approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval (REQ-change-161, REQ-cmd-change-038 wording) ^t-6827c7
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 2c06cef49e88a660bdf145cabde79c705772e07d
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Leif in his planning session hc t4c, 2026-11-18, on the REQ-change-161 and REQ-cmd-change-038 wording in ss-lead's gates/1088-1089-wording.md; relayed through ss-lead hc t58
    - recorded_by: claude-code-t4t
    - native: specsync change approve rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T04:02:20Z · advance · plan → build · native:verifying ^t-6be65d
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - gate: native:verifying
  - revision: 2c06cef49e88a660bdf145cabde79c705772e07d
  - by: claude-code (hc t4t)
- 2026-11-19T04:03:31Z · advance · build → test · native:verifying ^t-117ac1
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - gate: native:verifying
  - revision: 5b2dfce4e6edb1404a303d3e9167ee759c806382
  - by: claude-code (hc t4t)
- 2026-11-19T04:03:51Z · advance · test → accept · native:verifying ^t-3c54c4
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - gate: native:verifying
  - revision: 5b2dfce4e6edb1404a303d3e9167ee759c806382
  - by: claude-code (hc t4t)
- 2026-11-19T04:03:51Z · gate-opened · accept → accept · Leif's review of #1085 (PR #1089) ^t-e08a9a
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 5b2dfce4e6edb1404a303d3e9167ee759c806382
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
- 2026-11-19T05:24:23Z · gate-closed · accept → accept · Leif's review of #1085 (PR #1089) ^t-6f26b5
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 0d31218a63ec66cb30293a3f75c641eb873717a5
  - by: claude-code (hc t4t)
- 2026-11-19T05:24:36Z · back · accept → plan · REQ-change-161 and REQ-cmd-change-038 reworded for Codex's second #1089 review, the tester's regression on 0d31218a and ss-lead's reviews (main's block read at HEAD's merge-base with the default branch; status names a refused record); Leif approved the new wording ^t-97bdab
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 0d31218a63ec66cb30293a3f75c641eb873717a5
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 2 observation(s) to re-check with `chough status`, native: the definition is verifying; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval`; Chough does not change native state.
  - by: claude-code (hc t4t)
- 2026-11-19T05:24:36Z · gate-opened · plan → plan · Re-approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval (REQ-change-161 and REQ-cmd-change-038, round 3) ^t-5d7e92
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 0d31218a63ec66cb30293a3f75c641eb873717a5
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T05:24:36Z · gate-closed · plan → plan · Re-approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval (REQ-change-161 and REQ-cmd-change-038, round 3) ^t-14f36e
  - native_change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
  - revision: 0d31218a63ec66cb30293a3f75c641eb873717a5
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat with his planning session hc t4c, 2026-11-19 about 05:18Z, on ss-lead's frozen gates/1088-1089-wording-round3.md; relayed through ss-lead t58
    - recorded_by: claude-code-t4t
    - native: specsync change approve rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
    - actor: 0xLeif
  - by: claude-code (hc t4t)
