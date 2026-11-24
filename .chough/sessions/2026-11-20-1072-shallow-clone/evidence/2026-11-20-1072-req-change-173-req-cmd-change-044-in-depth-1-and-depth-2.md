---
kind: evidence
id: 2026-11-20-1072-req-change-173-req-cmd-change-044-in-depth-1-and-depth-2
aliases: []
claim: "#1072 (REQ-change-173, REQ-cmd-change-044): in depth-1 and depth-2 clones of an adopted project change new refuses naming the shallow clone and git fetch --unshallow origin and leaves no workspace; status, check and audit name the fetch; after the fetch they pass; a full clone keeps the baseline message; text status in a clone deep enough for the baseline gives the shallow refusal instead of the ledger text. Real repos (rule 3): depth-1 and depth-50 clones of public spec-sync: 6.3.0's change new wrote a workspace, this build refuses naming the fetch and writes nothing; after git fetch --unshallow origin the audit passes. Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb."
result: passed
evidence_kind: product-test
revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
content: 956d73e9c39c4f15679f69f6f14f374c23eaad5b00edc19a55feadf27f83018b
environment: local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at c783e8cf
criteria: [CHANGE-7, CHANGE-1]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T06:14:56Z
actor:
  - agent: claude-code
    terminal: t4t
---
**#1072 (REQ-change-173, REQ-cmd-change-044): in depth-1 and depth-2 clones of an adopted project change new refuses naming the shallow clone and git fetch --unshallow origin and leaves no workspace; status, check and audit name the fetch; after the fetch they pass; a full clone keeps the baseline message; text status in a clone deep enough for the baseline gives the shallow refusal instead of the ledger text. Real repos (rule 3): depth-1 and depth-50 clones of public spec-sync: 6.3.0's change new wrote a workspace, this build refuses naming the fetch and writes nothing; after git fetch --unshallow origin the audit passes. Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at c783e8cf at revision 349d40236b99, observed 2026-11-20T06:14:56Z.
