---
kind: evidence
id: 2026-11-24-1149-policy-flip-red-at-a8b31fe8
aliases: []
claim: "Red first, through the binary at published a8b31fe8 (REQ-cmd-check-004). Default branch has SDD enabled; a branch drops REQ-auth-002 with no approved removal. With the policy kept, check --strict and change audit --strict both exit 1 and name the loss. With enabled set to false, or sdd.json deleted, on the branch (committed or not), change audit --strict still exits 1 and names the loss and check --strict exits 0. Controls pass both commands: default branch never had a policy; default branch has it disabled."
result: failed
evidence_kind: product-test
method: Real CLI in disposable Git projects, debug binary fingerprinted to its source at a8b31fe8
revision: 2e2ddab1af06a6b8e877a3efd6fb40801786a2fc
content: 43fd384c7f5740d4a6e1e72fea540e6ec27ed19b7e2db875ff6e9fea8cededde
environment: macos-arm64-local
criteria: [CHANGE-17.d]
requirements: []
contracts: []
limitations: ["macOS arm64 only; ten cases, five uncommitted and the same five committed"]
observed: 2026-11-24T04:45:26Z
actor:
  - agent: claude-code
    terminal: t14v
---
**Red first, through the binary at published a8b31fe8 (REQ-cmd-check-004). Default branch has SDD enabled; a branch drops REQ-auth-002 with no approved removal. With the policy kept, check --strict and change audit --strict both exit 1 and name the loss. With enabled set to false, or sdd.json deleted, on the branch (committed or not), change audit --strict still exits 1 and names the loss and check --strict exits 0. Controls pass both commands: default branch never had a policy; default branch has it disabled.**

Recorded as failed in macos-arm64-local at revision 2e2ddab1af06, observed 2026-11-24T04:45:26Z.

## Key failing lines

- worktree  check=0  audit=1(names loss)  default enabled, branch sets enabled to false
