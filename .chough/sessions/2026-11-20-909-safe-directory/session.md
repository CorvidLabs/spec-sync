---
kind: session
id: 2026-11-20-909-safe-directory
aliases: [909-safe-directory]
objective: When SpecSync's isolated Git refuses a repository for dubious ownership that the user's own Git trusts, pass safe.directory for its top level through a private temporary global file; otherwise refuse naming the git config command (#909, REQ-change-171; Leif's dialog choice of 2026-11-20 about 00:45Z, relayed via t4p, t4c and ss-lead t58)
intent_scope: CHANGE
modules: [change]
stage: accept
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
observations:
  - 2026-11-20-909-req-change-171-with-git-s-git-test-assume-different-owner
  - 2026-11-20-1122-round-one-codex-2-p1-4-p2-corvid-agent-s-review-tester2-s
bugs: []
blockers: []
waiting_on_human:
  - gate: "Leif's review of #909"
    holds: accept
    drives: review
    opened: 2026-11-20T05:50:03Z
run_consents: []
next: Continue accept
---
# When SpecSync's isolated Git refuses a repository for dubious ownership that the user's own Git trusts, pass safe.directory for its top level through a private temporary global file; otherwise refuse naming the git config command (#909, REQ-change-171; Leif's dialog choice of 2026-11-20 about 00:45Z, relayed via t4p, t4c and ss-lead t58)

## Goal

When SpecSync's isolated Git refuses a repository for dubious ownership that the user's own Git trusts, pass safe.directory for its top level through a private temporary global file; otherwise refuse naming the git config command (#909, REQ-change-171; Leif's dialog choice of 2026-11-20 about 00:45Z, relayed via t4p, t4c and ss-lead t58)

## Where it stands

## Decisions and why

- No new intent criterion: the approved definition proposes none. CHANGE-7 (be told the state and the single next thing to do) already covers reading a repository the user's Git trusts, and naming the command when it does not.
- 0xLeif decided "Scope approval of pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user": "Yes, temp file (Recommended)" (a choice in a question dialog in the tester's (hc t4p) Claude Code session, 2026-11-20 at about 00:45Z, relayed by the tester to his planning session hc t4c and by t4c through ss-lead t58 (a dialog choice, not a chat message), on ss-lead/gates/git-environments-909-1072-1077-definition.md: choice 2 (#909: the trust setting goes through a private temporary file, not the command line)); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user as 0xLeif

## What was verified

- [#909 (REQ-change-171), with Git's GIT_TEST_ASSUME_DIFFERENT_OWNER: a repository the user's global file trusts (its top level, or safe.directory = *) passes change status, audit and check, also from a linked worktree; the private file is gone after each command and none is made for a repository the user owns; without the entry the refusal names dubious ownership, the path and git config --global --add safe.directory. Real repos (rule 3): Chough main and spec-sync main with the switch: 6.3.0 refuses even when trusted ('unable to inspect changed paths'), this build passes when trusted and names the command when not; without the switch both pass. Not run: the container runs with a real second owner (Debian 12, Alpine 3.16, Ubuntu 22.04). Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb.](evidence/2026-11-20-909-req-change-171-with-git-s-git-test-assume-different-owner.md)
- [#1122 round one (Codex 2 P1 + 4 P2, corvid-agent's review, tester2's Linux run). Red first da1b8d91 and fd7f1aa3, fixes 647c58af and 7efd6e73: every history read in run_git_bounded triggers the one partial-clone fetch (audit with no active change: one fetch); the trust probe and the fetch are rooted (GIT_DIR and the rest removed) and the probe counts only for this repository's top level; stderr bounded and never joined, the fetch's session killed on timeout; gitdir and commondir in the owner check and the file's placement; exact top-level bytes; Git's boolean spellings; '--' before the remote; trust tests set GIT_CONFIG_NOSYSTEM. Lane at 7ad4ff5d through one-lane (3137 unit, 660 integration, 62/62, 100%); change audit --strict exit 0. Real repos: spec-sync fresh blobless audit 42.6 s (6.3.0) against 2.3 s with one fetch of 3245 objects; private peck blobless 85 errors as before after one fetch of 377 objects; GIT_DIR naming a trusted sibling still refuses.](evidence/2026-11-20-1122-round-one-codex-2-p1-4-p2-corvid-agent-s-review-tester2-s.md)

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
- 2026-11-20T05:50:00Z · direct-native-write · plan → plan · pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user's lifecycle records changed outside Chough: state.json changed, verification.json added, verification-attempts.json added; state approved → verifying ^t-288cbd
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - direct_write:
    - change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
    - files:
      - state.json: changed
      - verification.json: added
      - verification-attempts.json: added
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:00Z · advance · plan → build · native:verifying ^t-26ac77
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - gate: native:verifying
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:01Z · advance · build → test · native:verifying ^t-3b7760
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - gate: native:verifying
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:03Z · advance · test → accept · native:verifying ^t-b08598
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - gate: native:verifying
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - by: claude-code (hc t4t)
- 2026-11-20T05:50:03Z · gate-opened · accept → accept · Leif's review of #909 ^t-3e1d7e
  - native_change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
  - revision: 0921a371f9091b12aac13cfbf07cceb0e11f35b8
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
