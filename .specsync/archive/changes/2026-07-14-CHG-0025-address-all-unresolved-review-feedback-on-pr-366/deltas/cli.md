## ADDED

### REQUIREMENT REQ-cli-003

The root CLI dispatcher SHALL fail closed when a configured verification child re-enters lifecycle checking or mutation.

Acceptance Criteria

- `check`, `change`, and `lifecycle` command families consult the inherited verification context before dispatch.
- A blocked nested command exits non-zero with one actionable diagnostic.
- Commands outside the lifecycle boundary preserve current dispatch behavior.

## MODIFIED

### SPEC SECTION Purpose

The `specsync` command-line entry point parses global options, blocks configured verification children from recursively dispatching `check`, `change`, or `lifecycle` commands, routes canonical validation and verified SDD lifecycle commands to focused handlers, and preserves equivalent human-readable and structured output without owning domain policy.
