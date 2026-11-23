---
kind: evidence
id: 2026-11-20-codex-s-third-round-on-1108-2-p1-3-p2-and-the-tester-s-excl2
aliases: []
claim: "Codex's third round on #1108 (2 P1, 3 P2) and the tester's excl2 at 4cdc28dd. Red first f6c3854c, fix 419a9449: a directory that leads outside the project, or is itself a link, is not read (P1); a REMOVED item counts only when its change's approval binds the delta by digest (P1), and through the binary the approval the refusal prints, run as printed, clears the refusal and the check passes (rule 3); a directory is left out of the presence scan only when both the working tree and the reference exclude it (P2 4638136052); the default branch's ids are read under its own configuration, so a newly excluded directory's loss names a restore that runs as printed, in both spellings of the exclusion (the tester's excl2). P2s 4638136059 and 4638136068 filed as #1112 and #1113. Real repos (rule 3): same results as the build before on all of them; starling PR #43 still refused with 26 ids. fledge lanes run verify passed at 0e4df637 through one-lane (3134 unit, 646 integration, 62/62, 100%); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 0e4df63710561e413fe02fafcc9687cf6452281d
content: 9da7a6ced8648a5c64b02d17698b38dd18619c69efcb8641a980ba8d52457008
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 0e4df637
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-20T03:57:20Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Codex's third round on #1108 (2 P1, 3 P2) and the tester's excl2 at 4cdc28dd. Red first f6c3854c, fix 419a9449: a directory that leads outside the project, or is itself a link, is not read (P1); a REMOVED item counts only when its change's approval binds the delta by digest (P1), and through the binary the approval the refusal prints, run as printed, clears the refusal and the check passes (rule 3); a directory is left out of the presence scan only when both the working tree and the reference exclude it (P2 4638136052); the default branch's ids are read under its own configuration, so a newly excluded directory's loss names a restore that runs as printed, in both spellings of the exclusion (the tester's excl2). P2s 4638136059 and 4638136068 filed as #1112 and #1113. Real repos (rule 3): same results as the build before on all of them; starling PR #43 still refused with 26 ids. fledge lanes run verify passed at 0e4df637 through one-lane (3134 unit, 646 integration, 62/62, 100%); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 0e4df637 at revision 0e4df6371056, observed 2026-11-20T03:57:20Z.
