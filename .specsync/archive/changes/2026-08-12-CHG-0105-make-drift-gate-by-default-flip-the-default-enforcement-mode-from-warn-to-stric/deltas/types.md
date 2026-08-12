## ADDED

### REQUIREMENT REQ-types-004

The default enforcement mode SHALL be strict, so that a validation error exits non-zero
without an explicit flag.

Acceptance Criteria

- A bare `specsync check` over a tree with a validation error exits 1.
- Warnings do not gate unless `--strict` is supplied.
- `--enforcement warn` remains available and restores non-blocking behavior.
- `--enforcement`, `--strict`, and `--require-coverage` precedence is otherwise unchanged.
