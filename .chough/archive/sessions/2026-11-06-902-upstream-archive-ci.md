---
kind: session
id: 2026-11-06-902-upstream-archive-ci
aliases: [902-upstream-archive-ci]
objective: Archive readiness must not tell an upstream-archived copy to run ship (#902)
intent_scope: CHANGE
modules: [cmd_change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
pr: ""
criteria: [SHIP-4.a]
requirements: [REQ-cmd-change-029]
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "check --commit, audit --strict, pre-push, push, PR (Fixes #902), verify, ready for Leif's GitHub review"
closed: 2026-11-06
outcome: closed-without-done
---
# Archive readiness must not tell an upstream-archived copy to run ship (#902)

## Goal

Archive readiness must not tell an upstream-archived copy to run ship (#902)

## Where it stands

- Red first: test-archive-readiness.sh prints 'Run specsync change ship stale-copy' for an upstream-archived copy; commands::change::tests::an_upstream_archived_copy_is_not_told_to_ship fails on main cd8cab5e
- Fix applied (57e5e6eb): archive_candidate_can_ship consults upstream_archive_of; archive-readiness.sh mirrors the test without fetching; an_upstream_archived_copy_is_not_told_to_ship and test-archive-readiness.sh green
- Merged main through #944 at b0b2d578; lesson at cmd_change z=8; tests green on the merged tree

## Decisions and why

- orc t18 (2026-11-06, overnight): take #902 in worktree ss-902 to 'ready for approval' (queued for Leif)
- decided: the script mirrors upstream_archive_of and never fetches, because the Archive gate runs without SpecSync and other repositories use it through the packaged Action; with no remote default ref it keeps today's classification, as Rust does when Git cannot answer
- rejected: a Rust integration test through 'change status', because status prints the pull-request sentence without its command; every command that prints the full sentence pushes or archives, so the Rust test is a unit test
- Leif decided "Leif via orc t18 (morning): approve the scope of stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship (no new intent criterion; MODIFIED REQ-cmd-change-029)": "Ok yes I approve all" (orc tab, 2026-11-06 06:1x MST, in reply to the orc's morning brief; relayed by orc t18, who recorded the native approval as 0xLeif); recorded by ss-s3 Claude agent; a relayed claim, not authenticated
- HI: #902 maps to SHIP-4.a (the answer names the stage and the one thing holding it); reported to orc t18 with the option to object

## What was verified

## Open threads

## Timeline

- 2026-11-06T07:04:07Z · gate-opened · intent → intent · Leif via orc t18 (morning): approve the scope of stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship (no new intent criterion; MODIFIED REQ-cmd-change-029) ^t-7a7413
  - native_change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
  - revision: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
- 2026-11-06T16:57:59Z · gate-closed · intent → intent · Leif via orc t18 (morning): approve the scope of stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship (no new intent criterion; MODIFIED REQ-cmd-change-029) ^t-e9247a
  - native_change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
  - revision: b0b2d578b783d1b53a15d9478971daa7991682da
  - relayed_by:
    - decided_by: Leif
    - words: Ok yes I approve all
    - where: orc tab, 2026-11-06 06:1x MST, in reply to the orc's morning brief; relayed by orc t18, who recorded the native approval as 0xLeif
    - recorded_by: ss-s3 Claude agent
- 2026-11-06T16:58:00Z · advance · intent → design ·  ^t-373408
  - native_change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
  - revision: b0b2d578b783d1b53a15d9478971daa7991682da
- 2026-11-06T16:58:00Z · advance · design → plan ·  ^t-de032c
  - native_change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
  - revision: b0b2d578b783d1b53a15d9478971daa7991682da
- 2026-11-06T16:58:01Z · advance · plan → build · native:approved ^t-dc6150
  - native_change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
  - gate: native:approved
  - revision: b0b2d578b783d1b53a15d9478971daa7991682da

Closed 2026-11-06: closed-without-done.

## Receipts

```yaml
receipts: []
```
