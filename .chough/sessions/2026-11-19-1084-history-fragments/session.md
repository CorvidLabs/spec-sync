---
kind: session
id: 2026-11-19-1084-history-fragments
aliases: [1084-history-fragments]
objective: At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)
intent_scope: ""
modules: [change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
pr: ""
criteria: [MERGE-1]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Full suite under GIT_CONFIG_GLOBAL=/dev/null, release build, then chough change check --commit with the branch build, verify lane, pre-push, PR
---
# At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)

## Goal

At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)

## Where it stands

- Change opened with six deltas and its artifacts; failing tests in tests/integration/spec_history.rs fail on 65864de8 (7 of 8: no history subcommand, materialization bumps version:, check ignores fragments)
- Implemented REQ-change-159/160, REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037, REQ-cli-args-026, REQ-cli-014: fragment at materialization, legacy row still counts, fragment in commit scope/coverage/ownership/verification-commit paths/send-back, check validation and cache binding, specsync history; tests updated for no bump/row; specs, change.3md z=53, site docs, AGENTS.md, CHANGELOG

## Decisions and why

- Authorization: Leif approved spec-sync #1084 on 2026-11-18, written on the issue as 'Leif's decision (2026-11-18): approved', relayed to this agent by hc t4c as his scope approval; scope is the issue's three bullets; limits: no release, tag or publish, no merge without his relayed yes
- Intent: MERGE-1 already says two changes to one module merge cleanly with no conflict on version: or the change-log row; #1084 makes that hold on GitHub's merge button, which runs no merge driver. No new HI criterion is captured because none was confirmed
- decided: the freeze needs no command and rewrites nothing, because the issue says no row is rewritten and a per-spec marker would itself touch every spec; rejected: converting rows into fragments
- 0xLeif decided "Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1084 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump as 0xLeif
- Delta corrected before any code was checked: it now also MODIFIES REQ-change-021, REQ-change-092 and REQ-change-123, which still described a Change Log row and a version bump that the issue removes; same scope, so re-approved under the same relayed authority
- 0xLeif decided "Scope re-approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires); recorded by claude-code-t4t; a relayed claim, not authenticated
- 0xLeif decided "Scope re-approval at plan of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump as 0xLeif
- decided: a module owns every .md file in its history/ directory for coverage and acceptance ownership, as it owns its companions, because a successor's manifest can name a predecessor's fragment; commit staging stays limited to the change's own fragment
- Environment note: this Mac's global git config sets tag.gpgsign, so changelog::tests' lightweight git tag fails; they pass with GIT_CONFIG_GLOBAL=/dev/null and do not touch this change

## What was verified

## Open threads

## Timeline

- 2026-11-19T00:29:41Z · start · intent → intent · At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c) ^t-0a145b
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - by: claude-code (hc t4t)
- 2026-11-19T00:29:51Z · advance · intent → design ·  ^t-2c39e2
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - by: claude-code (hc t4t)
- 2026-11-19T00:29:51Z · skip · design → plan · CLI and record-format change only: no visuals or interactions ^t-478550
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - by: claude-code (hc t4t)
- 2026-11-19T00:38:22Z · gate-opened · plan → plan · Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-c7128a
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T00:38:27Z · gate-closed · plan → plan · Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-2f08e1
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's decision (2026-11-18): approved.
    - where: Written on spec-sync issue #1084 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets
    - recorded_by: claude-code-t4t
    - native: specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T00:38:34Z · advance · plan → build · native:approved ^t-27e158
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - gate: native:approved
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:21Z · gate-opened · build → build · Scope re-approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-c065e2
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - holds: build
  - drives: none
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:21Z · gate-closed · build → build · Scope re-approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-00e274
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's decision (2026-11-18): approved.
    - where: Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires
    - recorded_by: claude-code-t4t
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:40Z · back · build → plan · The delta gained MODIFIED REQ-change-021, -092 and -123; the build-stage gate drove no native approval, so the re-approval is recorded at plan ^t-847f37
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, native: the definition is approved; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump`; Chough does not change native state.
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:40Z · gate-opened · plan → plan · Scope re-approval at plan of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-a76902
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:55Z · gate-closed · plan → plan · Scope re-approval at plan of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-8e7534
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's decision (2026-11-18): approved.
    - where: Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires
    - recorded_by: claude-code-t4t
    - native: specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:59Z · advance · plan → build · native:approved ^t-1c41ef
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - gate: native:approved
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - by: claude-code (hc t4t)
