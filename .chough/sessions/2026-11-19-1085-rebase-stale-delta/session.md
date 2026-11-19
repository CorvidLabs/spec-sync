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
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Gate \"Scope approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval\" closed (approved); run `chough stage advance` to move on."
---
# When a MODIFIED delta is stale only because main edited the same requirement, change check merges base, main and the delta word by word; non-overlapping edits rebase the delta automatically, keep the approval and record the rebase; overlapping edits are refused as today (#1085, Leif's approval on the issue, relayed by hc t4c)

## Goal

When a MODIFIED delta is stale only because main edited the same requirement, change check merges base, main and the delta word by word; non-overlapping edits rebase the delta automatically, keep the approval and record the rebase; overlapping edits are refused as today (#1085, Leif's approval on the issue, relayed by hc t4c)

## Where it stands

- Change opened with deltas for change and cmd_change and its artifacts; tests/integration/rebase_stale_delta.rs fails on 99f61b2b: change check refuses the non-overlapping stale delta (the overlapping control passes, as it should)

## Decisions and why

- Authorization: Leif approved spec-sync #1085 on 2026-11-18, written on the issue as 'Leif's decision (2026-11-18): approved.', relayed to this agent by hc t4c as his scope approval; limits: no release, tag or publish, no merge without his relayed yes
- Stacked on the #1084 branch (PR #1086) so this change's own check writes history fragments; it is rebased onto main once #1086 merges
- 0xLeif decided "Scope approval of rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1085 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval as 0xLeif

## What was verified

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
