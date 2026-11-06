## ADDED

### REQUIREMENT REQ-cmd-change-028

`specsync change` command handlers SHALL read repository facts through the per-process repository-fact memo (REQ-git-utils-008) and committed file contents at fixed commits through the batched object reader (REQ-git-utils-009) instead of spawning one git process per read. Every handler path that commits, writes the index, moves a ref or switches branches SHALL invalidate the memo for that repository before any later read. Command output and exit status SHALL be unchanged.

#### Acceptance Criteria

- `change check --commit`, `change finalize` and `change ship` read fresh repository facts after their own commit.
- Lifecycle command output and exit codes are byte-identical to the unmemoized behavior in the existing command tests.
