---
kind: evidence
id: 2026-11-19-covers-req-change-161-req-change-122-and-req-cmd-change-038-a5d044
aliases: []
claim: "Covers REQ-change-161, REQ-change-122 and REQ-cmd-change-038 (the last not linked: chough 0.6.0 looks up specs/cmd-change). After the tester's regression on 0d31218a, Codex's second #1089 review and ss-lead's F1/F2, order and #654 reviews: main's block is read at HEAD's merge-base with the default branch on its first-parent line; a rebase happens when the working tree holds main's block, the delta's body or their merge; a merge-base block equal to the delta's body is the change's own text; records name main's commit and readers check it; an applied requirement main edited that cannot be rebased is refused; text status names a refused record; a word one side deleted is never put back; 256 KiB and 64-record bounds; the hint names a module with a stale requirement. The #654 squash-merge test fails without its fix; F1, F2, (c), (d), (e) tests fail on the old code. Re-approved under Leif's relayed 'Approve' (round 3). fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3100 unit and 633 integration tests, strict specs 62/62; change audit --strict, fledge trust verify and pre-push passed."
result: passed
evidence_kind: product-test
revision: 985de94004f440d5bbfe5d40f3d24e8d3c2b5abe
content: 34671f2b2e667b79f12921b2e20a45a6f46aa96526e9487bd79b46e7473e593d
environment: local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 985de940
criteria: [CHANGE-6.a, CHANGE-6.c]
requirements: [REQ-change-161, REQ-change-122]
contracts: []
limitations: []
observed: 2026-11-19T05:56:26Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Covers REQ-change-161, REQ-change-122 and REQ-cmd-change-038 (the last not linked: chough 0.6.0 looks up specs/cmd-change). After the tester's regression on 0d31218a, Codex's second #1089 review and ss-lead's F1/F2, order and #654 reviews: main's block is read at HEAD's merge-base with the default branch on its first-parent line; a rebase happens when the working tree holds main's block, the delta's body or their merge; a merge-base block equal to the delta's body is the change's own text; records name main's commit and readers check it; an applied requirement main edited that cannot be rebased is refused; text status names a refused record; a word one side deleted is never put back; 256 KiB and 64-record bounds; the hint names a module with a stale requirement. The #654 squash-merge test fails without its fix; F1, F2, (c), (d), (e) tests fail on the old code. Re-approved under Leif's relayed 'Approve' (round 3). fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3100 unit and 633 integration tests, strict specs 62/62; change audit --strict, fledge trust verify and pre-push passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 985de940 at revision 985de94004f4, observed 2026-11-19T05:56:26Z.
