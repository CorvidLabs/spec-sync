---
kind: session
id: 2026-11-20-folder-walks-and-read-cache
aliases: [folder-walks-and-read-cache]
objective: Every walk of a configured folder goes through the confined walk, so a committed specs, source or schema link that leaves the project is refused, not walked; the change read cache lives in the user's cache folder, never in a Git directory an archive can ship (items 2 and 4 of Leif's 04:09Z round, private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules:
  - util
  - change
  - validator
  - config
  - schema
  - commands
  - cmd_scaffold
  - registry
  - cmd_wizard
  - mcp
  - cmd_check
stage: build
stage_status: active
iteration: 1
callers: []
native_change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
pr: ""
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
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
---
# Every walk of a configured folder goes through the confined walk, so a committed specs, source or schema link that leaves the project is refused, not walked; the change read cache lives in the user's cache folder, never in a Git directory an archive can ship (items 2 and 4 of Leif's 04:09Z round, private, with GHSA-7q5v-8jcc-rx54)

## Goal

Every walk of a configured folder goes through the confined walk, so a committed specs, source or schema link that leaves the project is refused, not walked; the change read cache lives in the user's cache folder, never in a Git directory an archive can ship (items 2 and 4 of Leif's 04:09Z round, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder": "continue with 95% confidence" (Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 04:09Z, answering t4c's five-item list with t4c's recommendations; t4c read it as yes to items 1-4, among them item 2 'Fold the folder-walk fix into change 7?' (the ~9 configured-folder walks routed through the safe helper so a linked folder is not followed out) and item 4 'Move the history-verdict cache out of .git too, in change 7?' (read_cache.rs to the user cache folder, same LocalState pattern). Under Leif's delegation of 2026-11-20 (about 04:11Z: process calls decided by t4c on his behalf), decided by t4c: change 8 on top of change 7, no send-back. Relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-20T04:18:57Z · start · intent → intent · Every walk of a configured folder goes through the confined walk, so a committed specs, source or schema link that leaves the project is refused, not walked; the change read cache lives in the user's cache folder, never in a Git directory an archive can ship (items 2 and 4 of Leif's 04:09Z round, private, with GHSA-7q5v-8jcc-rx54) ^t-92c52b
  - revision: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
  - by: claude-code (hc t5r)
- 2026-11-20T04:18:57Z · advance · intent → design ·  ^t-16a1bd
  - revision: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
  - by: claude-code (hc t5r)
- 2026-11-20T04:18:57Z · advance · design → plan ·  ^t-25bedb
  - revision: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
  - by: claude-code (hc t5r)
- 2026-11-20T04:20:32Z · gate-opened · plan → plan · Scope approval of walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder ^t-4cb1ab
  - native_change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
  - revision: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-20T04:20:32Z · direct-native-write · plan → plan · walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-b3a1e9
  - native_change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
  - revision: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
  - direct_write:
    - change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t5r)
- 2026-11-20T04:20:32Z · gate-closed · plan → plan · Scope approval of walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder ^t-5669a8
  - native_change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
  - revision: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
  - relayed_by:
    - decided_by: 0xLeif
    - words: continue with 95% confidence
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 04:09Z, answering t4c's five-item list with t4c's recommendations; t4c read it as yes to items 1-4, among them item 2 'Fold the folder-walk fix into change 7?' (the ~9 configured-folder walks routed through the safe helper so a linked folder is not followed out) and item 4 'Move the history-verdict cache out of .git too, in change 7?' (read_cache.rs to the user cache folder, same LocalState pattern). Under Leif's delegation of 2026-11-20 (about 04:11Z: process calls decided by t4c on his behalf), decided by t4c: change 8 on top of change 7, no send-back. Relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change approve walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T04:21:04Z · advance · plan → build · native:approved ^t-cc2dd6
  - native_change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
  - gate: native:approved
  - revision: 4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b
  - by: claude-code (hc t5r)
