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
observations:
  - 2026-11-20-1077-req-change-172-option-1-in-a-blobless-clone-whose-remote
  - 2026-11-20-1122-round-one-codex-2-p1-4-p2-corvid-agent-s-review-tester2-s
  - 2026-11-20-1122-codex-round-two-2-p2-plus-two-gaps-the-rule-3-runs-found
bugs: []
blockers: []
waiting_on_human:
  - gate: "Leif's review of #1077"
    holds: accept
    drives: review
    opened: 2026-11-20T06:14:32Z
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
- 0xLeif decided "Send-back of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them, run by the builder on Leif's word (an added path)": "Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)" (Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c); t4c confirmed this addition (src/change/validate_loaded_change.rs to #1077's affected paths, where the approved fetch runs: it adds no refusal and no wording) on 2026-11-20 at about 06:04Z, through ss-lead t58 (draft ss-lead/gates/1077-added-path-draft.md)); recorded by claude-code-t4t; a relayed claim, not authenticated
- 0xLeif decided "Re-approval of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them (an added path)": "Fetch it itself (Recommended); yes to all 3; Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)" (REQ-change-172 option 1: a choice in a question dialog in the tester's (hc t4p) Claude Code session, 2026-11-20 at about 00:45Z ('Fetch it itself (Recommended)'), relayed by the tester to his planning session hc t4c and by t4c through ss-lead t58, and 'yes to all 3', a Claude Code chat message to hc t4c at about 01:00Z, relayed through ss-lead t58, for the two details. The added path src/change/validate_loaded_change.rs: Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c), confirmed by t4c on 2026-11-20 at about 06:04Z through ss-lead t58); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them as 0xLeif

## What was verified

- [#1077 (REQ-change-172 option 1), in a blobless clone whose remote resolves only through the user's global insteadOf: change status, audit and ship-status succeed after one fetch announced by one line, and nothing under .specsync and specs is missing afterwards; a full clone makes no fetch; with the user's configuration gone the refusal says the clone is partial and names a git -C … fetch … <oid> … that, run as printed, lets the command pass; a blobless clone on main audits #1081's base-commit requirements after the one fetch. Real repos (rule 3): the blobless HTTPS clone of private peck: this build downloads 377 objects once and then gives the full clone's 85 errors with no 'cannot tell' (the build before gave 11). Added path src/change/validate_loaded_change.rs re-approved under Leif's standing OK (t4c, about 06:04Z). Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb.](evidence/2026-11-20-1077-req-change-172-option-1-in-a-blobless-clone-whose-remote.md)
- 2026-11-20-1122-round-one-codex-2-p1-4-p2-corvid-agent-s-review-tester2-s
- [#1122 Codex round two (2 P2) plus two gaps the rule-3 runs found in the same download. Red first fe275bdb, 24d64637 and ed3495d5; fixes 158e3654, f4617e03 and 2917f7d6: the one fetch lists .specsync.toml, specsync.json and specsync-registry.toml beside .specsync; the printed recovery is printf '%s\n' <oid> ... | git -C <root> fetch ... --stdin -- <remote> (the ids in the line, none in Git's arguments); a failed download gives the loss check's way out as that fetch, not git fetch origin; the listing walks with --full-history. Lane at 7a571bbe through one-lane (3137 unit, 664 integration, 62/62, 100%); change audit --strict exit 0. Real repos, download failing: private peck audit refuses naming 377 ids in a 15.7 KB line, run as printed rc 0, then the same 85 errors as its full clone; spec-sync with its remote unreachable, audit --archives refuses naming 3329 ids in a 137 KB line, run as printed rc 0, then 0 errors as its full clone. With the user's config: spec-sync blobless audit 42.1 s on 6.3.0 against 1.9 s with one fetch of 3329 objects; peck one fetch of 377 then its 85 errors.](evidence/2026-11-20-1122-codex-round-two-2-p2-plus-two-gaps-the-rule-3-runs-found.md)

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
- 2026-11-20T06:13:49Z · gate-closed · accept → accept · Leif's review of #1077 ^t-f55eae
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - by: claude-code (hc t4t)
- 2026-11-20T06:13:49Z · gate-opened · accept → accept · Send-back of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them, run by the builder on Leif's word (an added path) ^t-458811
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - holds: accept
  - drives: none
  - by: claude-code (hc t4t)
- 2026-11-20T06:13:50Z · direct-native-write · accept → accept · download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them's lifecycle records changed outside Chough: state.json changed, verification.json changed, verification-attempts.json changed; state stays verifying ^t-28e5cb
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - direct_write:
    - change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - files:
      - state.json: changed
      - verification.json: changed
      - verification-attempts.json: changed
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-20T06:13:50Z · gate-closed · accept → accept · Send-back of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them, run by the builder on Leif's word (an added path) ^t-caa1c7
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)
    - where: Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c); t4c confirmed this addition (src/change/validate_loaded_change.rs to #1077's affected paths, where the approved fetch runs: it adds no refusal and no wording) on 2026-11-20 at about 06:04Z, through ss-lead t58 (draft ss-lead/gates/1077-added-path-draft.md)
    - recorded_by: claude-code-t4t
  - by: claude-code (hc t4t)
- 2026-11-20T06:13:51Z · direct-native-write · accept → accept · download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state verifying → draft ^t-c87658
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - direct_write:
    - change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: verifying
    - after_state: draft
  - by: claude-code (hc t4t)
- 2026-11-20T06:13:51Z · back · accept → plan · #1077 gains one affected path under Leif's standing OK; re-approval follows at plan ^t-7e9f9c
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:09Z · gate-opened · plan → plan · Re-approval of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them (an added path) ^t-6d753a
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:09Z · direct-native-write · plan → plan · download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-c53f57
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - direct_write:
    - change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:09Z · gate-closed · plan → plan · Re-approval of download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them (an added path) ^t-395a2e
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: c783e8cf739986777c7f73e1232b640da1c3b018
  - relayed_by:
    - decided_by: 0xLeif
    - words: Fetch it itself (Recommended); yes to all 3; Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)
    - where: REQ-change-172 option 1: a choice in a question dialog in the tester's (hc t4p) Claude Code session, 2026-11-20 at about 00:45Z ('Fetch it itself (Recommended)'), relayed by the tester to his planning session hc t4c and by t4c through ss-lead t58, and 'yes to all 3', a Claude Code chat message to hc t4c at about 01:00Z, relayed through ss-lead t58, for the two details. The added path src/change/validate_loaded_change.rs: Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c), confirmed by t4c on 2026-11-20 at about 06:04Z through ss-lead t58
    - recorded_by: claude-code-t4t
    - native: specsync change approve download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:26Z · direct-native-write · plan → plan · download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them's lifecycle records changed outside Chough: state.json changed, verification.json changed, verification-attempts.json changed; state approved → verifying ^t-9deb2a
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
  - direct_write:
    - change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
    - files:
      - state.json: changed
      - verification.json: changed
      - verification-attempts.json: changed
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:26Z · advance · plan → build · native:verifying ^t-8bad77
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - gate: native:verifying
  - revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:29Z · advance · build → test · native:verifying ^t-6f373e
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - gate: native:verifying
  - revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:32Z · advance · test → accept · native:verifying ^t-2f68a1
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - gate: native:verifying
  - revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
  - by: claude-code (hc t4t)
- 2026-11-20T06:14:32Z · gate-opened · accept → accept · Leif's review of #1077 ^t-258387
  - native_change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
  - revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
