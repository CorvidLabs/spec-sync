---
kind: session
id: 2026-11-24-1163-lesson-file-follow-ups
aliases: [1163-lesson-file-follow-ups]
objective: "check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163)"
intent_scope: ""
modules: [cmd_check, hash_cache]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
pr: ""
criteria: [THREE-1]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Continue build
summary:
  - "Three review findings on #1163's lesson-file warning, each reproduced on the CLI: a cached check missed it, the file was named from the spec file rather than the module, and a directory of that name was skipped."
---
# check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163)

## Goal

check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file": "I approve all three" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The third was '#1163 successor: do you approve these requirement words?' and quoted them: REQ-cmd-check-023 amended (the file is the one named for the spec's module, an entry of that name that is not a regular file is reported as unreadable, and a strict lifecycle check fails on the warning as on any other, with three added acceptance criteria), REQ-hash-cache-011 added in full, and the validator Error Cases row. He had already chosen, earlier in the same chat, to fix the three findings in one successor change and keep change check --strict failing.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change approve check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-24T22:29:19Z · start · intent → intent · check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163) ^t-5e4a8a
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:01Z · advance · intent → design ·  ^t-46b427
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:01Z · skip · design → plan · Fixes to an existing warning and its cache input. No visuals or interactions. ^t-91def8
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:01Z · gate-opened · plan → plan · Scope approval of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file ^t-5defe1
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - holds: plan
  - drives: approve
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:04Z · direct-native-write · plan → plan · check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state draft → approved ^t-e2a9eb
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - direct_write:
    - change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: draft
    - after_state: approved
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:04Z · gate-closed · plan → plan · Scope approval of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file ^t-475c50
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - relayed_by:
    - decided_by: 0xLeif
    - words: I approve all three
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The third was '#1163 successor: do you approve these requirement words?' and quoted them: REQ-cmd-check-023 amended (the file is the one named for the spec's module, an entry of that name that is not a regular file is reported as unreadable, and a strict lifecycle check fails on the warning as on any other, with three added acceptance criteria), REQ-hash-cache-011 added in full, and the validator Error Cases row. He had already chosen, earlier in the same chat, to fix the three findings in one successor change and keep change check --strict failing.
    - recorded_by: claude-code-t14v
    - native: specsync change approve check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
    - actor: 0xLeif
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:08Z · advance · plan → build · native:approved ^t-5cb4ab
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - gate: native:approved
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
