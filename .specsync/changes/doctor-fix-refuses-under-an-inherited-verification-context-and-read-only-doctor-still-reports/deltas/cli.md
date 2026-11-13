## MODIFIED

### REQUIREMENT REQ-cli-003

The root CLI dispatcher SHALL fail closed when a `check`, `change`, or `lifecycle` command, or `doctor --fix` (with or without `--dry-run` or `--close-late`), runs under an inherited `SPECSYNC_VERIFICATION_CONTEXT`. A `doctor` without `--fix` only reads, and SHALL still report there.

Acceptance Criteria

- `check`, `change`, and `lifecycle` command families, and `doctor --fix`, consult the inherited verification context before dispatch.
- A blocked nested command exits non-zero with one actionable diagnostic.
- `doctor` without `--fix` gives the same output and exit status under an inherited verification context as without one.
- Commands outside the lifecycle boundary preserve current dispatch behavior.
- SpecSync sets that marker on no child of its own, because verification executes no configured command (REQ-change-023), so the refusal applies only to a context a caller outside SpecSync supplies.

### SPEC SECTION Purpose

The `specsync` command-line entry point parses global options, refuses `check`, `change`, `lifecycle` and `doctor --fix` commands that run under an inherited `SPECSYNC_VERIFICATION_CONTEXT` (SpecSync starts no verification child of its own), routes canonical validation, verified SDD lifecycle commands and `specsync doctor` (REQ-cmd-change-035) to focused handlers, and preserves equivalent human-readable and structured output without owning domain policy.
