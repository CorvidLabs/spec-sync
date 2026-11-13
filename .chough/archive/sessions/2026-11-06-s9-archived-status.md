---
kind: session
id: 2026-11-06-s9-archived-status
aliases: [s9-archived-status]
objective: Archived change status within SPEED-1.a budgets, plus actionable read timeouts (S9)
intent_scope: SPEED-1.a SPEED-1.c SPEED-1.d
stage: intent
stage_status: archived
iteration: 1
native_change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
closed: 2026-11-13
outcome: done
---
# Archived change status within SPEED-1.a budgets, plus actionable read timeouts (S9)

## Goal

Archived change status within SPEED-1.a budgets, plus actionable read timeouts (S9)

## Where it stands

## Decisions and why

- Leif (2026-11-05 21:43 MST, orc tab, relayed by orc t18): "Start step 1" (measurement only) for S9
- Leif (2026-11-05 22:35 MST, orc tab, relayed by orc t18): "Approve all" for S9: approach A and B, the budgets, and the --timeout design

## What was verified

## Bugs

None.

## Open threads

## Timeline

- 2026-11-06T05:44:18Z · gate-opened · intent → intent · Native approval of archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout ^t-13df81
  - native_change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
  - revision: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
- 2026-11-13T04:45:37Z · gate-closed · intent → intent · Native approval of archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout ^t-5aa1eb
  - native_change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
  - revision: 131f31af6dd3c60b8b9f370ca7e67ac498f91158

Closed 2026-11-13: done.
- 2026-11-13T14:19:16Z · correction · intent → intent · Withdrawn, not passed: the human decision this gate waited for was already made and is recorded in the archived native change. Leif chose 'Yes, withdraw (Recommended)' at 2026-11-13T03:37Z in the coordinator chat (relayed by claude-code-t370, posted on spec-sync#1073 at 03:38Z), asked: 'Those repos also have old human gates still open on sessions that already finished (5 in spec-sync, 1 in raven), so chough doctor complains. Withdraw them, with the reason "session finished; its change is archived"?' Reason: session finished; its change is archived. chough doctor --fix closed the gate without recording a reason (CorvidLabs/chough#250). Recorded by claude-code-t38c (Claude Opus 5.5, hc t38c). ^t-323f2c
  - corrects: ^t-5aa1eb
  - by: claude-code (hc t38c)

## Receipts

```yaml
receipts: []
```
