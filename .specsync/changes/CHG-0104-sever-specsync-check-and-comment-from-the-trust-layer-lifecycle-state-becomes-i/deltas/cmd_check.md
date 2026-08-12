## ADDED

### REQUIREMENT REQ-cmd-check-004

The primary check command SHALL treat SDD lifecycle state as information and SHALL NOT
derive its exit status from it.

Acceptance Criteria

- The number of active changes is reported without affecting exit status in any supported
  output format.
- Workspace files that cannot be parsed, or that record an illegal state, produce an explicit
  shape warning rather than a gate failure.
- Exit status derives solely from spec validation results, the effective enforcement mode,
  `--strict`, and `--require-coverage`.
- Lifecycle gating remains reachable through the `change` verbs and `specsync change audit`,
  whose behavior is unchanged.
