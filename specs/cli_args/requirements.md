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
- `ChangeAction::Reopen` requires a change ID, explicit human actor, and non-empty reason input.

### REQ-cli-args-002

The shared CLI grammar SHALL expose deterministic generation without embedded inference selection.

Acceptance Criteria
- `generate` retains deterministic module, uncovered, and batch selection.
- `generate` exposes no provider or model flags.
- Agent installation and MCP commands remain available.

### REQ-cli-args-003

The shared CLI grammar SHALL describe the files produced by initialization and full spec scaffolding accurately.

Acceptance Criteria
- `init` help names `.specsync/config.toml` rather than the retired root JSON configuration.
- Global option help names the canonical configuration and every accepted output format.
- `add-spec` help describes the required companion set and optional design artifact.
- `new --full` help lists the required companion files and identifies `design.md` as optional.
- Help-only corrections do not change argument parsing or command behavior.

### REQ-cli-args-004

The shared CLI grammar SHALL expose a complete explicit command for supported accepted interview
metadata correction.

Acceptance Criteria

- `change correct` requires a change ID, supported field, `yes` or `no` value, human actor, and
  non-empty reason input.
- Help distinguishes accepted metadata correction from delivery-only `change reopen`.
- Missing audit arguments and invalid field/value choices fail through deterministic Clap errors.

