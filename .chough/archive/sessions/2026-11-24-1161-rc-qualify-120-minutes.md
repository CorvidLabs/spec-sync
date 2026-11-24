---
kind: session
id: 2026-11-24-1161-rc-qualify-120-minutes
aliases: [1161-rc-qualify-120-minutes]
objective: "v6.4.2 can qualify on macOS today: the release-candidate qualify job gets 120 minutes instead of 75, as a stopgap until the lane fits in 30 (#1161)"
intent_scope: SPEED
stage: done
stage_status: archived
iteration: 1
native_change: give-the-macos-release-candidate-qualification-120-minutes
criteria: [SPEED-2.b.1, SPEED-2.b.2]
observations: [2026-11-24-1161-contract-suites-pass-on-the-edit]
title: "6.4.2: 120 minutes for the macOS candidate lane"
summary:
  - v6.4.2-rc.1 passed all 4,176 tests on macOS and was cancelled at the 75-minute qualify limit, so the limit is 120 for v6.4.2-rc.2. One value in release.yml and a handoff note; no product, test or spec file changes.
  - The validators and all six contract suites pass, the parsed workflow is otherwise identical to main, and scoped check, pre-push and Trust pass.
  - "Next: bring every test lane under 30 minutes (#1161)."
closed: 2026-11-24
outcome: done
---
# v6.4.2 can qualify on macOS today: the release-candidate qualify job gets 120 minutes instead of 75, as a stopgap until the lane fits in 30 (#1161)

## Goal

v6.4.2 can qualify on macOS today: the release-candidate qualify job gets 120 minutes instead of 75, as a stopgap until the lane fits in 30 (#1161)

## Where it stands

- Drafted the native change (operations, no spec change): context, plan and testing written; ready reports nothing missing
- Edited the qualify limit to 120 with its comment and added the handoff paragraph; the parsed workflow is otherwise identical

## Decisions and why

- 0xLeif said 'yes raise it to 120 and cut rc.2' in the Claude Code release-lead chat on 2026-11-24 UTC, after v6.4.2-rc.1 was cancelled on macOS at the 75-minute limit with all 4,176 tests passed. He was asked to approve this scope: raise the qualify limit from 75 to 120 in .github/workflows/release.yml with a note in the 6.4.2 handoff, as its own small change, archived once its scoped check, the release contract tests, pre-push and Trust pass, with no further native Verify lane because no product, test or spec file changes. This moves away from SPEED-2.b.1 on purpose and for now. In the same chat he said he wants every test lane to have a maximum of about 30 minutes and does not want the limit raised again; that work is #1161 and is not part of this change. SPEED-2.b.2 holds: no test is dropped or weakened.
- 0xLeif decided "Scope approval of give-the-macos-release-candidate-qualification-120-minutes": "yes raise it to 120 and cut rc.2" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request that named this scope: raise the qualify limit from 75 to 120 in .github/workflows/release.yml, with a note in the 6.4.2 handoff, as its own small change, archived once its scoped check, the release contract tests, pre-push and Trust pass, with no further native Verify lane.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change approve give-the-macos-release-candidate-qualification-120-minutes as 0xLeif
- 0xLeif decided "Implementation review of give-the-macos-release-candidate-qualification-120-minutes": "yes raise it to 120 and cut rc.2" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request that named this scope and said the change would be archived once its scoped check, the release contract tests, pre-push and Trust pass. They passed at ffca9ef0a6e5147522a7db883b1a56103d18d25e.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change review give-the-macos-release-candidate-qualification-120-minutes as 0xLeif with verdict pass

## What was verified

- With timeout-minutes for the qualify job at 120: validate-release-version and validate-workflow-runtime-pins pass, test-validate-release-candidate passes 67 tests, test-required-ci-gate 38, test-linux-baseline 35, test-ci-trigger-paths 3, test-ci-attest-and-summary 9, test-clippy-all-targets 5, test-classify-ci-paths passes, and actionlint reports nothing. Parsed as YAML the workflow is identical to main apart from that one value. No product, test or spec file is changed, and no test is dropped or weakened.; limits: Structural only: whether 120 minutes is enough is shown by the v6.4.2-rc.2 qualification, not here ^2026-11-24-1161-contract-suites-pass-on-the-edit

## Bugs

None.

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
- 2026-11-24T18:42:34Z · advance · plan → build · native:approved ^t-d2de6f
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - gate: native:approved
  - revision: 1dfd0dbf5cd9eed09cfbc5ef49b3351cc696646c
  - by: claude-code (hc t14v)
- 2026-11-24T18:43:23Z · advance · build → test · native:approved ^t-290c91
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - gate: native:approved
  - revision: 1dfd0dbf5cd9eed09cfbc5ef49b3351cc696646c
  - by: claude-code (hc t14v)
- 2026-11-24T18:43:24Z · advance · test → accept · native:approved ^t-954b0f
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - gate: native:approved
  - revision: 1dfd0dbf5cd9eed09cfbc5ef49b3351cc696646c
  - by: claude-code (hc t14v)
- 2026-11-24T18:43:39Z · gate-opened · accept → accept · Implementation review of give-the-macos-release-candidate-qualification-120-minutes ^t-d59e72
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - revision: 1dfd0dbf5cd9eed09cfbc5ef49b3351cc696646c
  - holds: accept
  - drives: review
  - by: claude-code (hc t14v)
- 2026-11-24T18:47:25Z · gate-closed · accept → accept · Implementation review of give-the-macos-release-candidate-qualification-120-minutes ^t-878243
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - revision: 0bf8e8dcc50446818c0386a013e3f564e782097a
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes raise it to 120 and cut rc.2
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request that named this scope and said the change would be archived once its scoped check, the release contract tests, pre-push and Trust pass. They passed at ffca9ef0a6e5147522a7db883b1a56103d18d25e.
    - recorded_by: claude-code-t14v
    - native: specsync change review give-the-macos-release-candidate-qualification-120-minutes
    - actor: 0xLeif
  - by: claude-code (hc t14v)
- 2026-11-24T18:47:27Z · advance · accept → done · native:archived ^t-0c9d28
  - native_change: give-the-macos-release-candidate-qualification-120-minutes
  - gate: native:archived
  - revision: 0bf8e8dcc50446818c0386a013e3f564e782097a
  - by: claude-code (hc t14v)

Closed 2026-11-24: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-24-1161-contract-suites-pass-on-the-edit
    claim: "With timeout-minutes for the qualify job at 120: validate-release-version and validate-workflow-runtime-pins pass, test-validate-release-candidate passes 67 tests, test-required-ci-gate 38, test-linux-baseline 35, test-ci-trigger-paths 3, test-ci-attest-and-summary 9, test-clippy-all-targets 5, test-classify-ci-paths passes, and actionlint reports nothing. Parsed as YAML the workflow is identical to main apart from that one value. No product, test or spec file is changed, and no test is dropped or weakened."
    result: passed
    revision: 1dfd0dbf5cd9eed09cfbc5ef49b3351cc696646c
    environment: macos-arm64-local
    content: 0c8817fa7b1f6660deddd16db860c1c3dfd48732718cba644034a7f93597f5a5
    criteria: SPEED-2.b.2
    limitations: "Structural only: whether 120 minutes is enough is shown by the v6.4.2-rc.2 qualification, not here"
    observed: 2026-11-24T18:43:23Z
    actor: claude-code (hc t14v)
```
