---
kind: session
id: 2026-11-13-1003-ci-archive-check-docs
aliases: [1003-ci-archive-check-docs]
objective: Document change status --json as the supported CI check of one archived change, with its trust model, tested against the page's own check (#1003)
intent_scope: ""
modules: [cmd_change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
pr: ""
criteria: [CI-1]
requirements: []
cases: []
design: []
files:
  - site/src/content/docs/integrations/ci-archive-check.md
  - site/src/content/docs/integrations/github-action.md
  - tests/integration/archived_upstream.rs
observations: []
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
    holds: accept
    drives: review
    opened: 2026-11-13T18:37:52Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# Document change status --json as the supported CI check of one archived change, with its trust model, tested against the page's own check (#1003)

## Goal

Document change status --json as the supported CI check of one archived change, with its trust model, tested against the page's own check (#1003)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own approval ('#1003 CI archive check docs', coordinator chat 2026-11-13T03:28Z, relayed by claude-code-t370); REQ-cmd-change-038 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the change is narrow and an existing lesson covers it

## What was verified

## Open threads

## Timeline

- 2026-11-13T17:52:21Z · start · intent → intent · Document change status --json as the supported CI check of one archived change, with its trust model, tested against the page's own check (#1003) ^t-88786e
  - revision: 2d9f84b0412c98eb87fbe17d2d2a15de870e4a14
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:23Z · advance · intent → design ·  ^t-c19a87
  - native_change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
  - revision: 2d9f84b0412c98eb87fbe17d2d2a15de870e4a14
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:23Z · skip · design → plan · Docs, CI and CLI behaviour only: no visuals or interactions ^t-db4f18
  - native_change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
  - revision: 2d9f84b0412c98eb87fbe17d2d2a15de870e4a14
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:25Z · advance · plan → build · native:verifying ^t-20f95e
  - native_change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
  - gate: native:verifying
  - revision: 2d9f84b0412c98eb87fbe17d2d2a15de870e4a14
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:25Z · advance · build → test · native:verifying ^t-4ee716
  - native_change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
  - gate: native:verifying
  - revision: 2d9f84b0412c98eb87fbe17d2d2a15de870e4a14
  - by: claude-code (hc t38p)
- 2026-11-13T18:37:52Z · advance · test → accept · native:verifying ^t-a311eb
  - native_change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
  - gate: native:verifying
  - revision: 2d9f84b0412c98eb87fbe17d2d2a15de870e4a14
  - by: claude-code (hc t38p)
- 2026-11-13T18:37:52Z · gate-opened · accept → accept · Implementation review of document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model ^t-a3d40b
  - native_change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
  - revision: 2d9f84b0412c98eb87fbe17d2d2a15de870e4a14
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
