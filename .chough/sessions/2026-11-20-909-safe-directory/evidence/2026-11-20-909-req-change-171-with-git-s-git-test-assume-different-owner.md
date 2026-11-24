---
kind: evidence
id: 2026-11-20-909-req-change-171-with-git-s-git-test-assume-different-owner
aliases: []
claim: "#909 (REQ-change-171), with Git's GIT_TEST_ASSUME_DIFFERENT_OWNER: a repository the user's global file trusts (its top level, or safe.directory = *) passes change status, audit and check, also from a linked worktree; the private file is gone after each command and none is made for a repository the user owns; without the entry the refusal names dubious ownership, the path and git config --global --add safe.directory. Real repos (rule 3): Chough main and spec-sync main with the switch: 6.3.0 refuses even when trusted ('unable to inspect changed paths'), this build passes when trusted and names the command when not; without the switch both pass. Not run: the container runs with a real second owner (Debian 12, Alpine 3.16, Ubuntu 22.04). Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb."
result: passed
evidence_kind: product-test
revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
content: 956d73e9c39c4f15679f69f6f14f374c23eaad5b00edc19a55feadf27f83018b
environment: local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at c783e8cf
criteria: [CHANGE-7]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T06:14:57Z
actor:
  - agent: claude-code
    terminal: t4t
---
**#909 (REQ-change-171), with Git's GIT_TEST_ASSUME_DIFFERENT_OWNER: a repository the user's global file trusts (its top level, or safe.directory = *) passes change status, audit and check, also from a linked worktree; the private file is gone after each command and none is made for a repository the user owns; without the entry the refusal names dubious ownership, the path and git config --global --add safe.directory. Real repos (rule 3): Chough main and spec-sync main with the switch: 6.3.0 refuses even when trusted ('unable to inspect changed paths'), this build passes when trusted and names the command when not; without the switch both pass. Not run: the container runs with a real second owner (Debian 12, Alpine 3.16, Ubuntu 22.04). Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at c783e8cf at revision 349d40236b99, observed 2026-11-20T06:14:57Z.
