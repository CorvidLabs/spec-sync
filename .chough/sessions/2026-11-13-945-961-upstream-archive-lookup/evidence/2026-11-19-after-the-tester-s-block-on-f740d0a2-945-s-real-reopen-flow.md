---
kind: evidence
id: 2026-11-19-after-the-tester-s-block-on-f740d0a2-945-s-real-reopen-flow
aliases: []
claim: "After the tester's BLOCK on f740d0a2 (#945's real reopen flow still refused after an upstream rename): containment of an upstream archive is now decided by HEAD's history (log -1 --find-object=<state.json blob> HEAD -- :(top,literal)<archive root>, no renames; a Git failure is cannot-tell), not HEAD's tree. archived_upstream::a_reopened_change_reviews_after_the_remote_default_renames_its_archive runs the real flow (finalize, reopen, check --commit, upstream rename, review) with a bare origin plus a no-rename control; it fails on f740d0a2 with 'already archived on origin/main' at the rename commit and passes now. fledge lanes run verify at 0e9977c2 with GIT_CONFIG_GLOBAL=/dev/null: 10/10 steps, 3079 unit and 628 integration tests, strict specs 62/62, 100% file and LOC coverage; change audit --strict and pre-push passed."
result: passed
evidence_kind: product-test
revision: 0e9977c27bebf6b066fb1c09de3c038869e88f3b
content: 7e21a250e3b4d7e24e8a6ca016fbb3ee4891887f0b2ede8dce8fc0b00e983ff7
environment: local macOS 26, rust 1.89.0, branch fix/upstream-archive-945-961-965 at 0e9977c2
criteria: [SHIP-3.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T08:20:14Z
actor:
  - agent: claude-code
    terminal: t4t
---
**After the tester's BLOCK on f740d0a2 (#945's real reopen flow still refused after an upstream rename): containment of an upstream archive is now decided by HEAD's history (log -1 --find-object=<state.json blob> HEAD -- :(top,literal)<archive root>, no renames; a Git failure is cannot-tell), not HEAD's tree. archived_upstream::a_reopened_change_reviews_after_the_remote_default_renames_its_archive runs the real flow (finalize, reopen, check --commit, upstream rename, review) with a bare origin plus a no-rename control; it fails on f740d0a2 with 'already archived on origin/main' at the rename commit and passes now. fledge lanes run verify at 0e9977c2 with GIT_CONFIG_GLOBAL=/dev/null: 10/10 steps, 3079 unit and 628 integration tests, strict specs 62/62, 100% file and LOC coverage; change audit --strict and pre-push passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch fix/upstream-archive-945-961-965 at 0e9977c2 at revision 0e9977c27beb, observed 2026-11-19T08:20:14Z.
