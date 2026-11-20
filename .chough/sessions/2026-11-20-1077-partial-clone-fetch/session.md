---
kind: session
id: 2026-11-20-1077-partial-clone-fetch
aliases: [1077-partial-clone-fetch]
objective: In a partial clone, SpecSync downloads every history object its reads need in one fetch run with the user's own Git, never prompting and bounded at 60 s, and refuses naming one git fetch when that fails (#1077, REQ-change-172 option 1; Leif's dialog choice of 2026-11-20 about 00:45Z and 'yes to all 3' about 01:00Z, relayed via t4c and ss-lead t58)
intent_scope: CHANGE
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
pr: ""
criteria: [CHANGE-7]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human:
  - gate: "Leif's review of #1077"
    holds: accept
    drives: review
    opened: 2026-11-20T05:50:07Z
run_consents: []
next: Continue accept
---
# In a partial clone, SpecSync downloads every history object its reads need in one fetch run with the user's own Git, never prompting and bounded at 60 s, and refuses naming one git fetch when that fails (#1077, REQ-change-172 option 1; Leif's dialog choice of 2026-11-20 about 00:45Z and 'yes to all 3' about 01:00Z, relayed via t4c and ss-lead t58)

## Goal

In a partial clone, SpecSync downloads every history object its reads need in one fetch run with the user's own Git, never prompting and bounded at 60 s, and refuses naming one git fetch when that fails (#1077, REQ-change-172 option 1; Leif's dialog choice of 2026-11-20 about 00:45Z and 'yes to all 3' about 01:00Z, relayed via t4c and ss-lead t58)

## Where it stands

## Decisions and why

- No new intent criterion: the approved definition proposes none. CHANGE-7 (be told the state and the single next thing to do) covers reading a partial clone's history, and naming one fetch when the download fails.
- 0xLeif decided "Scope approval of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them": "Fetch it itself (Recommended); yes to all 3" (option 1: a choice in a question dialog in the tester's (hc t4p) Claude Code session, 2026-11-20 at about 00:45Z, relayed by the tester to his planning session hc t4c and by t4c through ss-lead t58 (a dialog choice, not a chat message), on ss-lead/gates/git-environments-909-1072-1077-definition.md, choice 1 (#1077). The two details under option 1 (every command that reads history fetches, change status, audit and ship-status included, and CI fetches the same way): 'yes to all 3', a Claude Code chat message to his planning session hc t4c, 2026-11-20 at about 01:00Z, relayed through ss-lead t58, answering t4c's item 2: '#1077: should every command that reads Git history fetch what it needs, CI included?'); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-20T05:16:42Z · start · intent → intent · In a partial clone, SpecSync downloads every history object its reads need in one fetch run with the user's own Git, never prompting and bounded at 60 s, and refuses naming one git fetch when that fails (#1077, REQ-change-172 option 1; Leif's dialog choice of 2026-11-20 about 00:45Z and 'yes to all 3' about 01:00Z, relayed via t4c and ss-lead t58) ^t-c3250d
  - revision: bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e
  - by: claude-code (hc t4t)
- 2026-11-20T05:18:09Z · direct-native-write · intent → intent · download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-dc5e4b
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e
  - direct_write:
    - change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t4t)
- 2026-11-20T05:18:09Z · advance · intent → design ·  ^t-d186c9
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e
  - by: claude-code (hc t4t)
- 2026-11-20T05:18:09Z · skip · design → plan · Git fetch and message behaviour only: no visuals or interactions ^t-361250
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e
  - by: claude-code (hc t4t)
- 2026-11-20T05:18:09Z · gate-opened · plan → plan · Scope approval of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them ^t-e3dd56
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-20T05:18:10Z · gate-closed · plan → plan · Scope approval of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them ^t-fcff02
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e
  - relayed_by:
    - decided_by: 0xLeif
    - words: Fetch it itself (Recommended); yes to all 3
    - where: option 1: a choice in a question dialog in the tester's (hc t4p) Claude Code session, 2026-11-20 at about 00:45Z, relayed by the tester to his planning session hc t4c and by t4c through ss-lead t58 (a dialog choice, not a chat message), on ss-lead/gates/git-environments-909-1072-1077-definition.md, choice 1 (#1077). The two details under option 1 (every command that reads history fetches, change status, audit and ship-status included, and CI fetches the same way): 'yes to all 3', a Claude Code chat message to his planning session hc t4c, 2026-11-20 at about 01:00Z, relayed through ss-lead t58, answering t4c's item 2: '#1077: should every command that reads Git history fetch what it needs, CI included?'
    - recorded_by: claude-code-t4t
    - native: specsync change approve download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:04Z · direct-native-write · plan → plan · download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them's lifecycle records changed outside Chough: state.json changed, verification.json added, verification-attempts.json added; state approved → verifying ^t-0dadbb
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - direct_write:
    - change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - files:
      - state.json: changed
      - verification.json: added
      - verification-attempts.json: added
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:04Z · advance · plan → build · native:verifying ^t-cd1731
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - gate: native:verifying
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:06Z · advance · build → test · native:verifying ^t-282872
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - gate: native:verifying
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:07Z · advance · test → accept · native:verifying ^t-29d00c
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - gate: native:verifying
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:07Z · gate-opened · accept → accept · Leif's review of #1077 ^t-a9e0ee
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
