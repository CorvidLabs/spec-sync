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
- `ChangeAction` declares every lifecycle, inspection, checking, adoption, and semantic-succession operation.
- `ChangeAction::Supersede` requires change ID, predecessor ID, path, module, and predecessor entry digest.
- `ChangeAction::Reopen` requires a change ID, explicit human actor, and non-empty reason input.
- `ChangeAction::Approve` exposes an explicit `--portable-5-0-1` switch for the atomic marked dual-engine definition event.

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

### REQ-cli-args-005

The shared CLI grammar SHALL expose one explicit audited command for correcting an exact canonical
owner used by reopened acceptance evidence.

Acceptance Criteria

- `change correct-owner` requires a change ID, exact portable path, canonical spec module, human
  actor, and non-empty reason.
- The canonical module is provided through `--spec` and remains distinct from semantic rescoping.
- Missing path, module, actor, or reason inputs fail through deterministic Clap errors before any
  domain mutation.

### REQ-cli-args-006

The shared CLI grammar SHALL expose batch selection for `change correct-owner` while keeping actor
and reason mandatory and rejecting empty or conflicting selection modes before domain mutation.

Acceptance Criteria

- `--path` and `--spec` are repeatable; one `--spec` may apply to every path, or path/spec counts must match.
- `--manifest` accepts a JSON array of path/module objects or TSV `path<TAB>module` lines.
- `--all-missing` requires exactly one `--spec` and excludes `--path`/`--manifest`.
- Actor and reason remain required.
- Empty or conflicting selection fails through deterministic Clap errors before domain mutation.

### REQ-cli-args-007

The shared CLI grammar SHALL expose the 5.0 ledger migration as an optional source-family
positional on the `migrate` command.

Acceptance Criteria

- `specsync migrate 5.0` selects the ledger backfill mode; bare `specsync migrate` keeps the
  v3→v4 default.
- An unknown source family fails through a deterministic Clap validation error before any
  mutation.
- `--dry-run` and `--no-backup` remain accepted in both modes.

### REQ-cli-args-008

The shared CLI grammar SHALL allow semantic supersession to omit a manually copied predecessor
entry digest.

Acceptance Criteria

- `change supersede` accepts predecessor, path, and canonical spec without `--digest`.
- Omission delegates exact digest resolution to signed predecessor evidence.
- A supplied `--digest` remains accepted and is verified by the same domain checks.
