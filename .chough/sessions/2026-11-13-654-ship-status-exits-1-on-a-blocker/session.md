---
kind: session
id: 2026-11-13-654-ship-status-exits-1-on-a-blocker
aliases: [654-ship-status-exits-1-on-a-blocker]
objective: change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)
intent_scope: ""
modules: [cmd_change]
stage: test
stage_status: active
iteration: 1
callers: []
native_change: change-ship-status-exits-1-when-it-reports-a-blocker
pr: ""
criteria: [SHIP-9.a]
requirements: []
cases: []
design: []
files:
  - src/commands/change.rs
  - tests/integration/change.rs
  - site/src/content/docs/cli.md
  - docs/ADOPTING.md
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Goal

change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Where it stands

## Decisions and why

- decided: authorization is Leif's own decision ('Exit 1 on a blocker (Recommended)', option text 'So scripts and CI can act on it.', coordinator chat 2026-11-13T14:12Z, relayed by claude-code-t370 and passed on by claude-code-t38c); REQ-cmd-change-039 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form

## What was verified

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
