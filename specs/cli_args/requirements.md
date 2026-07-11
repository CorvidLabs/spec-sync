---
spec: cli_args.spec.md
---

## User Stories

- As a user, I want one consistent Clap grammar and global flags.
- As a developer, I want deterministic generation arguments without credential-bearing inference choices.
- As an agent user, I want native Agents, MCP, Lifecycle, and Change command surfaces preserved.

### REQ-cli-args-001

The system SHALL declare the complete verified SDD change command grammar in the shared Clap parser.

Acceptance Criteria
- `Command` includes the `Change` namespace.
- `ChangeAction` declares every lifecycle, inspection, checking, and adoption operation.
