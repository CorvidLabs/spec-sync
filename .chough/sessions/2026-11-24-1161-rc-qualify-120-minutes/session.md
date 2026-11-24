---
kind: session
id: 2026-11-24-1161-rc-qualify-120-minutes
aliases: [1161-rc-qualify-120-minutes]
objective: "v6.4.2 can qualify on macOS today: the release-candidate qualify job gets 120 minutes instead of 75, as a stopgap until the lane fits in 30 (#1161)"
intent_scope: SPEED
modules: []
stage: plan
stage_status: active
iteration: 1
callers: []
native_change: give-the-macos-release-candidate-qualification-120-minutes
criteria: [SPEED-2.b.1, SPEED-2.b.2]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Gate \"Scope approval of give-the-macos-release-candidate-qualification-120-minutes\" closed (approved); run `chough stage advance` to move on."
---
# v6.4.2 can qualify on macOS today: the release-candidate qualify job gets 120 minutes instead of 75, as a stopgap until the lane fits in 30 (#1161)

## Goal

v6.4.2 can qualify on macOS today: the release-candidate qualify job gets 120 minutes instead of 75, as a stopgap until the lane fits in 30 (#1161)

## Where it stands

- Drafted the native change (operations, no spec change): context, plan and testing written; ready reports nothing missing

## Decisions and why

- 0xLeif said 'yes raise it to 120 and cut rc.2' in the Claude Code release-lead chat on 2026-11-24 UTC, after v6.4.2-rc.1 was cancelled on macOS at the 75-minute limit with all 4,176 tests passed. He was asked to approve this scope: raise the qualify limit from 75 to 120 in .github/workflows/release.yml with a note in the 6.4.2 handoff, as its own small change, archived once its scoped check, the release contract tests, pre-push and Trust pass, with no further native Verify lane because no product, test or spec file changes. This moves away from SPEED-2.b.1 on purpose and for now. In the same chat he said he wants every test lane to have a maximum of about 30 minutes and does not want the limit raised again; that work is #1161 and is not part of this change. SPEED-2.b.2 holds: no test is dropped or weakened.
- 0xLeif decided "Scope approval of give-the-macos-release-candidate-qualification-120-minutes": "yes raise it to 120 and cut rc.2" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request that named this scope: raise the qualify limit from 75 to 120 in .github/workflows/release.yml, with a note in the 6.4.2 handoff, as its own small change, archived once its scoped check, the release contract tests, pre-push and Trust pass, with no further native Verify lane.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change approve give-the-macos-release-candidate-qualification-120-minutes as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-24T18:41:54Z · start · intent → intent · v6.4.2 can qualify on macOS today: the release-candidate qualify job gets 120 minutes instead of 75, as a stopgap until the lane fits in 30 (#1161) ^t-2e46f2
  - revision: b114d09fd1b102ddbb57dd4032769d6c854e5c52
  - by: claude-code (hc t14v)
- 2026-11-24T18:41:54Z · advance · intent → design ·  ^t-bf2b47
  - revision: b114d09fd1b102ddbb57dd4032769d6c854e5c52
  - by: claude-code (hc t14v)
- 2026-11-24T18:41:54Z · skip · design → plan · A workflow timeout value and a handoff note. No visuals or interactions. ^t-679a09
  - revision: b114d09fd1b102ddbb57dd4032769d6c854e5c52
  - by: claude-code (hc t14v)
- 2026-11-24T18:42:31Z · gate-opened · plan → plan · Scope approval of give-the-macos-release-candidate-qualification-120-minutes ^t-9ef8cb
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - revision: cfad11449c0d15535df538235fbafb47a7663829
  - holds: plan
  - drives: approve
  - by: claude-code (hc t14v)
- 2026-11-24T18:42:31Z · gate-closed · plan → plan · Scope approval of give-the-macos-release-candidate-qualification-120-minutes ^t-0a028e
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - revision: cfad11449c0d15535df538235fbafb47a7663829
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes raise it to 120 and cut rc.2
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request that named this scope: raise the qualify limit from 75 to 120 in .github/workflows/release.yml, with a note in the 6.4.2 handoff, as its own small change, archived once its scoped check, the release contract tests, pre-push and Trust pass, with no further native Verify lane.
    - recorded_by: claude-code-t14v
    - native: specsync change approve give-the-macos-release-candidate-qualification-120-minutes
    - actor: 0xLeif
  - by: claude-code (hc t14v)
