---
spec: types.spec.md
---

## User Stories

- As a contributor, I want shared deterministic types in one dependency-light module.
- As a language maintainer, I want consistent extension and test-pattern mapping.
- As an integrator, I want stable validation and coverage result structures.

## Constraints

- The module has no dependency on other SpecSync modules.
- Default values are deterministic, local, and usable.
- MSRV remains 1.89.

### REQ-types-001

The `types` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-types-002

The shared language registry SHALL classify `.mjs` and `.cjs` as TypeScript-family sources for both direct detection and default extension discovery.

Acceptance Criteria

- Direct extension lookup maps `mjs` and `cjs` to TypeScript.
- The TypeScript default extension list includes `mjs` and `cjs` alongside existing JavaScript and TypeScript suffixes.
- Explicit source-extension filtering remains unchanged.

### REQ-types-003

Shared validation types SHALL represent planned draft mappings as notices distinct from errors and warnings.

Acceptance Criteria

- Notices carry deterministic human-readable messages.
- Notices do not increment warning counts or fail strict validation.
- The shared configuration type exposes the default-false draft-file enforcement setting.

### REQ-types-004

The default enforcement mode SHALL be strict, so that a validation error exits non-zero
without an explicit flag.

Acceptance Criteria

- A bare `specsync check` over a tree with a validation error exits 1.
- Warnings do not gate unless `--strict` is supplied.
- `--enforcement warn` remains available and restores non-blocking behavior.
- `--enforcement`, `--strict`, and `--require-coverage` precedence is otherwise unchanged.

### REQ-types-005

`ValidationResult` SHALL record what validation was able to observe, so a reporter can
distinguish a check that passed from a check that did not run.

Acceptance Criteria
- Whether any mapped source file was present and readable is recorded.
- Whether the spec's Public API names at least one symbol is recorded.
- Both are recorded even when section and export validation are skipped.

### REQ-types-006

The coverage report SHALL carry the symlinked entries that discovery skipped.

Acceptance Criteria
- Skipped entries are reported in a deterministic order.
- An inconclusive coverage result reports no skipped entries rather than omitting the field.

### REQ-types-007

Configuration SHALL record when it is the built-in defaults standing in for a config file
that exists but could not be loaded.

Acceptance Criteria
- A successful load records no such condition.
- A config file that cannot be read, and one that cannot be parsed, both record it, naming the file.
- The absence of a config file is not recorded as a failure, because the defaults are then the intended configuration.
