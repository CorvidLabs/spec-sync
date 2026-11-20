---
kind: evidence
id: 2026-11-20-codex-round-1-on-1108-76b59636-seven-findings-three-p1-four-p2
aliases: []
claim: "Codex round 1 on #1108 (76b59636): seven findings (three P1, four P2), all fixed within REQ-change-169's approved text, as ss-lead confirmed. Red first: b8e4d9cd's six new tests failed on 76b59636, each for its finding; the archived-removal test now builds a real archived record. Fix 3d3492ef: (1) a spec registered at the project root brings in the root's own requirements.md and <module>.req.md, not a walk of the tree (a stray file elsewhere is still not read); (2) a finalization retried after acceptance with its workspace active asks the loss question again, and the audit asks it of such an accepted change; (3) the configuration and registry at the reference are read through Git and join the scope; (4, 5) one mechanism gives removal authority: an active change past its draft, or a real archived record that resolves to its package with a valid definition approval; a retired change or a bare archived delta gives none; (6) every Git listing is -z (non-ASCII module path tested with core.quotePath on); (7) the working tree is read to any depth. change check --commit re-recorded verification (903c34af, fbeb3f99). Real repos (rule 3): change audit --strict gives the same exit codes and errors as 6.3.0 and as the build before on spec-sync main 10d9bb3f, Chough main 4c764c09, Chough #311 and #310, starling c2863c8 and two other projects; starling PR #43's last open commit is still refused with all 26 ids. Fixes 1 and 3 changed no real result: none registers a spec at the root, and none moved specs_dir or its registry since the merge-base. fledge lanes run verify passed at fbeb3f99 through one-lane (3120 unit, 645 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: fbeb3f99f5d24205a95687c235e1a4f68bdc4dde
content: 8d3b06a8516a072ae5057c1d5a8ef749ed6ef652b325d75c61ce6d95507e3188
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at fbeb3f99
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-20T02:03:56Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Codex round 1 on #1108 (76b59636): seven findings (three P1, four P2), all fixed within REQ-change-169's approved text, as ss-lead confirmed. Red first: b8e4d9cd's six new tests failed on 76b59636, each for its finding; the archived-removal test now builds a real archived record. Fix 3d3492ef: (1) a spec registered at the project root brings in the root's own requirements.md and <module>.req.md, not a walk of the tree (a stray file elsewhere is still not read); (2) a finalization retried after acceptance with its workspace active asks the loss question again, and the audit asks it of such an accepted change; (3) the configuration and registry at the reference are read through Git and join the scope; (4, 5) one mechanism gives removal authority: an active change past its draft, or a real archived record that resolves to its package with a valid definition approval; a retired change or a bare archived delta gives none; (6) every Git listing is -z (non-ASCII module path tested with core.quotePath on); (7) the working tree is read to any depth. change check --commit re-recorded verification (903c34af, fbeb3f99). Real repos (rule 3): change audit --strict gives the same exit codes and errors as 6.3.0 and as the build before on spec-sync main 10d9bb3f, Chough main 4c764c09, Chough #311 and #310, starling c2863c8 and two other projects; starling PR #43's last open commit is still refused with all 26 ids. Fixes 1 and 3 changed no real result: none registers a spec at the root, and none moved specs_dir or its registry since the merge-base. fledge lanes run verify passed at fbeb3f99 through one-lane (3120 unit, 645 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at fbeb3f99 at revision fbeb3f99f5d2, observed 2026-11-20T02:03:56Z.
