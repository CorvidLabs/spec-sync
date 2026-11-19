---
kind: evidence
id: 2026-11-19-option-2-propose-don-t-approve-as-leif-approved-it-approve
aliases: []
claim: "Option 2 (propose, don't approve), as Leif approved it ('approve 1089', 14:59Z). Red first: fe86dd9e's reworked tests failed 10 of 24 on the record-based code; the fix is 1141ea78. Now 24 unit tests and 6 integration tests pass: the proposal is written into the delta and refused, naming the requirement and 'specsync change approve <id>'; the canonical file and approvals.json stay byte for byte; status shows the delta as changed after approval; after approval the check applies it; each working-tree text (main's, the change's, the merge) gets the same proposal; a second main edit is proposed again; overlap, main's deletion, a heading main edited, other working-tree text, the 4,000-word limit and an unreadable main text each refuse naming the cause and the way on; one item that cannot be proposed holds back all; no proposal in CI, with no default branch, or over the change's own text (#654). Real cases (rule 3): the tester's three English cases and Chough's seven real stale triples give 9 proposals and the #336 overlap refusal; after approval the applied text is byte-identical to round 4's rebase (88867715) in all 10. Chough's open PRs #311 and #310: the same check and strict-audit results under released 6.3.0 and this build (no new refusal). Chough main 4c764c09, starling main db97480 and spec-sync main ee76ef1b audit --strict clean under both. fledge lanes run verify passed at fbe085f3 (3101 unit, 633 integration, 62/62 specs, 100% coverage); change audit --strict exit 0, pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: fbe085f31a2051fb0f289503c7f605c2d712daaa
content: c9709671caa061169fc30d26d43c00bbb956e0d1d105d63093f30ff7e285a7ce
environment: local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at fbe085f3
criteria: [CHANGE-6.a, CHANGE-6.c]
requirements: [REQ-change-161, REQ-change-122]
contracts: []
limitations: []
observed: 2026-11-19T15:37:08Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Option 2 (propose, don't approve), as Leif approved it ('approve 1089', 14:59Z). Red first: fe86dd9e's reworked tests failed 10 of 24 on the record-based code; the fix is 1141ea78. Now 24 unit tests and 6 integration tests pass: the proposal is written into the delta and refused, naming the requirement and 'specsync change approve <id>'; the canonical file and approvals.json stay byte for byte; status shows the delta as changed after approval; after approval the check applies it; each working-tree text (main's, the change's, the merge) gets the same proposal; a second main edit is proposed again; overlap, main's deletion, a heading main edited, other working-tree text, the 4,000-word limit and an unreadable main text each refuse naming the cause and the way on; one item that cannot be proposed holds back all; no proposal in CI, with no default branch, or over the change's own text (#654). Real cases (rule 3): the tester's three English cases and Chough's seven real stale triples give 9 proposals and the #336 overlap refusal; after approval the applied text is byte-identical to round 4's rebase (88867715) in all 10. Chough's open PRs #311 and #310: the same check and strict-audit results under released 6.3.0 and this build (no new refusal). Chough main 4c764c09, starling main db97480 and spec-sync main ee76ef1b audit --strict clean under both. fledge lanes run verify passed at fbe085f3 (3101 unit, 633 integration, 62/62 specs, 100% coverage); change audit --strict exit 0, pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at fbe085f3 at revision fbe085f31a20, observed 2026-11-19T15:37:08Z.
