---
kind: session
id: 2026-11-19-cover-root-call-sites
aliases: [cover-root-call-sites]
objective: The six call sites that pass the project root to change 3's bounded reads are covered by a change, so the change audit passes (private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules: [cmd_init_registry, cmd_issues, cmd_score, cmd_view, mcp]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
pr: ""
criteria: []
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
    holds: accept
    drives: review
    opened: 2026-11-19T15:53:08Z
run_consents: []
next: Continue accept
title: Change 3's call sites covered
summary:
  - After change 3 was archived, the change audit named six call-site files that change 3's commit edited but its paths did not declare. This coverage-only change declares them; no code or contract changes.
  - "Verified: the audit passes with it; the full suites already ran on this code with change 3."
---
# The six call sites that pass the project root to change 3's bounded reads are covered by a change, so the change audit passes (private, with GHSA-7q5v-8jcc-rx54)

## Goal

The six call sites that pass the project root to change 3's bounded reads are covered by a change, so the change audit passes (private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads": "yes" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:50Z, after t4c described the coverage-only change (Cover the call sites that pass the project root to change 3's reads, --no-spec-change, exactly the six call-site files, no new code, already in the code he reviewed) and asked whether one yes covers both its scope approval and its implementation review; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-19T15:51:06Z · start · intent → intent · The six call sites that pass the project root to change 3's bounded reads are covered by a change, so the change audit passes (private, with GHSA-7q5v-8jcc-rx54) ^t-b985ae
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - by: claude-code (hc t5r)
- 2026-11-19T15:52:01Z · gate-opened · intent → intent · Scope approval of cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads ^t-24ce82
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-19T15:52:04Z · skip · intent → design · No new intent: covers six call sites of change 3, whose intent is SETUP-7 ^t-cfee82
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - by: claude-code (hc t5r)
- 2026-11-19T15:52:04Z · skip · design → plan · No visuals or interactions ^t-534843
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - by: claude-code (hc t5r)
- 2026-11-19T15:52:43Z · gate-closed · plan → plan · Scope approval of cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads ^t-70de20
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:50Z, after t4c described the coverage-only change (Cover the call sites that pass the project root to change 3's reads, --no-spec-change, exactly the six call-site files, no new code, already in the code he reviewed) and asked whether one yes covers both its scope approval and its implementation review; relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change approve cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T15:53:03Z · advance · plan → build · native:approved ^t-91cad4
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - gate: native:approved
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - by: claude-code (hc t5r)
- 2026-11-19T15:53:05Z · advance · build → test · native:approved ^t-5b9d42
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - gate: native:approved
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - by: claude-code (hc t5r)
- 2026-11-19T15:53:08Z · advance · test → accept · native:approved ^t-0a25b3
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - gate: native:approved
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - by: claude-code (hc t5r)
- 2026-11-19T15:53:08Z · gate-opened · accept → accept · Implementation review of cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads ^t-0ccbf6
  - native_change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
  - revision: 6d524c6c2f5c4579b81bd4acca85651834be45da
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
