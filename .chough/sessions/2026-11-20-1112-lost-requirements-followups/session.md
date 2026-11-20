---
kind: session
id: 2026-11-20-1112-lost-requirements-followups
aliases: [1112-lost-requirements-followups]
objective: "The lost-requirements check (REQ-change-169) reads only what was canonical at the reference, blocks only changes that can still move, runs in the audit whatever a change's own errors, and says it cannot tell when a requirements directory cannot be read (#1112, #1113, #1116, #1117; definition drafted for t4c's scope check under Leif's standing OK)"
intent_scope: CHANGE
modules: [change, cmd_change]
stage: accept
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
observations:
  - 2026-11-20-1112-1113-1116-1117-req-change-175-176-177-req-cmd-change-045
  - 2026-11-20-1134-round-one-corvid-agent-s-review-codex-s-two-p2s-red-first
  - 2026-11-20-1134-codex-round-two-one-p2-red-first-c197e040-both-working
bugs: []
blockers: []
waiting_on_human:
  - gate: "Leif's review of the lost-requirements follow-ups (#1112, #1113, #1116, #1117)"
    holds: accept
    drives: review
    opened: 2026-11-20T15:52:32Z
run_consents: []
next: Continue accept
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

- [#1112, #1113, #1116, #1117 (REQ-change-175, 176, 177, REQ-cmd-change-045). Red first a6a8725f: seven tests through the binary failed on #1108's head ed258f58, each as its issue says (spec registered beside / specs_dir moved onto an existing requirements.md; ship-status of an archived change; audit with a malformed delta / an unreadable approval; an unreadable directory / file); a guard (unreadable directory, nothing lost) passes. Fix 84c668b4. Lane at 76585573 through one-lane: 3137 unit, 655 integration, 62/62 specs, 100% coverage; change audit --strict exit 0. Real repos, #1108's build against this one: private starling at PR #43's last commit, ship-status of the archived change exit 1 with the 26-id blocker before, exit 0 with none now; at the commit before the archive both refuse with the same lines; private peck full clone and the spec-sync and chough mains: the same audit lines.](evidence/2026-11-20-1112-1113-1116-1117-req-change-175-176-177-req-cmd-change-045.md)
- [#1134 round one (corvid-agent's review, Codex's two P2s). Red first b7c738aa: with nothing canonical at the reference (specs_dir outside the project, or a configuration that does not parse) the diff ran with no pathspec and read every requirements.md (two tests, regression from #1108's head); a registered spec's directory below an unreadable directory was passed by before the scan named it. Four more tests cover criteria that had none (REQ-change-175 c3, 176 c2, 177 c2, REQ-cmd-change-045 c2). Fix aad978c4: return before the diff when the reference scope is empty; the scope-directory check names a path or metadata it cannot read; testing.md maps each criterion to its test. Lane at 30c27ad4 (3137 unit, 660 integration, 62/62, 100%); audit --strict 0; real repos as round one (starling archived tip exit 0 no blocker; active commit, peck, spec-sync and chough mains the same lines as #1108's build).](evidence/2026-11-20-1134-round-one-corvid-agent-s-review-codex-s-two-p2s-red-first.md)
- [#1134 Codex round two (one P2). Red first c197e040: both working-tree scans dropped an error the directory listing yields, or one from reading an entry's kind (entries.flatten()), so a requirement there read as lost; the test drives the scan's entry step with injected errors, since a failing filesystem cannot be made. Fix 16dfbbc0: one entry step, listed_entry, keeps the error with its path for the cannot-tell refusal. Lane at bd79255b (3138 unit, 660 integration, 62/62, 100%; the first run lost one unrelated test, archived_upstream::review_and_finalize_stop_when_the_upstream_archive_cannot_be_read, to a packed object under load; it passed 3 of 3 alone and the rerun was green); audit --strict 0; real repos as before.](evidence/2026-11-20-1134-codex-round-two-one-p2-red-first-c197e040-both-working.md)

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
- 2026-11-20T15:52:29Z · direct-native-write · plan → plan · make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a's lifecycle records changed outside Chough: state.json changed, verification.json added, verification-attempts.json added; state approved → verifying ^t-b4a7dc
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: 76585573d0838b387404ac762f700e2fb5cd6ab0
  - direct_write:
    - change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
    - files:
      - state.json: changed
      - verification.json: added
      - verification-attempts.json: added
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t81)
- 2026-11-20T15:52:29Z · advance · plan → build · native:verifying ^t-056378
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - gate: native:verifying
  - revision: 76585573d0838b387404ac762f700e2fb5cd6ab0
  - by: claude-code (hc t81)
- 2026-11-20T15:52:30Z · advance · build → test · native:verifying ^t-84e93d
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - gate: native:verifying
  - revision: 76585573d0838b387404ac762f700e2fb5cd6ab0
  - by: claude-code (hc t81)
- 2026-11-20T15:52:32Z · advance · test → accept · native:verifying ^t-b6da2b
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - gate: native:verifying
  - revision: 76585573d0838b387404ac762f700e2fb5cd6ab0
  - by: claude-code (hc t81)
- 2026-11-20T15:52:32Z · gate-opened · accept → accept · Leif's review of the lost-requirements follow-ups (#1112, #1113, #1116, #1117) ^t-243a7b
  - native_change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
  - revision: 76585573d0838b387404ac762f700e2fb5cd6ab0
  - holds: accept
  - drives: review
  - by: claude-code (hc t81)
