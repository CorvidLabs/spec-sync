---
kind: evidence
id: 2026-11-20-codex-s-fourth-round-on-1108-3-p1-fixed-p2s-filed-as-1116-and
aliases: []
claim: "Codex's fourth round on #1108 (3 P1 fixed; P2s filed as #1116 and #1117) and the tester's outside specs directory, on Leif's 'continue with 95% confidence' (2026-11-20 about 04:09Z). REQ-change-169 sent back and re-approved with one replacement, 'in none of the tracked ones in the working tree' (e36fc35e). Red first 86add4bc (four tests, one through the binary), fix b8538c70: only files Git tracks hold a requirement, and a lost id kept only in an untracked file is named with a git add that, run as printed, lets the check pass; the reference's registry is judged by its paths, not today's tree; an archived record counts only when the reference holds no package for that change; a specs directory or registry path outside the project is never sent to Git, and a specs_dir outside the project is refused in specsync check's words naming the setting, and passes once fixed. Real repos (rule 3): same results as the build before on all of them; starling PR #43 still refused with 26 ids. fledge lanes run verify passed at 7f1593b2 through one-lane (3137 unit, 647 integration, 62/62, 100%); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 7f1593b27a9c6d68387f99be8889b66c2c482dc1
content: 80005ce4d7a374de59524b2cb9e8bfcb0ee2c3f17a3480d0d402a9eff4e7bcf6
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 7f1593b2
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-20T04:40:49Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Codex's fourth round on #1108 (3 P1 fixed; P2s filed as #1116 and #1117) and the tester's outside specs directory, on Leif's 'continue with 95% confidence' (2026-11-20 about 04:09Z). REQ-change-169 sent back and re-approved with one replacement, 'in none of the tracked ones in the working tree' (e36fc35e). Red first 86add4bc (four tests, one through the binary), fix b8538c70: only files Git tracks hold a requirement, and a lost id kept only in an untracked file is named with a git add that, run as printed, lets the check pass; the reference's registry is judged by its paths, not today's tree; an archived record counts only when the reference holds no package for that change; a specs directory or registry path outside the project is never sent to Git, and a specs_dir outside the project is refused in specsync check's words naming the setting, and passes once fixed. Real repos (rule 3): same results as the build before on all of them; starling PR #43 still refused with 26 ids. fledge lanes run verify passed at 7f1593b2 through one-lane (3137 unit, 647 integration, 62/62, 100%); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 7f1593b2 at revision 7f1593b27a9c, observed 2026-11-20T04:40:49Z.
