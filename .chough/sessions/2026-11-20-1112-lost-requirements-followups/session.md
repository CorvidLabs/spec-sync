---
kind: session
id: 2026-11-20-1112-lost-requirements-followups
aliases: [1112-lost-requirements-followups]
objective: "The lost-requirements check (REQ-change-169) reads only what was canonical at the reference, blocks only changes that can still move, runs in the audit whatever a change's own errors, and says it cannot tell when a requirements directory cannot be read (#1112, #1113, #1116, #1117; definition drafted for t4c's scope check under Leif's standing OK)"
intent_scope: CHANGE
modules: [change, cmd_change]
stage: plan
stage_status: active
iteration: 1
callers: []
native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
pr: ""
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Gate \"Scope approval of make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a (at plan)\" closed (approved); run `chough stage advance` to move on."
---
# The lost-requirements check (REQ-change-169) reads only what was canonical at the reference, blocks only changes that can still move, runs in the audit whatever a change's own errors, and says it cannot tell when a requirements directory cannot be read (#1112, #1113, #1116, #1117; definition drafted for t4c's scope check under Leif's standing OK)

## Goal

The lost-requirements check (REQ-change-169) reads only what was canonical at the reference, blocks only changes that can still move, runs in the audit whatever a change's own errors, and says it cannot tell when a requirements directory cannot be read (#1112, #1113, #1116, #1117; definition drafted for t4c's scope check under Leif's standing OK)

## Where it stands

## Decisions and why

- No new intent criterion: the confirmed definition proposes none. CHANGE-17 and its three cases, approved by Leif for #1081, cover the four follow-ups: 17.a names every lost requirement and stops only a change that can still move (#1112, #1113), 17.b the audit reports the loss (#1116), 17.c the message says how to fix what is wrong (#1117).
- 0xLeif decided "Scope approval of make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a": "Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)" (Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'), confirmed for this definition (defs/1112-1113-1116-1117-definition.md: the recommended choices taken, no new refusal, all four requirements ADDED) by t4c on 2026-11-20 at about 14:50Z, sent directly from hc t4c while ss-lead t58 was stopped); recorded by claude-code-t4t; a relayed claim, not authenticated
- 0xLeif decided "Scope approval of make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a (at plan)": "Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)" (Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'), confirmed for this definition (defs/1112-1113-1116-1117-definition.md: the recommended choices taken, no new refusal, all four requirements ADDED) by t4c on 2026-11-20 at about 14:50Z, sent directly from hc t4c while ss-lead t58 was stopped); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-20T14:40:35Z · start · intent → intent · The lost-requirements check (REQ-change-169) reads only what was canonical at the reference, blocks only changes that can still move, runs in the audit whatever a change's own errors, and says it cannot tell when a requirements directory cannot be read (#1112, #1113, #1116, #1117; definition drafted for t4c's scope check under Leif's standing OK) ^t-6ea131
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - by: claude-code (hc t4t)
- 2026-11-20T15:11:57Z · direct-native-write · intent → intent · make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-d1e100
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - direct_write:
    - change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t81)
- 2026-11-20T15:11:57Z · advance · intent → design ·  ^t-73a7d1
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - by: claude-code (hc t81)
- 2026-11-20T15:11:58Z · gate-opened · design → design · Scope approval of make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a ^t-454f51
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - holds: design
  - drives: none
  - by: claude-code (hc t81)
- 2026-11-20T15:11:58Z · gate-closed · design → design · Scope approval of make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a ^t-8264f0
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)
    - where: Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'), confirmed for this definition (defs/1112-1113-1116-1117-definition.md: the recommended choices taken, no new refusal, all four requirements ADDED) by t4c on 2026-11-20 at about 14:50Z, sent directly from hc t4c while ss-lead t58 was stopped
    - recorded_by: claude-code-t4t
  - by: claude-code (hc t81)
- 2026-11-20T15:12:06Z · advance · design → plan ·  ^t-f6e9ed
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - by: claude-code (hc t81)
- 2026-11-20T15:12:07Z · gate-opened · plan → plan · Scope approval of make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a (at plan) ^t-8fc6a2
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - holds: plan
  - drives: approve
  - by: claude-code (hc t81)
- 2026-11-20T15:12:07Z · gate-closed · plan → plan · Scope approval of make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a (at plan) ^t-c120bd
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: ed258f58d734be7c4c0ac092ddbc934f492630d3
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)
    - where: Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'), confirmed for this definition (defs/1112-1113-1116-1117-definition.md: the recommended choices taken, no new refusal, all four requirements ADDED) by t4c on 2026-11-20 at about 14:50Z, sent directly from hc t4c while ss-lead t58 was stopped
    - recorded_by: claude-code-t4t
    - native: specsync change approve make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
    - actor: 0xLeif
  - by: claude-code (hc t81)
