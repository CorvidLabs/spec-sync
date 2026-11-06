# Describe the guard as refusing an inherited verification context, not a SpecSync-made child

## MODIFIED

### REQUIREMENT REQ-cli-003

The root CLI dispatcher SHALL fail closed when a `check`, `change`, or `lifecycle` command runs under an inherited `SPECSYNC_VERIFICATION_CONTEXT`.

Acceptance Criteria

- `check`, `change`, and `lifecycle` command families consult the inherited verification context before dispatch.
- A blocked nested command exits non-zero with one actionable diagnostic.
- Commands outside the lifecycle boundary preserve current dispatch behavior.
- SpecSync sets that marker on no child of its own, because verification executes no configured command (REQ-change-023), so the refusal applies only to a context a caller outside SpecSync supplies.

### SPEC SECTION Purpose

The `specsync` command-line entry point parses global options, refuses `check`, `change`, or `lifecycle` commands that run under an inherited `SPECSYNC_VERIFICATION_CONTEXT` (SpecSync starts no verification child of its own), routes canonical validation and verified SDD lifecycle commands to focused handlers, and preserves equivalent human-readable and structured output without owning domain policy.
