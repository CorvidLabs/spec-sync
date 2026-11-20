---
kind: session
id: 2026-11-20-909-safe-directory
aliases: [909-safe-directory]
objective: When SpecSync's isolated Git refuses a repository for dubious ownership that the user's own Git trusts, pass safe.directory for its top level through a private temporary global file; otherwise refuse naming the git config command (#909, REQ-change-171; Leif's dialog choice of 2026-11-20 about 00:45Z, relayed via t4p, t4c and ss-lead t58)
intent_scope: CHANGE
modules: [change]
stage: plan
stage_status: active
iteration: 1
callers: []
native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
pr: ""
criteria: [CHANGE-7]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Gate \"Scope approval of pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user\" closed (approved); run `chough stage advance` to move on."
---
# When SpecSync's isolated Git refuses a repository for dubious ownership that the user's own Git trusts, pass safe.directory for its top level through a private temporary global file; otherwise refuse naming the git config command (#909, REQ-change-171; Leif's dialog choice of 2026-11-20 about 00:45Z, relayed via t4p, t4c and ss-lead t58)

## Goal

When SpecSync's isolated Git refuses a repository for dubious ownership that the user's own Git trusts, pass safe.directory for its top level through a private temporary global file; otherwise refuse naming the git config command (#909, REQ-change-171; Leif's dialog choice of 2026-11-20 about 00:45Z, relayed via t4p, t4c and ss-lead t58)

## Where it stands

## Decisions and why

- No new intent criterion: the approved definition proposes none. CHANGE-7 (be told the state and the single next thing to do) already covers reading a repository the user's Git trusts, and naming the command when it does not.
- 0xLeif decided "Scope approval of pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user": "Yes, temp file (Recommended)" (a choice in a question dialog in the tester's (hc t4p) Claude Code session, 2026-11-20 at about 00:45Z, relayed by the tester to his planning session hc t4c and by t4c through ss-lead t58 (a dialog choice, not a chat message), on ss-lead/gates/git-environments-909-1072-1077-definition.md: choice 2 (#909: the trust setting goes through a private temporary file, not the command line)); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-20T05:13:02Z · start · intent → intent · When SpecSync's isolated Git refuses a repository for dubious ownership that the user's own Git trusts, pass safe.directory for its top level through a private temporary global file; otherwise refuse naming the git config command (#909, REQ-change-171; Leif's dialog choice of 2026-11-20 about 00:45Z, relayed via t4p, t4c and ss-lead t58) ^t-17bdd6
  - revision: c076583c09ff9acbe529426922a3e225eed42f56
  - by: claude-code (hc t4t)
- 2026-11-20T05:14:29Z · direct-native-write · intent → intent · pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-e9b9c5
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - revision: c076583c09ff9acbe529426922a3e225eed42f56
  - direct_write:
    - change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t4t)
- 2026-11-20T05:14:29Z · advance · intent → design ·  ^t-3aabf4
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - revision: c076583c09ff9acbe529426922a3e225eed42f56
  - by: claude-code (hc t4t)
- 2026-11-20T05:14:29Z · skip · design → plan · Git configuration and message behaviour only: no visuals or interactions ^t-adafeb
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - revision: c076583c09ff9acbe529426922a3e225eed42f56
  - by: claude-code (hc t4t)
- 2026-11-20T05:14:29Z · gate-opened · plan → plan · Scope approval of pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user ^t-16d359
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - revision: c076583c09ff9acbe529426922a3e225eed42f56
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-20T05:14:29Z · gate-closed · plan → plan · Scope approval of pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user ^t-71901e
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - revision: c076583c09ff9acbe529426922a3e225eed42f56
  - relayed_by:
    - decided_by: 0xLeif
    - words: Yes, temp file (Recommended)
    - where: a choice in a question dialog in the tester's (hc t4p) Claude Code session, 2026-11-20 at about 00:45Z, relayed by the tester to his planning session hc t4c and by t4c through ss-lead t58 (a dialog choice, not a chat message), on ss-lead/gates/git-environments-909-1072-1077-definition.md: choice 2 (#909: the trust setting goes through a private temporary file, not the command line)
    - recorded_by: claude-code-t4t
    - native: specsync change approve pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
    - actor: 0xLeif
  - by: claude-code (hc t4t)
