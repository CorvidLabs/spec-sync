---
kind: session
id: 2026-11-19-1084-history-fragments
aliases: [1084-history-fragments]
objective: At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)
intent_scope: ""
modules: [change]
stage: plan
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
next: "Gate \"Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump\" closed (approved); run `chough stage advance` to move on."
---
# At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)

## Goal

At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)

## Where it stands

- Change opened with six deltas and its artifacts; failing tests in tests/integration/spec_history.rs fail on 65864de8 (7 of 8: no history subcommand, materialization bumps version:, check ignores fragments)

## Decisions and why

- Authorization: Leif approved spec-sync #1084 on 2026-11-18, written on the issue as 'Leif's decision (2026-11-18): approved', relayed to this agent by hc t4c as his scope approval; scope is the issue's three bullets; limits: no release, tag or publish, no merge without his relayed yes
- Intent: MERGE-1 already says two changes to one module merge cleanly with no conflict on version: or the change-log row; #1084 makes that hold on GitHub's merge button, which runs no merge driver. No new HI criterion is captured because none was confirmed
- decided: the freeze needs no command and rewrites nothing, because the issue says no row is rewritten and a per-spec marker would itself touch every spec; rejected: converting rows into fragments
- 0xLeif decided "Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1084 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump as 0xLeif

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
